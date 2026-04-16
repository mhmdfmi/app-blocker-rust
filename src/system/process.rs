//! Process Management Module
//! 
//! Modul untuk manajemen proses dan terminate process dengan aman.

use crate::utils::error::{AppResult, AppError};
use crate::security::integrity::is_protected_process;
use sysinfo::{System, Pid, ProcessStatus};
use parking_lot::RwLock;
use std::sync::Arc;

/// Process information
#[derive(Debug, Clone)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    pub path: Option<String>,
    pub status: ProcessStatus,
}

/// Process manager untuk scanning dan termination
pub struct ProcessManager {
    blacklist: Arc<RwLock<Vec<String>>>,
    system: Arc<RwLock<System>>,
}

impl ProcessManager {
    /// Buat process manager baru
    pub fn new(blacklist: Vec<String>) -> Self {
        Self {
            blacklist: Arc::new(RwLock::new(blacklist)),
            system: Arc::new(RwLock::new(System::new_all())),
        }
    }
    
    /// Update system info
    pub fn refresh(&self) {
        self.system.write().refresh_all();
    }
    
    /// Scan untuk proses dalam blacklist
    pub fn scan_blacklist(&self) -> AppResult<Option<ProcessInfo>> {
        self.refresh();
        
        let blacklist = self.blacklist.read().clone();
        let sys = self.system.read();
        
        for (pid, process) in sys.processes() {
            let name = process.name().to_string_lossy().to_string();
            
            // Check against blacklist
            for blocked in &blacklist {
                if name.to_lowercase().contains(&blocked.to_lowercase()) {
                    return Ok(Some(ProcessInfo {
                        pid: pid.as_u32(),
                        name,
                        path: Some(process.exe().map(|p| p.to_string_lossy().to_string()).unwrap_or_default()),
                        status: process.status(),
                    }));
                }
            }
        }
        
        Ok(None)
    }
    
    /// Check apakah aman untuk membunuh proses
    pub fn is_safe_to_kill(&self, pid: u32) -> bool {
        let sys = self.system.read();
        
        if let Some(process) = sys.process(Pid::from_u32(pid)) {
            let name = process.name().to_string_lossy();
            
            // Check protected processes
            if is_protected_process(&name) {
                tracing::warn!("Attempted to kill protected process: {}", name);
                return false;
            }
            
            // Check parent process
            if let Some(parent) = process.parent() {
                if let Some(parent_process) = sys.process(parent) {
                    let parent_name = parent_process.name().to_string_lossy();
                    if is_protected_process(&parent_name) {
                        tracing::warn!("Process {} parent {} is protected", name, parent_name);
                        return false;
                    }
                }
            }
            
            true
        } else {
            false
        }
    }
    
    /// Terminate proses
    pub fn terminate(&self, pid: u32) -> AppResult<bool> {
        self.refresh();
        
        let sys = self.system.read();
        
        if let Some(process) = sys.process(Pid::from_u32(pid)) {
            let name = process.name().to_string_lossy();
            
            if is_protected_process(&name) {
                return Err(AppError::ProcessError(
                    format!("Cannot terminate protected process: {}", name)
                ));
            }
            
            let result = process.kill();
            
            if result {
                tracing::info!("Process {} (PID: {}) terminated successfully", name, pid);
            } else {
                tracing::error!("Failed to terminate process {} (PID: {})", name, pid);
            }
            
            Ok(result)
        } else {
            Ok(false) // Process already gone
        }
    }
    
    /// Get process oleh PID
    pub fn get_process(&self, pid: u32) -> Option<ProcessInfo> {
        let sys = self.system.read();
        
        sys.process(Pid::from_u32(pid)).map(|p| ProcessInfo {
            pid,
            name: p.name().to_string_lossy().to_string(),
            path: p.exe().map(|e| e.to_string_lossy().to_string()),
            status: p.status(),
        })
    }
    
    /// Get all processes
    pub fn get_all_processes(&self) -> Vec<ProcessInfo> {
        self.refresh();
        let sys = self.system.read();
        
        sys.processes()
            .iter()
            .map(|(pid, process)| ProcessInfo {
                pid: pid.as_u32(),
                name: process.name().to_string_lossy().to_string(),
                path: process.exe().map(|e| e.to_string_lossy().to_string()),
                status: process.status(),
            })
            .collect()
    }
    
    /// Update blacklist
    pub fn update_blacklist(&self, new_blacklist: Vec<String>) {
        *self.blacklist.write() = new_blacklist;
    }
}
