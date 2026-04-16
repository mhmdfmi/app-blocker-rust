//! Engine Module
//! 
//! Engine utama yang mengoordinasikan semua komponen.
//! Mengatur flow data antara monitor thread (TX) dan UI thread (RX).

use crate::core::events::{AppEvent, ProcessInfo};
use crate::core::state::{SharedState, State, AppState};
use crate::core::monitor::MonitorThread;
use crate::config::settings::Settings;
use crate::config::env_loader::load_env;
use crate::utils::error::{AppResult, AppError};
use parking_lot::RwLock;
use std::sync::mpsc::{self, Sender, Receiver};
use std::thread;
use std::time::Duration;
use uuid::Uuid;

/// Engine utama aplikasi
pub struct Engine {
    state: SharedState,
    settings: Settings,
    monitor_thread: Option<MonitorThread>,
    event_sender: Option<Sender<AppEvent>>,
}

impl Engine {
    /// Buat engine baru
    pub fn new(state: SharedState) -> Self {
        // Load environment
        let _ = load_env();
        
        // Load settings
        let settings = Settings::default();
        
        Self {
            state,
            settings,
            monitor_thread: None,
            event_sender: None,
        }
    }
    
    /// Jalankan engine
    pub fn run(&mut self) -> AppResult<()> {
        tracing::info!("Engine starting...");
        
        // Validasi startup
        self.validate_startup()?;
        
        // Buat channel untuk komunikasi antar thread
        let (tx, _rx) = mpsc::channel::<AppEvent>();
        self.event_sender = Some(tx.clone());
        
        // Update state ke Monitoring
        {
            let mut state = self.state.write();
            state.current_state = State::Monitoring;
        }
        
        // Validasi single instance
        self.check_single_instance()?;
        
        // Apply startup delay
        self.apply_startup_delay()?;
        
        // Mulai monitor thread
        self.monitor_thread = Some(MonitorThread::start(
            self.state.clone(),
            tx,
            self.settings.clone(),
        ));
        
        // Jalankan UI thread (event receiver)
        self.run_ui_thread(_rx)?;
        
        Ok(())
    }
    
    /// Validasi startup
    fn validate_startup(&self) -> AppResult<()> {
        // Load dan validasi konfigurasi
        self.settings.validate()?;
        tracing::info!("Startup validation passed");
        Ok(())
    }
    
    /// Validasi single instance
    fn check_single_instance(&self) -> AppResult<()> {
        use std::fs::File;
        use std::io::Write;
        
        let lock_path = "C:\\AppBlocker\\app.lock";
        
        // Buat directory jika belum ada
        if let Some(parent) = std::path::Path::new(lock_path).parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        
        match File::create(lock_path) {
            Ok(mut file) => {
                let pid = std::process::id();
                let _ = file.write_all(format!("{}", pid).as_bytes());
                tracing::info!("Single instance lock acquired (PID: {})", pid);
                Ok(())
            }
            Err(_) => {
                tracing::error!("Another instance is already running");
                Err(AppError::ConfigError("App is already running".into()))
            }
        }
    }
    
    /// Apply startup delay
    fn apply_startup_delay(&self) -> AppResult<()> {
        let delay_ms = self.settings.startup_delay_ms;
        tracing::info!("Applying startup delay: {}ms", delay_ms);
        thread::sleep(Duration::from_millis(delay_ms));
        Ok(())
    }
    
    /// Jalankan UI thread (receiver)
    fn run_ui_thread(&mut self, _rx: Receiver<AppEvent>) -> AppResult<()> {
        tracing::info!("UI thread starting...");
        
        loop {
            // Terima event
            let event = match _rx.recv_timeout(Duration::from_millis(500)) {
                Ok(e) => e,
                Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                    // Check apakah harus shutdown
                    let state = self.state.read();
                    if state.current_state == State::SafeMode {
                        break;
                    }
                    continue;
                }
                Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                    tracing::error!("Channel disconnected");
                    self.handle_channel_disconnect()?;
                    break;
                }
            };
            
