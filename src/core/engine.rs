/// Engine loop utama - menerima event dan mengelola state machine.
/// Semua mutasi state terjadi di sini, tidak di thread lain.
use crate::config::ConfigManager;
use crate::config::settings::AppConfig;
use crate::core::audit::{audit, AuditEntry, AuditEventKind};
use crate::core::events::{AppEvent, ComponentId};
use crate::core::state::{AppState, StateManager};
use crate::security::auth::{AuthManager, AuthStatus};
use crate::system::process::ProcessService;
use crate::system::student_mode::{apply_restrictions, restore_restrictions, StudentModeConfig};
use crate::utils::error::{AppError, AppResult};
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, Mutex, RwLock};
use std::time::Duration;
use tracing::{debug, error, info, warn};

/// Callback yang dipanggil engine untuk menampilkan overlay
pub type OverlayCallback = Box<dyn Fn(OverlayRequest) + Send + 'static>;

/// Data yang dikirim ke callback overlay
#[derive(Debug, Clone)]
pub struct OverlayRequest {
    pub pid: u32,
    pub process_name: String,
    pub username: String,
    pub computer_name: String,
    pub timestamp: String,
    pub trace_id: String,
    pub matched_game: Option<String>,
    pub score: u32,
}

/// Engine utama aplikasi
pub struct AppEngine {
    event_rx: Receiver<AppEvent>,
    event_tx: Sender<AppEvent>,
    state_manager: Arc<StateManager>,
    config: Arc<RwLock<AppConfig>>,
    config_manager: Option<Arc<ConfigManager>>,
    process_service: Arc<Mutex<Box<dyn ProcessService>>>,
    auth_manager: Arc<Mutex<AuthManager>>,
    student_mode: StudentModeConfig,
    overlay_callback: Option<OverlayCallback>,
    /// Waktu proses diblokir (untuk hitung session_duration)
    block_started_at: Option<std::time::Instant>,
}

impl AppEngine {
    /// Buat engine baru
    pub fn new(
        event_rx: Receiver<AppEvent>,
        event_tx: Sender<AppEvent>,
        state_manager: Arc<StateManager>,
        config: Arc<RwLock<AppConfig>>,
        process_service: Box<dyn ProcessService>,
        auth_manager: AuthManager,
    ) -> Self {
        Self {
            event_rx,
            event_tx,
            state_manager,
            config,
            config_manager: None,
            process_service: Arc::new(Mutex::new(process_service)),
            auth_manager: Arc::new(Mutex::new(auth_manager)),
            student_mode: StudentModeConfig::default(),
            overlay_callback: None,
            block_started_at: None,
        }
    }

    /// Set config manager untuk hot reload
    pub fn set_config_manager(&mut self, mgr: Arc<ConfigManager>) {
        self.config_manager = Some(mgr);
    }

    /// Set callback untuk menampilkan overlay
    pub fn set_overlay_callback(&mut self, cb: OverlayCallback) {
        self.overlay_callback = Some(cb);
    }

    /// Set konfigurasi student mode
    pub fn set_student_mode(&mut self, config: StudentModeConfig) {
        self.student_mode = config;
    }

