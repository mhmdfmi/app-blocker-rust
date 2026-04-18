/// Thread watcher konfigurasi - mendeteksi perubahan file dan trigger hot reload.
/// Menggunakan crate `notify` untuk file system events.
use crate::core::events::AppEvent;
use crate::utils::error::{AppError, AppResult};
use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{channel, Sender};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, error, info, warn};

/// Spawn thread yang watch perubahan file konfigurasi dan kirim event reload
///
/// Menggunakan debounce 500ms agar tidak trigger berulang kali
pub fn spawn_config_watcher(
    config_path: PathBuf,
    event_tx: Sender<AppEvent>,
    shutdown_flag: Arc<AtomicBool>,
) -> AppResult<()> {
    let path_display = config_path.display().to_string();

    std::thread::Builder::new()
        .name("app-blocker-config-watcher".to_string())
        .spawn(move || {
            if let Err(e) = run_watcher(config_path, event_tx, shutdown_flag) {
                error!(error = %e, "Config watcher error");
            }
        })
        .map_err(|e| AppError::System(format!("Gagal spawn config watcher: {e}")))?;

    info!(path = %path_display, "Config file watcher dimulai");
    Ok(())
}

fn run_watcher(
    config_path: PathBuf,
    event_tx: Sender<AppEvent>,
    shutdown_flag: Arc<AtomicBool>,
) -> AppResult<()> {
    let (tx, rx) = channel::<notify::Result<Event>>();

    let mut watcher = RecommendedWatcher::new(tx, Config::default()
        .with_poll_interval(Duration::from_millis(500)))
        .map_err(|e| AppError::Config(format!("Gagal buat file watcher: {e}")))?;

    // Watch direktori parent agar deteksi create/rename juga
    let watch_dir = config_path
        .parent()
        .unwrap_or(Path::new("."));

    watcher.watch(watch_dir, RecursiveMode::NonRecursive)
        .map_err(|e| AppError::Config(format!("Gagal watch direktori config: {e}")))?;

    let mut last_reload = std::time::Instant::now();
    let debounce = Duration::from_millis(500);

    loop {
        if shutdown_flag.load(Ordering::SeqCst) {
            break;
        }

        match rx.recv_timeout(Duration::from_millis(200)) {
            Ok(Ok(event)) => {
                // Filter hanya event yang relevan dengan file config kita
                let is_our_file = event.paths.iter().any(|p| p == &config_path);
                let is_write = matches!(
                    event.kind,
                    EventKind::Create(_) | EventKind::Modify(_)
                );

                if is_our_file && is_write && last_reload.elapsed() >= debounce {
                    last_reload = std::time::Instant::now();
                    info!("Perubahan file konfigurasi terdeteksi, mengirim event reload");

                    if let Err(e) = event_tx.send(AppEvent::ConfigReloaded) {
                        error!(error = %e, "Gagal kirim event ConfigReloaded");
                        break; // Channel mati, hentikan watcher
                    }
                }
            }
            Ok(Err(e)) => {
                warn!(error = %e, "Error dari file watcher");
            }
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                // Normal - lanjut loop
            }
            Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                error!("Config watcher channel terputus");
                break;
            }
        }
    }

    info!("Config file watcher selesai");
    Ok(())
}
