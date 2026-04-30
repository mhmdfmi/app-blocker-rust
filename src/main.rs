// #![windows_subsystem = "windows"]
/// Entry point App Blocker v1.1.3
/// Fix: Arc::try_unwrap anti-pattern, ctrlc real handler, config watcher thread,
///      student mode, audit logger, ConfigManager di engine.
/// Update: Semua config dari database, tanpa .env / TOML
/// Note: Pastikan database sudah terinisialisasi dengan benar sebelum menjalankan aplikasi.
use app_blocker_lib::{
    cli::{run_command, Cli, Commands},
    config::DbConfigLoader,
    constants::paths,
    core::{
        audit::{init_global_audit, AuditEntry, AuditEventKind},
        monitor::MonitorThread,
        AppEngine, AppEvent, StateManager, WatchdogThread,
    },
    db::init::init_database,
    metrics::AppMetrics,
    repository::ConfigRepository,
    security::{
        auth::{Argon2AuthService, AuthManager, DEFAULT_PASSWORD},
        integrity::IntegrityService,
    },
    system::{
        service::{acquire_single_instance_lock, is_disable_flag_active},
        student_mode::StudentModeConfig,
        WindowsProcessService,
    },
    ui::{run_overlay, DisplayData},
    utils::{
        error::{AppError, AppResult},
        logger::{flush_logs, init_logger},
    },
};
use clap::Parser;
use std::{
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc, Arc, Mutex,
    },
    time::Duration,
};
use tokio::runtime::Runtime;
use tracing::{error, info, warn};

fn main() {
    let cli: Cli = Cli::parse();

    if let Some(cmd) = &cli.command {
        match cmd {
            Commands::RunSimulation | Commands::RunProduction => {
                // Lanjut ke startup penuh
            }
            _ => {
                run_command(&cli, cmd);
                return;
            }
        }
    }

    match startup(cli) {
        Ok(_) => {}
        Err(e) => {
            eprintln!("FATAL: App Blocker gagal: {e}");
            std::process::exit(1);
        }
    }
}