            // Proses event
            self.handle_event(event)?;
        }
        
        tracing::info!("UI thread stopped");
        Ok(())
    }
    
    /// Handle event yang diterima
    fn handle_event(&mut self, event: AppEvent) -> AppResult<()> {
        let trace_id = event.trace_id().map(String::from).unwrap_or_else(|| Uuid::new_v4().to_string());
        
        tracing::debug!("Handling event: {:?} (trace: {})", 
            std::mem::discriminant(&event), trace_id);
        
        match event {
            AppEvent::ProcessDetected(info) => {
                self.handle_process_detected(info, &trace_id)
            }
            AppEvent::ProcessBlocked(info) => {
                self.handle_process_blocked(info, &trace_id)
            }
            AppEvent::OverlayRequested(request) => {
                self.handle_overlay_requested(request, &trace_id)
            }
            AppEvent::UnlockSuccess { username, trace_id } => {
                self.handle_unlock_success(username, &trace_id)
            }
            AppEvent::UnlockFailed { attempt, reason, trace_id } => {
                self.handle_unlock_failed(attempt, reason, &trace_id)
            }
            AppEvent::ShutdownRequested => {
                self.handle_shutdown()
            }
            AppEvent::EmergencyUnlock => {
                self.handle_emergency_unlock()
            }
            AppEvent::Error(err) => {
                self.handle_error(err)
            }
            _ => Ok(())
        }
    }
    
    /// Handle proses terdeteksi
    fn handle_process_detected(&mut self, info: ProcessInfo, trace_id: &str) -> AppResult<()> {
        // Transition ke Blocking
        {
            let mut state = self.state.write();
            state.current_state = State::Blocking;
            state.trace_id = Some(trace_id.to_string());
            state.last_blocked_pid = Some(info.pid);
            state.last_blocked_name = Some(info.name.clone());
        }
        
        // Terminate proses
        self.terminate_blocked_process(info.pid, &info.name, trace_id)
    }
    
    /// Terminasi proses yang diblokir
    fn terminate_blocked_process(&mut self, pid: u32, name: &str, trace_id: &str) -> AppResult<()> {
        tracing::info!("Attempting to terminate process: {} (PID: {})", name, pid);
        
        // Validasi safe to kill
        if self.settings.simulation_mode && self.settings.simulate_process_kill {
            tracing::info!("[SIMULATED] Process {} would be terminated", name);
            
            // Kirim event ProcessBlocked
            if let Some(sender) = &self.event_sender {
                let _ = sender.send(AppEvent::ProcessBlocked(ProcessInfo {
                    pid,
                    name: name.to_string(),
                    path: None,
                    username: None,
                    timestamp: chrono::Utc::now().to_rfc3339(),
                    suspicious_score: 100,
                }));
            }
            
            return Ok(());
        }
        
        // Transition ke Locked
        {
            let mut state = self.state.write();
            state.current_state = State::Locked;
            state.is_locked = true;
            state.is_safe_to_block = true;
        }
        
        // Request overlay
        if let Some(sender) = &self.event_sender {
            let _ = sender.send(AppEvent::OverlayRequested(
                crate::core::events::OverlayRequest {
                    process_info: ProcessInfo {
                        pid,
                        name: name.to_string(),
                        path: None,
                        username: None,
                        timestamp: chrono::Utc::now().to_rfc3339(),
                        suspicious_score: 100,
                    },
                    trace_id: trace_id.to_string(),
                    is_simulation: self.settings.simulation_mode,
                }
            ));
        }
        
        Ok(())
    }
    
    /// Handle proses diblokir
    fn handle_process_blocked(&mut self, info: ProcessInfo, trace_id: &str) -> AppResult<()> {
        // Transition ke Locked
        {
            let mut state = self.state.write();
            state.current_state = State::Locked;
            state.is_locked = true;
            state.overlay_active = true;
        }
        
        // Request overlay
        if let Some(sender) = &self.event_sender {
            let _ = sender.send(AppEvent::OverlayRequested(
                crate::core::events::OverlayRequest {
                    process_info: info,
                    trace_id: trace_id.to_string(),
                    is_simulation: self.settings.simulation_mode,
                }
            ));
        }
        
        Ok(())
    }
    
    /// Handle overlay request
    fn handle_overlay_requested(&mut self, request: crate::core::events::OverlayRequest, trace_id: &str) -> AppResult<()> {
        tracing::info!("Overlay requested for process: {} (trace: {})", 
            request.process_info.name, trace_id);
        
        // Di simulation mode, tetap show overlay tapi tidak blocker
        if self.settings.simulation_mode && self.settings.simulate_overlay {
            tracing::info!("[SIMULATED] Overlay would be displayed");
            // Dalam simulation, langsung recovery
            self.handle_recovery()?;
            return Ok(());
        }
        
        // Show overlay through UI
        // UI akan block sampei unlock
        
        Ok(())
    }
    
    /// Handle unlock success
    fn handle_unlock_success(&mut self, username: String, trace_id: &str) -> AppResult<()> {
        tracing::info!("Unlock successful by {} (trace: {})", username, trace_id);
        
        // Reset failed attempts
        {
            let mut state = self.state.write();
            state.failed_unlock_attempts = 0;
        }
        
        // Transition ke Recovering
        self.handle_recovery()
    }
    
    /// Handle unlock failed
    fn handle_unlock_failed(&mut self, attempt: u32, reason: String, trace_id: &str) -> AppResult<()> {
        tracing::warn!("Unlock attempt {} failed: {} (trace: {})", attempt, reason, trace_id);
        
        // Increment failed attempts
        {
            let mut state = self.state.write();
            state.failed_unlock_attempts += 1;
            
            // Check max attempts
            if state.failed_unlock_attempts >= self.settings.max_auth_attempts {
                tracing::error!("Max unlock attempts reached, entering SafeMode");
                let mut s = state;
                s.enter_safe_mode();
            }
        }
        
        Ok(())
    }
    
    /// Handle emergency unlock
    fn handle_emergency_unlock(&mut self) -> AppResult<()> {
        tracing::info!("Emergency unlock triggered");
        self.handle_recovery()
    }
    
    /// Handle recovery
    fn handle_recovery(&mut self) -> AppResult<()> {
        // Transition ke Recovering
        {
            let mut state = self.state.write();
            state.current_state = State::Recovering;
            state.cleanup_in_progress = true;
        }
        
        // Cleanup
        self.cleanup()?;
        
        // Transition ke Monitoring
        {
            let mut state = self.state.write();
            state.reset();
            state.current_state = State::Monitoring;
        }
        
        tracing::info!("Recovery complete, back to Monitoring");
        
        // Kirim event recovery complete
        if let Some(sender) = &self.event_sender {
            let _ = sender.send(AppEvent::RecoveryComplete);
        }
        
        Ok(())
    }
    
    /// Cleanup resources
    fn cleanup(&self) -> AppResult<()> {
        // Reset state flags
        {
            let mut state = self.state.write();
            state.cleanup_in_progress = false;
            state.overlay_active = false;
        }
        
        tracing::info!("Cleanup completed");
        Ok(())
    }
    
    /// Handle error
    fn handle_error(&mut self, error: crate::core::events::ErrorEvent) -> AppResult<()> {
        tracing::error!("Error: {:?} - {}", error.error_type, error.message);
        
        match error.error_type {
            crate::core::events::ErrorType::DeadlockDetected => {
                self.enter_safe_mode()?;
            }
            crate::core::events::ErrorType::ChannelDisconnected => {
                self.handle_channel_disconnect()?;
            }
            _ => {}
        }
        
        Ok(())
    }
    
    /// Handle channel disconnect
    fn handle_channel_disconnect(&mut self) -> AppResult<()> {
        tracing::error!("Channel disconnected, attempting recovery");
        
        // Recreate channel
        let (tx, _rx) = mpsc::channel();
        self.event_sender = Some(tx.clone());
        
        // Restart monitor thread
        self.monitor_thread = Some(MonitorThread::start(
            self.state.clone(),
            tx,
            self.settings.clone(),
        ));
        
        // Enter safe mode jika gagal
        self.enter_safe_mode()?;
        
        Ok(())
    }
    
    /// Masuk ke safe mode
    fn enter_safe_mode(&mut self) -> AppResult<()> {
        tracing::warn!("Entering SafeMode...");
        
        {
            let mut state = self.state.write();
            state.enter_safe_mode();
        }
        
        Ok(())
    }
    
    /// Handle shutdown
    fn handle_shutdown(&mut self) -> AppResult<()> {
        tracing::info!("Shutdown requested");
        
        // Stop monitor thread
        if let Some(mut monitor) = self.monitor_thread.take() {
            let _ = monitor.join();
        }
        
        // Reset state
        {
            let mut state = self.state.write();
            state.reset();
        }
        
        // Release lock file
        let _ = std::fs::remove_file("C:\\AppBlocker\\app.lock");
        
        tracing::info!("Shutdown complete");
        Ok(())
    }
    
    /// Shutdown engine
    pub fn shutdown(&mut self) {
        tracing::info!("Engine shutdown initiated");
        
        // Stop threads
        if let Some(mut monitor) = self.monitor_thread.take() {
            let _ = monitor.join();
        }
        
        // Release lock
        let _ = std::fs::remove_file("C:\\AppBlocker\\app.lock");
    }
}
