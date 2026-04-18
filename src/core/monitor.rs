<<<<<<< HEAD
/// Thread monitor - scan proses secara periodik dan kirim event ke engine.
/// Thread ini HANYA mengirim event, tidak mengubah state.
use crate::config::settings::AppConfig;
use crate::core::events::{AppEvent, ComponentId};
use crate::core::state::{AppState, StateManager};
use crate::detection::DetectionEngine;
use crate::system::process::{ProcessInfo, ProcessService};
use crate::system::service::is_disable_flag_active;
use crate::utils::error::{AppError, AppResult};
use std::sync::mpsc::Sender;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};
use tracing::{debug, error, info, warn};

/// Thread monitor yang scan proses dan kirim event
pub struct MonitorThread {
    event_tx: Sender<AppEvent>,
    state_manager: Arc<StateManager>,
    config: Arc<RwLock<AppConfig>>,
    process_service: Box<dyn ProcessService>,
    detection_engine: DetectionEngine,
}

impl MonitorThread {
    /// Buat monitor thread baru
    pub fn new(
        event_tx: Sender<AppEvent>,
        state_manager: Arc<StateManager>,
        config: Arc<RwLock<AppConfig>>,
        process_service: Box<dyn ProcessService>,
    ) -> AppResult<Self> {
        let cfg = config
            .read()
            .map_err(|e| AppError::Config(format!("Baca config: {e}")))?
            .clone();

        let detection_engine = DetectionEngine::new(&cfg)?;

        Ok(Self {
            event_tx,
            state_manager,
            config,
            process_service,
            detection_engine,
        })
    }

    /// Jalankan loop monitoring utama (blocking)
    pub fn run(mut self, shutdown_flag: Arc<std::sync::atomic::AtomicBool>) {
        info!("Monitor thread dimulai");
        let mut kill_rate = KillRateCounter::new(3, Duration::from_secs(60));

        loop {
            // Kirim heartbeat
            let _ = self.event_tx.send(AppEvent::Heartbeat {
                component: ComponentId::Monitor,
            });

            if shutdown_flag.load(std::sync::atomic::Ordering::SeqCst) {
                info!("Monitor thread: shutdown");
                break;
            }

            // Flag disable darurat
            if is_disable_flag_active() {
                warn!("Disable flag aktif - monitor berhenti blokir");
                let _ = self.event_tx.send(AppEvent::DisableFlagDetected);
                self.sleep_interval();
                continue;
            }

            // Hanya scan jika dalam state Monitoring
            let state = match self.state_manager.current_state() {
                Ok(s)  => s,
                Err(e) => {
                    error!(error = %e, "Gagal baca state");
                    self.sleep_interval();
                    continue;
                }
            };

            if !state.allows_blocking() {
                debug!(state = %state, "Monitor: menunggu state Monitoring");
                self.sleep_interval();
                continue;
            }

            // Rate limit check
            if kill_rate.is_rate_limited() {
                debug!("Rate limit aktif, skip siklus ini");
                self.sleep_interval();
                continue;
            }

            // Scan proses
            let processes = match self.process_service.list_processes() {
                Ok(p)  => p,
                Err(e) => {
                    error!(error = %e, "Gagal list proses");
                    self.sleep_interval();
                    continue;
                }
            };

            // Deteksi proses terlarang
            for proc in &processes {
                match self.detection_engine.detect(proc) {
                    Ok(Some(result)) => {
                        let trace_id = AppEvent::new_trace_id();
                        info!(
                            %trace_id,
                            pid   = proc.pid,
                            name  = %proc.name,
                            score = result.score,
                            game  = ?result.matched_game,
                            "Proses terlarang terdeteksi"
                        );

                        if self.event_tx.send(AppEvent::ProcessDetected {
                            trace_id,
                            info: proc.clone(),
                            score: result.score,
                            detected_at: crate::utils::time::now_utc(),
                        }).is_err() {
                            error!("Channel terputus di monitor");
                            return;
                        }

                        kill_rate.record_detection();
                        break; // Satu deteksi per siklus
                    }
                    Ok(None) => {}
                    Err(e)   => error!(error = %e, pid = proc.pid, "Error deteksi"),
                }
            }

            self.sleep_interval();
        }

        info!("Monitor thread selesai");
    }

