/// Database Config Watcher - polling-based monitoring untuk perubahan config di database.
///
/// Perubahan ini menambahkan fitur monitoring otomatis untuk config dari database:
/// - Polling thread yang memeriksa `updated_at` dari tabel configs
/// - Auto-reload langsung ke cache saat perubahan terdeteksi (tanpa perlu restart)
/// - Validasi config sebelum apply - rollback jika invalid
/// - Thread-safe dan graceful shutdown dengan proper logging
///
/// Fitur utama:
/// - Polling setiap 5 detik (configurable)
/// - Validasi otomatis sebelum apply config baru
/// - Error rollback ke config lama jika validasi gagal
/// - Event logging untuk audit trail
///
/// Keterangan Perubahan (ID): Menambahkan modul DB config watcher dengan polling; Area: src/config/, Dampak OS: thread baru dengan sleep interval; Tes: cargo check
use crate::config::DbConfigLoader;
use crate::core::events::{AppEvent, ComponentId};
use crate::core::watchdog::send_watchdog_heartbeat;
use crate::repository::ConfigRepository;
use crate::utils::error::{AppError, AppResult};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::Sender;
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, error, info, warn};

/// Spawn DB config watcher dengan DbConfigLoader untuk auto-reload config
///
/// Ini adalah fungsi utama yang dipanggil saat startup untuk monitoring perubahan
/// config di database dan langsung apply ke aplikasi tanpa restart.
/// config baru akan divalidasi dulu sebelum diapply - rollback jika invalid.
///
/// # Arguments
/// * `loader` - Arc<DbConfigLoader> instance yang punya config cache (di-share)
/// * `event_tx` - Channel untuk mengirim event ke engine (optional, bisa None)
/// * `shutdown_flag` - Flag untuk graceful shutdown
///
/// # Returns
/// * `Ok(())` jika thread berhasil di-spawn
/// * `Err(AppError)` jika gagal spawn thread
///
/// # Example
/// ```ignore
/// let loader = Arc::new(loader);
/// spawn_db_config_watcher_with_loader(loader, Some(event_tx), shutdown_flag)?;
/// ```
pub fn spawn_db_config_watcher_with_loader(
    loader: Arc<DbConfigLoader>,
    event_tx: Option<Sender<AppEvent>>,
    shutdown_flag: Arc<AtomicBool>,
) -> AppResult<()> {
    std::thread::Builder::new()
        .name("app-blocker-db-config-watcher".to_string())
        .spawn(move || {
            if let Err(e) = run_db_watcher_with_loader(loader, event_tx, shutdown_flag) {
                error!(error = %e, "DB config watcher error - check logs for details");
            }
        })
        .map_err(|e| AppError::System(format!("Gagal spawn DB config watcher: {e}")))?;

    info!("DB config watcher dimulai - monitoring perubahan config database");
    Ok(())
}