    /// Jalankan loop engine utama
    pub fn run(mut self, shutdown_flag: Arc<std::sync::atomic::AtomicBool>) {
        info!("Engine loop dimulai");

        // Audit startup
        audit(AuditEntry::new(AuditEventKind::SystemStarted)
            .with_detail(format!("App Blocker v{} dimulai", env!("CARGO_PKG_VERSION"))));

        loop {
            if shutdown_flag.load(std::sync::atomic::Ordering::SeqCst) {
                info!("Engine: menerima sinyal shutdown");
                break;
            }

            match self.event_rx.recv_timeout(Duration::from_millis(500)) {
                Ok(event) => {
                    let event_name = event.name();
                    if let Err(e) = self.handle_event(event) {
                        // Shutdown diminta via error adalah normal
                        if matches!(e, AppError::System(ref s) if s.starts_with("Shutdown:")) {
                            info!("Engine: shutdown diminta");
                            break;
                        }
                        error!(error = %e, event = event_name, "Error handling event");
                        if let Err(e2) = self.state_manager.update_data(|d| {
                            d.consecutive_errors += 1;
                        }) {
                            error!(error = %e2, "Gagal update error counter");
                        }
                        let errors = self.state_manager
                            .read_data(|d| d.consecutive_errors)
                            .unwrap_or(0);
                        if errors >= 5 {
                            warn!("Terlalu banyak error berturut-turut, masuk SafeMode");
                            let _ = self.state_manager.force_safe_mode("consecutive_errors_threshold");
                        }
                    }
                }
                Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                    debug!("Engine: timeout menunggu event");
                }
                Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                    error!("Engine: channel terputus!");
                    let _ = self.state_manager.force_safe_mode("channel_disconnected");
                    break;
                }
            }
        }

        // Audit shutdown
        audit(AuditEntry::new(AuditEventKind::SystemStopped)
            .with_detail("Shutdown normal"));

        info!("Engine loop selesai");
    }

    /// Handle satu event
    fn handle_event(&mut self, event: AppEvent) -> AppResult<()> {
        debug!(event = event.name(), "Engine menerima event");

        match event {
            AppEvent::ProcessDetected { trace_id, info, score, detected_at: _ } => {
                self.handle_process_detected(trace_id, info, score, None)?;
            }

            AppEvent::UnlockSuccess { trace_id, username, unlocked_at: _ } => {
                self.handle_unlock_success(trace_id, &username)?;
            }

            AppEvent::UnlockFailed { trace_id, attempts, max_attempts } => {
                warn!(%trace_id, attempts, max_attempts, "Percobaan unlock gagal");
                audit(AuditEntry::new(AuditEventKind::AuthFailed)
                    .with_trace(&trace_id)
                    .with_detail(format!("Percobaan {attempts}/{max_attempts}")));
            }

            AppEvent::EmergencyUnlock { trace_id } => {
                info!(%trace_id, "Emergency unlock diterima");
                audit(AuditEntry::new(AuditEventKind::EmergencyUnlock)
                    .with_trace(&trace_id));
                self.handle_unlock_success(trace_id, "EMERGENCY")?;
            }

            AppEvent::ShutdownRequested { reason } => {
                info!(reason, "Shutdown diminta");
                return Err(AppError::System(format!("Shutdown: {reason}")));
            }

            AppEvent::EnterSafeMode { reason } => {
                audit(AuditEntry::new(AuditEventKind::SafeModeEntered)
                    .with_detail(&reason));
                self.state_manager.force_safe_mode(&reason)?;
            }

            AppEvent::DisableFlagDetected => {
                warn!("Flag disable darurat - masuk SafeMode");
                audit(AuditEntry::new(AuditEventKind::DisableFlagDetected));
                self.state_manager.force_safe_mode("disable_flag")?;
            }

            AppEvent::ConfigReloaded => {
                // Jalankan hot reload via config manager
                if let Some(mgr) = &self.config_manager {
                    match mgr.hot_reload() {
                        Ok(true) => {
                            audit(AuditEntry::new(AuditEventKind::ConfigReloaded));
                            // Update config arc dengan yang baru
                            info!("Config hot reload berhasil diterapkan di engine");
                        }
                        Ok(false) => debug!("Config tidak berubah"),
                        Err(e) => warn!(error = %e, "Hot reload gagal, config lama dipertahankan"),
                    }
                }
            }

            AppEvent::ThreadDied { component, reason } => {
                error!(component = %component, reason, "Thread mati terdeteksi");
            }

            AppEvent::Heartbeat { .. }
            | AppEvent::ProcessBlocked { .. }
            | AppEvent::ProcessBlockFailed { .. }
            | AppEvent::OverlayRequested { .. } => {}
        }

        Ok(())
    }

    /// Handle deteksi proses terlarang
    fn handle_process_detected(
        &mut self,
        trace_id: uuid::Uuid,
        info: crate::system::process::ProcessInfo,
        score: u32,
        matched_game: Option<String>,
    ) -> AppResult<()> {
        let current = self.state_manager.current_state()?;
        if !current.allows_blocking() {
            debug!(state = %current, "Skip - tidak dalam state Monitoring");
            return Ok(());
        }

        audit(AuditEntry::new(AuditEventKind::ProcessDetected)
            .with_trace(&trace_id)
            .with_process(info.pid, &info.name)
            .with_score(score)
            .with_detection_method(matched_game.as_deref().unwrap_or("behavior"))
            .with_schedule(true));

        self.state_manager.transition_to(AppState::Blocking, "process_detected")?;
        self.state_manager.update_data(|d| {
            d.blocked_pid = Some(info.pid);
            d.blocked_name = Some(info.name.clone());
            d.is_safe_to_block = true;
            d.consecutive_errors = 0;
        })?;

        self.block_started_at = Some(std::time::Instant::now());

        // Apply student mode saat akan lock
        if self.student_mode.apply_only_when_locked {
            if let Err(e) = apply_restrictions(&self.student_mode) {
                warn!(error = %e, "Gagal terapkan student mode (non-fatal)");
            }
        }

        let kill_result = {
            let mut svc = self.process_service
                .lock()
                .map_err(|e| AppError::System(format!("Lock process service: {e}")))?;
            svc.kill_process(info.pid, &info.name)
        };

        match kill_result {
            Ok(()) => {
                audit(AuditEntry::new(AuditEventKind::ProcessKilled)
                    .with_trace(&trace_id)
                    .with_process(info.pid, &info.name)
                    .with_user(
                        &info.username.clone().unwrap_or_default(),
                        &crate::system::user::get_computer_name(),
                    ));

                info!(%trace_id, pid = info.pid, name = %info.name, "Proses berhasil dihentikan");

                let _ = self.event_tx.send(AppEvent::ProcessBlocked {
                    trace_id,
                    pid: info.pid,
                    name: info.name.clone(),
                    killed_at: crate::utils::time::now_utc(),
                });

                self.state_manager.transition_to(AppState::Locked, "process_blocked")?;
                self.state_manager.update_data(|d| { d.overlay_active = true; })?;

                audit(AuditEntry::new(AuditEventKind::OverlayShown)
                    .with_trace(&trace_id)
                    .with_process(info.pid, &info.name));

                self.trigger_overlay(trace_id, &info, score, matched_game)?;
            }
            Err(e) => {
                audit(AuditEntry::new(AuditEventKind::ProcessKillFailed)
                    .with_trace(&trace_id)
                    .with_process(info.pid, &info.name)
                    .with_detail(e.to_string()));

                error!(%trace_id, pid = info.pid, name = %info.name, error = %e, "Gagal kill proses");

                let _ = self.event_tx.send(AppEvent::ProcessBlockFailed {
                    trace_id,
                    pid: info.pid,
                    name: info.name.clone(),
                    reason: e.to_string(),
                });

                // Restore student mode jika kill gagal
                let _ = restore_restrictions(&self.student_mode);

                self.state_manager.transition_to(AppState::Recovering, "kill_failed")?;
                self.state_manager.reset_data()?;
                self.state_manager.transition_to(AppState::Monitoring, "recovery_after_kill_fail")?;
            }
        }

        Ok(())
    }

    /// Handle unlock berhasil
    fn handle_unlock_success(&mut self, trace_id: uuid::Uuid, username: &str) -> AppResult<()> {
        info!(%trace_id, username, "Unlock berhasil, memulai recovery");

        let session_duration = self.block_started_at
            .map(|t| t.elapsed().as_secs_f64())
            .unwrap_or(0.0);
        self.block_started_at = None;

        audit(AuditEntry::new(AuditEventKind::AuthSuccess)
            .with_trace(&trace_id)
            .with_user(username, &crate::system::user::get_computer_name())
            .with_duration(session_duration));

        audit(AuditEntry::new(AuditEventKind::OverlayDismissed)
            .with_trace(&trace_id)
            .with_duration(session_duration));

        self.state_manager.transition_to(AppState::Recovering, "unlock_success")?;
        self.state_manager.reset_data()?;

        // Restore student mode setelah unlock
        if let Err(e) = restore_restrictions(&self.student_mode) {
            warn!(error = %e, "Gagal restore student mode (non-fatal)");
        }

        if let Ok(mut auth) = self.auth_manager.lock() {
            auth.reset_attempts();
        }

        self.state_manager.transition_to(AppState::Monitoring, "recovery_complete")?;
        info!("Sistem kembali ke mode Monitoring");
        Ok(())
    }

    /// Trigger tampilan overlay melalui callback
    fn trigger_overlay(
        &self,
        trace_id: uuid::Uuid,
        info: &crate::system::process::ProcessInfo,
        score: u32,
        matched_game: Option<String>,
    ) -> AppResult<()> {
        let request = OverlayRequest {
            pid: info.pid,
            process_name: info.name.clone(),
            username: info.username.clone().unwrap_or_else(|| "UNKNOWN".to_string()),
            computer_name: crate::system::user::get_computer_name(),
            timestamp: crate::utils::time::format_datetime(&crate::utils::time::now_utc()),
            trace_id: trace_id.to_string(),
            matched_game,
            score,
        };

        if let Some(cb) = &self.overlay_callback {
            cb(request);
            Ok(())
        } else {
            warn!("Tidak ada overlay callback terdaftar!");
            Err(AppError::Overlay("Overlay callback tidak ada".to_string()))
        }
    }

    pub fn auth_manager_arc(&self) -> Arc<Mutex<AuthManager>> {
        Arc::clone(&self.auth_manager)
    }

    pub fn event_tx(&self) -> Sender<AppEvent> {
        self.event_tx.clone()
    }
}
