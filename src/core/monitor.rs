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
    }
}
