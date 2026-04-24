/// Entry point App Blocker v1.1.3
/// Fix: Arc::try_unwrap anti-pattern, ctrlc real handler, config watcher thread,
///      student mode, audit logger, ConfigManager di engine.
use app_blocker_lib::{
    cli::{run_command, Cli, Commands},
    config::{env_loader, hot_reload::spawn_config_watcher, ConfigManager},
    constants::paths,
    core::{
        audit::{init_global_audit, AuditEntry, AuditEventKind},
        monitor::MonitorThread,
        AppEngine, AppEvent, StateManager, WatchdogThread,
    },
    metrics::AppMetrics,
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
        logger::init_logger,
    },
};
use clap::Parser;
use std::{
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc, Arc, Mutex,
    },
    time::Duration,
};
use tracing::{error, info, warn};

fn main() {
    let cli = Cli::parse();

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
    // ── 1. Load .env ──────────────────────────────────────────────────────────
    let env_path = Path::new(paths::ENV_FILE);
    let env_vars = env_loader::load_env(env_path)?;

    // ── 2. Load konfigurasi ───────────────────────────────────────────────────
    // Resolve ke absolute path agar working directory tidak masalah
    let config_path = if cli.config.is_absolute() {
        cli.config.clone()
    } else {
        std::env::current_dir()
            .map(|p| p.join(&cli.config))
            .unwrap_or_else(|_| cli.config.clone())
    };
    let config_mgr = Arc::new(ConfigManager::load(&config_path)?);
    let config = config_mgr.get()?;
    let config_arc = config_mgr.get_arc();

    // ── 3. Inisialisasi logger ────────────────────────────────────────────────
    let log_level = env_vars
        .log_level
        .as_deref()
        .unwrap_or(&config.logging.level)
        .to_string();
    let log_dir = PathBuf::from(&config.logging.path);
    let _log_guard = init_logger(&log_dir, &log_level)?;

    info!(
        version  = env!("CARGO_PKG_VERSION"),
        mode     = %config.app.mode,
        config   = %config_path.display(),
        "App Blocker v{} dimulai", env!("CARGO_PKG_VERSION")
    );

    // ── 4. Inisialisasi audit logger ──────────────────────────────────────────
    let reports_dir = PathBuf::from(&config.logging.path)
        .parent()
        .unwrap_or(Path::new("."))
        .join("reports");
    init_global_audit(&reports_dir).unwrap_or_else(|e| {
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

    // let _default_password = SecureString::try_from_str(DEFAULT_PASSWORD)?;

    // ── 7. Setup autentikasi ──────────────────────────────────────────────────
    // FIX: AuthManager dibungkus Arc<Mutex> dari awal, tidak ada try_unwrap
    let password_hash = if env_vars.admin_password_hash.trim().is_empty() {
        info!(
            "Hash belum ada, generate dari default password {}...",
            DEFAULT_PASSWORD
        );
        // Gunakan with_default_password() untuk generate hash baru
        let (_tmp_svc, hash) = Argon2AuthService::with_default_password()?;
        info!(
            "Hash password default berhasil di-generate: {}...",
            &hash[..30.min(hash.len())]
        );
        env_loader::write_password_hash(env_path, &hash)?;
        hash
    } else {
        env_vars.admin_password_hash.clone()
    };

    info!("Hash password admin berhasil dimuat {}", password_hash);

    let auth_svc = Argon2AuthService::new(password_hash)?;
    let auth_manager_shared: Arc<Mutex<AuthManager>> = Arc::new(Mutex::new(AuthManager::new(
        Box::new(auth_svc),
        config.security.max_auth_attempts,
        config.security.lockout_duration_seconds,
    )));

    // ── 8. Inisialisasi state manager ─────────────────────────────────────────
    let state_manager = Arc::new(StateManager::new());

    // ── 9. Metrics ────────────────────────────────────────────────────────────
    let _metrics = AppMetrics::new();

    // ── 10. Shutdown flag + Ctrl+C handler ───────────────────────────────────
    // FIX: ctrlc real implementation (bukan stub)
    let shutdown_flag = Arc::new(AtomicBool::new(false));
    {
        let sf = Arc::clone(&shutdown_flag);
        ctrlc::set_handler(move || {
            info!("Ctrl+C diterima, memulai shutdown graceful...");
            sf.store(true, Ordering::SeqCst);
        })
        .unwrap_or_else(|e| warn!(error = %e, "Gagal pasang Ctrl+C handler"));
    }

    // ── 11. Startup delay ────────────────────────────────────────────────────
    let delay = if config.simulation.enabled {
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
    let cfg_mgr_for_engine = Arc::clone(&config_mgr);

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

    engine.set_config_manager(cfg_mgr_for_engine);
    engine.set_student_mode(StudentModeConfig::default());

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

    // ── 19. FIX: Spawn Config file watcher thread ─────────────────────────────
    spawn_config_watcher(
        config_path.clone(),
        event_tx.clone(),
        Arc::clone(&shutdown_flag),
    )?;

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

    info!("App Blocker shutdown selesai.");
    Ok(())
}