fn startup(cli: Cli) -> AppResult<()> {
    // ── 0. Ensure AppData directory exists ───────────────────────
    let _appdata_dir: PathBuf = paths::ensure_appdata_dir()
        .map_err(|e| AppError::Config(format!("Failed to create AppData dir: {}", e)))?;
    info!(path = %paths::get_appdata_dir().display(), "AppData directory ready");

    // ── 1. Initialize Database + Load ALL Config from DB ──────────────────
    // Semua config dari database - tidak ada .env / TOML
    info!("Initializing database and loading config...");
    let rt: Runtime = tokio::runtime::Runtime::new()?;

    // Init DB dan load semua config ke memory cache
    let (_db, config_arc, password_hash) = rt
        .block_on(async {
            let db = init_database(&paths::get_db_path())
                .await
                .map_err(|e| AppError::Database(e.to_string()))?;

            let loader: DbConfigLoader = DbConfigLoader::new_with_load(db.clone())
                .await
                .map_err(|e: AppError| AppError::Config(e.to_string()))?;

            // Ambil password hash dari DB (atau generate jika belum ada)
            let config_repo = ConfigRepository::new(db.clone());
            let password_hash = match config_repo.get_value("security.password_hash").await {
                Ok(Some(hash)) if !hash.is_empty() => hash,
                _ => {
                    // Generate hash baru dari default password
                    info!(
                        "Password hash belum ada di DB, generate dari default password {}...",
                        DEFAULT_PASSWORD
                    );
                    let (_svc, hash) = Argon2AuthService::with_default_password()?;
                    // Simpan ke DB
                    let _ = config_repo
                        .set("security.password_hash", &hash, Some("Admin password hash"))
                        .await;
                    hash
                }
            };

            info!("Config loaded from database - password hash ready");

            Ok::<_, AppError>((db, loader.get_arc(), password_hash))
        })
        .map_err(|e| AppError::Config(format!("DB init failed: {}", e)))?;

    // Ambil config dari Arc
    let config = config_arc
        .read()
        .map_err(|e| AppError::Config(format!("Read config: {}", e)))?
        .clone();

    // ── 2. Inisialisasi logger ────────────────────────────────────────────────
    // Log level dari DB, fallback ke "info"
    let log_level = config.logging.level.clone();
    // Gunakan path dari config, atau fallback ke AppData
    let log_dir = if config.logging.path.starts_with("C:\\") || config.logging.path.starts_with("/")
    {
        PathBuf::from(&config.logging.path)
    } else {
        paths::get_logs_dir()
    };
    init_logger(&log_dir, &log_level)?;

    info!(
        version  = env!("CARGO_PKG_VERSION"),
        mode     = %config.app.mode,
        db_path  = %paths::get_db_path().display(),
        "App Blocker v{} dimulai", env!("CARGO_PKG_VERSION")
    );

    // ── 4. Inisialisasi audit logger ──────────────────────────────────────────
    // Gunakan direktori reports dari paths.rs (AppData\Local\AppBlocker\reports)
    init_global_audit(&paths::get_reports_dir()).unwrap_or_else(|e| {
        warn!(error = %e, "Audit logger gagal diinisialisasi (non-fatal)");
    });
    app_blocker_lib::core::audit::audit(
        AuditEntry::new(AuditEventKind::SystemStarted).with_detail(format!(
            "v{} mode={}",
            env!("CARGO_PKG_VERSION"),
            config.app.mode
        )),
    );

    // ── 5. Single instance check ──────────────────────────────────────────────
    let _instance_guard = acquire_single_instance_lock(PathBuf::from(paths::LOCK_FILE))?;

    // ── 6. Integrity & anti-debug ─────────────────────────────────────────────
    let _integrity = IntegrityService::new()?;
    if config.security.anti_debugging {
        use app_blocker_lib::security::integrity::is_debugger_present;
        if is_debugger_present() {
            warn!("Debugger terdeteksi - masuk safe mode");
        }
    }

    // ── 7. Setup autentikasi ──────────────────────────────────────────────────
    // Password hash sudah di-load di awal startup - gunakan dari DB
    info!("Auth service initialized from DB config");

    // Password hash sudah di-load dari async block pertama

    let auth_svc: Argon2AuthService = Argon2AuthService::new(password_hash)?;
    let auth_manager_shared: Arc<Mutex<AuthManager>> = Arc::new(Mutex::new(AuthManager::new(
        Box::new(auth_svc),
        config.security.max_auth_attempts,
        config.security.lockout_duration_seconds,
    )));

    // ── 8. Inisialisasi state manager ─────────────────────────────────────────
    let state_manager: Arc<StateManager> = Arc::new(StateManager::new());

    // ── 9. Metrics ────────────────────────────────────────────────────────────
    let _metrics: Arc<AppMetrics> = AppMetrics::new();

    // ── 10. Shutdown flag + Ctrl+C handler ───────────────────────────────────
    // FIX: ctrlc real implementation (bukan stub)
    let shutdown_flag: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
    {
        let sf = Arc::clone(&shutdown_flag);
        ctrlc::set_handler(move || {
            info!("Ctrl+C diterima, memulai shutdown graceful...");
            sf.store(true, Ordering::SeqCst);
        })
        .unwrap_or_else(|e| warn!(error = %e, "Gagal pasang Ctrl+C handler"));
    }

    // ── 11. Startup delay ────────────────────────────────────────────────────
    let delay: u64 = if config.simulation.enabled {
        0
    } else {
        config.app.startup_delay_seconds.min(5)
    };
    if delay > 0 {
        info!(delay_seconds = delay, "Menunggu sistem siap...");
        std::thread::sleep(Duration::from_secs(delay));
    }

    // ── 12. Cek flag disable darurat ─────────────────────────────────────────
    if is_disable_flag_active() {
        warn!("Flag disable aktif saat startup - safe mode");
        state_manager.force_safe_mode("disable_flag_on_startup")?;
    }

    // ── 13. Setup channel komunikasi ─────────────────────────────────────────
    let (event_tx, event_rx) = mpsc::channel::<AppEvent>();

    // ── 14. Mode simulasi ─────────────────────────────────────────────────────
    let simulation_mode =
        config.simulation.enabled || matches!(cli.command, Some(Commands::RunSimulation));
    if simulation_mode {
        warn!("MODE SIMULASI aktif - proses tidak akan benar-benar dihentikan");
    }

    // ── 15. Spawn Monitor thread ──────────────────────────────────────────────
    {
        let tx = event_tx.clone();
        let sm = Arc::clone(&state_manager);
        let cfg = Arc::clone(&config_arc);
        let sf = Arc::clone(&shutdown_flag);
        let psvc = Box::new(WindowsProcessService::new(simulation_mode));

        std::thread::Builder::new()
            .name("app-blocker-monitor".to_string())
            .spawn(move || match MonitorThread::new(tx, sm, cfg, psvc) {
                Ok(mon) => mon.run(sf),
                Err(e) => error!(error = %e, "Gagal init monitor thread"),
            })
            .map_err(|e| AppError::System(format!("Spawn monitor: {e}")))?;
    }

    // ── 16. Buat Engine ───────────────────────────────────────────────────────
    // auth_manager_shared di-clone untuk overlay callback
    let auth_for_overlay = Arc::clone(&auth_manager_shared);
    let tx_for_overlay_cb = event_tx.clone();
    // config_arc dari DbConfigLoader - config dari database
    // Catatan: set_config_manager di-hilangkan karena config dari DB, bukan TOML

    // Ambil AuthManager untuk engine dengan cara aman (bukan try_unwrap)
    // Engine menerima Arc<Mutex<AuthManager>>, bukan owned AuthManager
    // Buat shim baru agar kompatibel dengan AppEngine::new yang butuh owned AuthManager:
    // Kita buat engine dengan dummy auth lalu set arc-nya
    let engine_auth = {
        // Clone konfigurasi auth untuk engine (bukan move Arc)
        let tmp_svc = Argon2AuthService::new(
            auth_manager_shared
                .lock()
                .map(|m| m.current_hash().to_string())
                .unwrap_or_default(),
        )?;
        AuthManager::new(
            Box::new(tmp_svc),
            config.security.max_auth_attempts,
            config.security.lockout_duration_seconds,
        )
    };

    let mut engine = AppEngine::new(
        event_rx,
        event_tx.clone(),
        Arc::clone(&state_manager),
        Arc::clone(&config_arc),
        Box::new(WindowsProcessService::new(simulation_mode)),
        engine_auth,
    );

    // StudentModeConfig dari DB config
    let student_config = StudentModeConfig {
        enabled: config.blocking.behavior_scoring_enabled,
        disable_task_manager: true,
        disable_registry_tools: true,
        disable_cmd: true,
        apply_only_when_locked: true,
    };
    engine.set_student_mode(student_config);

    // Set overlay callback - spawn thread UI untuk overlay
    engine.set_overlay_callback(Box::new(move |request| {
        let display = DisplayData {
            process_name: request.process_name.clone(),
            pid: request.pid,
            username: request.username.clone(),
            computer_name: request.computer_name.clone(),
            timestamp: request.timestamp.clone(),
            attempts: 0,
            max_attempts: 5,
        };

        let trace_id =
            uuid::Uuid::parse_str(&request.trace_id).unwrap_or_else(|_| uuid::Uuid::new_v4());

        let tx = tx_for_overlay_cb.clone();
        let auth = Arc::clone(&auth_for_overlay);

        std::thread::Builder::new()
            .name("app-blocker-ui".to_string())
            .spawn(move || {
                if let Err(e) = run_overlay(display, auth, tx, trace_id, 30) {
                    error!(error = %e, "Error overlay UI");
                }
            })
            .unwrap_or_else(|e| {
                // Jika spawn gagal, kirim unlock otomatis agar sistem tidak stuck
                error!(error = %e, "Gagal spawn UI thread - auto unlock");
                let _ = tx_for_overlay_cb.send(AppEvent::UnlockSuccess {
                    trace_id,
                    username: "SPAWN_FAILED_AUTO".to_string(),
                    unlocked_at: app_blocker_lib::utils::time::now_utc(),
                });
                // Return dummy handle
                std::thread::spawn(|| {})
            });
    }));

    // ── 17. Spawn Engine thread ───────────────────────────────────────────────
    {
        let sf = Arc::clone(&shutdown_flag);
        std::thread::Builder::new()
            .name("app-blocker-engine".to_string())
            .spawn(move || engine.run(sf))
            .map_err(|e| AppError::System(format!("Spawn engine: {e}")))?;
    }

    // ── 18. Spawn Watchdog thread ─────────────────────────────────────────────
    {
        let wd_cfg = config.watchdog.clone();
        let sm = Arc::clone(&state_manager);
        let tx = event_tx.clone();
        let sf = Arc::clone(&shutdown_flag);

        std::thread::Builder::new()
            .name("app-blocker-watchdog".to_string())
            .spawn(move || {
                WatchdogThread::new(
                    tx,
                    sm,
                    wd_cfg.heartbeat_interval_ms,
                    wd_cfg.max_missed_heartbeats,
                    wd_cfg.max_restart_attempts,
                )
                .run(sf);
            })
            .map_err(|e| AppError::System(format!("Spawn watchdog: {e}")))?;
    }

    // ── 19. Config reload via DB ────────────────────────────────────────────
    // Config dari database - TOML file watcher tidak diperlukan
    // Untuk reload config: call DbConfigLoader::reload() atau CLI/API

    info!("Semua thread berjalan. App Blocker aktif.");
    info!(
        "Tekan Ctrl+C atau buat file '{}' untuk berhenti.",
        paths::DISABLE_FLAG_FILE
    );

    // ── 20. Main thread: tunggu shutdown ─────────────────────────────────────
    while !shutdown_flag.load(Ordering::SeqCst) {
        // Cek flag disable darurat
        if is_disable_flag_active() {
            warn!("Flag disable terdeteksi dari main thread");
            let _ = event_tx.send(AppEvent::DisableFlagDetected);
        }
        std::thread::sleep(Duration::from_secs(2));
    }

    // ── 21. Graceful shutdown ────────────────────────────────────────────────
    info!("Shutdown dimulai...");
    let _ = event_tx.send(AppEvent::ShutdownRequested {
        reason: "user_signal".to_string(),
    });

    // Beri waktu thread selesai
    std::thread::sleep(Duration::from_secs(3));

    // Flush audit
    if let Some(writer) = app_blocker_lib::core::audit::GLOBAL_AUDIT.get() {
        writer.flush();
    }

    // Flush log ke file
    flush_logs();

    info!("App Blocker shutdown selesai.");
    Ok(())
}