    fn sleep_interval(&self) {
        let ms = self.config
            .read()
            .map(|c| c.monitoring.scan_interval_ms)
            .unwrap_or(2000);
        std::thread::sleep(Duration::from_millis(ms));
    }
}

/// Rate limiter untuk kill proses
struct KillRateCounter {
    detections: Vec<Instant>,
    window: Duration,
    max_per_window: u32,
}

impl KillRateCounter {
    fn new(max: u32, window: Duration) -> Self {
        Self { detections: Vec::new(), window, max_per_window: max }
    }

    fn record_detection(&mut self) {
        self.detections.push(Instant::now());
        let w = self.window;
        self.detections.retain(|t| t.elapsed() < w);
    }

    fn is_rate_limited(&mut self) -> bool {
        let w = self.window;
        self.detections.retain(|t| t.elapsed() < w);
        self.detections.len() as u32 >= self.max_per_window
=======
//! Monitor Module
use crate::core::events::{AppEvent, ProcessInfo};
use crate::core::state::{SharedState, State};
use crate::system::process::ProcessManager;
use crate::config::settings::Settings;
use crate::utils::error::AppResult;
use chrono::Local;
use std::sync::mpsc::Sender;
use std::time::Duration;
use std::thread;

pub struct Monitor {
    state: SharedState,
    sender: Sender<AppEvent>,
    settings: Settings,
    process_manager: ProcessManager,
}

impl Monitor {
    pub fn new(state: SharedState, sender: Sender<AppEvent>, settings: Settings) -> Self {
        Self {
            state,
            sender,
            settings: settings.clone(),
            process_manager: ProcessManager::new(settings.blacklist.clone()),
        }
    }
    
    pub fn run(&self) -> AppResult<()> {
        tracing::info!("Monitor thread started");
        
        loop {
            let state_guard = self.state.read();
            if state_guard.current_state == State::SafeMode {
                drop(state_guard);
                tracing::info!("Monitor: Entered SafeMode, stopping");
                break;
            }
            drop(state_guard);
            
            self.scan_processes()?;
            thread::sleep(Duration::from_millis(self.settings.monitoring_interval_ms));
        }
        
        tracing::info!("Monitor thread stopped");
        Ok(())
    }
    
    fn scan_processes(&self) -> AppResult<()> {
        let detected = self.process_manager.scan_blacklist()?;
        
        if let Some(process) = detected {
            tracing::warn!("Blacklisted process detected: {} (PID: {})", process.name, process.pid);
            
            let event = AppEvent::ProcessDetected(ProcessInfo {
                pid: process.pid,
                name: process.name.clone(),
                path: process.path.clone(),
                username: None,
                timestamp: Local::now().format("%Y-%m-%dT%H:%M:%S").to_string(),
                suspicious_score: 100,
            });
            
            if let Err(e) = self.sender.send(event) {
                tracing::error!("Failed to send event: {}", e);
            }
        }
        
        Ok(())
    }
    
    pub fn can_kill_process(&self, pid: u32) -> bool {
        self.process_manager.is_safe_to_kill(pid)
    }
    
    pub fn terminate_process(&self, pid: u32) -> AppResult<bool> {
        if !self.can_kill_process(pid) {
            tracing::warn!("Process {} is not safe to kill", pid);
            return Ok(false);
        }
        
        if self.settings.simulation_mode {
            tracing::info!("[SIMULATED] Process {} would be terminated", pid);
            return Ok(true);
        }
        
        self.process_manager.terminate(pid)
    }
}

pub struct MonitorThread {
    handle: Option<thread::JoinHandle<AppResult<()>>>,
}

impl MonitorThread {
    pub fn start(state: SharedState, sender: Sender<AppEvent>, settings: Settings) -> Self {
        let monitor = Monitor::new(state, sender, settings);
        let handle = thread::spawn(move || monitor.run());
        Self { handle: Some(handle) }
    }
    
    pub fn join(&mut self) -> AppResult<()> {
        if let Some(handle) = self.handle.take() {
            handle.join().map_err(|_| crate::utils::error::AppError::ThreadError("Monitor thread panicked".into()))?;
        }
        Ok(())
>>>>>>> bce0345919f371d153ccb843f2ddbfb5e8695c5f
    }
}