/// Run DB watcher loop dengan langsung reload config dari loader
fn run_db_watcher_with_loader(
    loader: Arc<DbConfigLoader>,
    event_tx: Option<Sender<AppEvent>>,
    shutdown_flag: Arc<AtomicBool>,
) -> AppResult<()> {
    let db = loader.db().clone();
    let config_repo = ConfigRepository::new(db);

    // Get initial timestamp untuk comparison
    let mut last_timestamps = match get_all_config_timestamps(&config_repo) {
        Ok(ts) => ts,
        Err(e) => {
            error!(error = %e, "Gagal load initial config timestamps - watcher unable to start");
            return Err(e);
        }
    };

    let mut last_reload = std::time::Instant::now();
    let poll_interval = Duration::from_secs(5); // Default: 5 detik
    let debounce = Duration::from_millis(500);

    info!(
        keys = last_timestamps.len(),
        interval_secs = 5,
        "DB config watcher ready - monitoring config changes"
    );

    loop {
        if shutdown_flag.load(Ordering::SeqCst) {
            info!("DB config watcher: shutdown diterima, stopping...");
            break;
        }

        // Kirim heartbeat ke watchdog - untuk监控 komponen ini
        send_watchdog_heartbeat(ComponentId::ConfigWatcher);

        // Sleep sesuai interval
        std::thread::sleep(poll_interval);

        // Cek perubahan
        match check_config_changes(&config_repo, &mut last_timestamps) {
            Ok(true) => {
                // Perubahan terdeteksi!
                if last_reload.elapsed() >= debounce {
                    last_reload = std::time::Instant::now();
                    info!("Perubahan config database terdeteksi, memulai reload...");

                    // RELOAD CONFIG DENGAN VALIDASI
                    let reload_result =
                        tokio::runtime::Handle::current().block_on(async { loader.reload().await });

                    // Handle result dengan validasi dan error logging
                    match reload_result {
                        Ok(()) => {
                            // Get config untuk logging
                            match loader.get() {
                                Ok(cfg) => {
                                    info!(
                                        mode = %cfg.app.mode,
                                        scan_interval_ms = cfg.monitoring.scan_interval_ms,
                                        "Config berhasil di-reload dan divalidasi"
                                    );
                                }
                                Err(e) => {
                                    warn!(error = %e, "Config reload berhasil tapi gagal read config");
                                }
                            }

                            // Kirim event ke engine jika ada
                            if let Some(tx) = &event_tx {
                                if let Err(e) = tx.send(AppEvent::ConfigReloaded) {
                                    error!(error = %e, "Gagal kirim event ConfigReloaded ke engine");
                                }
                            }
                        }
                        Err(e) => {
                            // VALIDATION ERROR - rollback ke config lama
                            error!(
                                error = %e,
                                "Config reload gagal - mempertahankan config lama (rollback)"
                            );
                            // Tidak perlu kirim event karena config tidak berubah
                        }
                    }
                }
            }
            Ok(false) => {
                // Tidak ada perubahan - normal
                debug!("Tidak ada perubahan config - continue monitoring");
            }
            Err(e) => {
                warn!(error = %e, "Error checking config changes - retrying...");
            }
        }
    }

    info!("DB config watcher selesai");
    Ok(())
}

/// Get semua config timestamps dari database
fn get_all_config_timestamps(
    repo: &ConfigRepository,
) -> AppResult<std::collections::HashMap<String, String>> {
    let configs = tokio::runtime::Handle::current()
        .block_on(repo.find_all())
        .map_err(|e| AppError::Database(format!("Find all configs: {e}")))?;

    let mut timestamps = std::collections::HashMap::new();
    for config in configs {
        timestamps.insert(config.key, config.updated_at);
    }

    Ok(timestamps)
}

/// Cek apakah ada perubahan config
fn check_config_changes(
    repo: &ConfigRepository,
    last_timestamps: &mut std::collections::HashMap<String, String>,
) -> AppResult<bool> {
    // Get all configs dari DB
    let configs = tokio::runtime::Handle::current()
        .block_on(repo.find_all())
        .map_err(|e| AppError::Database(format!("Find all configs: {}", e)))?;

    // Cek setiap config (iterate over reference to avoid move)
    for config in &configs {
        let key = config.key.clone();
        let new_timestamp = config.updated_at.clone();

        // Cek apakah ada perubahan
        if let Some(old_timestamp) = last_timestamps.get(&key) {
            if old_timestamp != &new_timestamp {
                // Perubahan terdeteksi untuk key ini
                info!(key = %key, old = %old_timestamp, new = %new_timestamp, "Config berubah");
                last_timestamps.insert(key, new_timestamp);
                return Ok(true);
            }
        } else {
            // Config baru
            info!(key = %key, "Config baru ditambahkan");
            last_timestamps.insert(key, new_timestamp);
            return Ok(true);
        }
    }

    // Cek config yang dihapus
    let current_keys: Vec<String> = last_timestamps.keys().cloned().collect();
    for key in current_keys {
        let exists = configs.iter().any(|c| c.key == key);
        if !exists {
            warn!(key = %key, "Config dihapus dari database");
            last_timestamps.remove(&key);
            return Ok(true);
        }
    }

    Ok(false)
}
