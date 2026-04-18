<<<<<<< HEAD
/// Layanan kontrol proses Windows dengan safe kill logic.
/// TIDAK PERNAH membunuh proses sistem yang dilindungi.
use crate::constants::paths::PROTECTED_PROCESSES;
use crate::utils::error::{AppError, AppResult};
use crate::utils::retry::{with_retry, RetryConfig};
use std::time::Duration;
use sysinfo::{Pid, Process, ProcessStatus, System};
use tracing::{debug, error, info, warn};

/// Informasi proses yang terdeteksi
=======
﻿//! Process Management Module
//! 
//! Modul untuk manajemen proses dan terminate process dengan aman.

use crate::utils::error::{AppResult, AppError};
use crate::security::integrity::is_protected_process;
use sysinfo::{System, Pid, ProcessStatus};
use parking_lot::RwLock;
use std::sync::Arc;

/// Process information
>>>>>>> bce0345919f371d153ccb843f2ddbfb5e8695c5f
#[derive(Debug, Clone)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
<<<<<<< HEAD
    pub exe_path: Option<String>,
    pub username: Option<String>,
    pub cpu_usage: f32,
    pub status: String,
}

impl ProcessInfo {
    /// Buat ProcessInfo dari sysinfo::Process
    pub fn from_sysinfo(pid: u32, proc: &Process) -> Self {
        Self {
            pid,
            name: proc.name().to_string_lossy().to_string(),
            exe_path: proc.exe().map(|p| p.display().to_string()),
            username: proc.user_id().map(|u| u.to_string()),
            cpu_usage: proc.cpu_usage(),
            status: format!("{:?}", proc.status()),
        }
    }
}

/// Trait abstraksi untuk layanan proses (memungkinkan mock pada testing)
pub trait ProcessService: Send + Sync {
    /// Ambil daftar semua proses yang berjalan
    fn list_processes(&mut self) -> AppResult<Vec<ProcessInfo>>;
    /// Hentikan proses berdasarkan PID
    fn kill_process(&self, pid: u32, process_name: &str) -> AppResult<()>;
    /// Periksa apakah proses masih berjalan
    fn is_running(&mut self, pid: u32) -> bool;
    /// Periksa apakah nama proses dilindungi
    fn is_protected(&self, name: &str) -> bool;
}

/// Implementasi ProcessService menggunakan sysinfo dan Win32
pub struct WindowsProcessService {
    system: System,
    simulation_mode: bool,
}

impl WindowsProcessService {
    /// Buat service baru
    pub fn new(simulation_mode: bool) -> Self {
        let mut system = System::new_all();
        system.refresh_all();
        Self {
            system,
            simulation_mode,
        }
    }

    /// Validasi bahwa proses aman untuk dihentikan
    fn validate_safe_to_kill(&self, pid: u32, name: &str) -> AppResult<()> {
        // Cek daftar protected processes
        if self.is_protected(name) {
            return Err(AppError::ProtectedProcess(format!(
                "Proses '{name}' (PID:{pid}) dilindungi dan tidak dapat dihentikan"
            )));
        }

        // Cek PID system-level (PID 0 dan 4 adalah Windows System)
        if pid == 0 || pid == 4 {
            return Err(AppError::ProtectedProcess(format!(
                "PID {pid} adalah proses kernel Windows"
            )));
        }

        Ok(())
    }

    /// Tunggu sampai proses benar-benar berhenti (polling)
    fn wait_for_termination(&mut self, pid: u32, timeout: Duration) -> bool {
        let start = std::time::Instant::now();
        while start.elapsed() < timeout {
            self.system.refresh_process(Pid::from_u32(pid));
            if self.system.process(Pid::from_u32(pid)).is_none() {
                return true;
            }
            std::thread::sleep(Duration::from_millis(100));
        }
        false
    }
}

impl ProcessService for WindowsProcessService {
    fn list_processes(&mut self) -> AppResult<Vec<ProcessInfo>> {
        self.system.refresh_processes();
        let processes: Vec<ProcessInfo> = self
            .system
            .processes()
            .iter()
            .map(|(pid, proc)| ProcessInfo::from_sysinfo(pid.as_u32(), proc))
            .collect();
        debug!(count = processes.len(), "Daftar proses diperbarui");
        Ok(processes)
    }

    fn kill_process(&self, pid: u32, process_name: &str) -> AppResult<()> {
        // Validasi keamanan sebelum kill
        self.validate_safe_to_kill(pid, process_name)?;

        if self.simulation_mode {
            info!(
                pid,
                name = process_name,
                "[SIMULASI] Proses akan dihentikan (tidak benar-benar dimatikan)"
            );
            return Ok(());
        }

        // Eksekusi kill dengan retry
        let config = RetryConfig::for_process_kill();
        with_retry(&config, &format!("kill_process:{process_name}"), || {
            kill_process_win32(pid, process_name)
        })
    }

    fn is_running(&mut self, pid: u32) -> bool {
        self.system.refresh_process(Pid::from_u32(pid));
        self.system.process(Pid::from_u32(pid)).is_some()
    }

    fn is_protected(&self, name: &str) -> bool {
        let name_lower = name.to_lowercase();
        PROTECTED_PROCESSES
            .iter()
            .any(|p| p.to_lowercase() == name_lower)
    }
}

/// Kill proses menggunakan Win32 API (Windows-only)
#[cfg(target_os = "windows")]
fn kill_process_win32(pid: u32, process_name: &str) -> AppResult<()> {
    use windows::Win32::Foundation::{CloseHandle, HANDLE};
    use windows::Win32::System::Threading::{
        OpenProcess, TerminateProcess, PROCESS_TERMINATE,
    };

    info!(pid, name = process_name, "Menghentikan proses...");

    let handle = unsafe {
        OpenProcess(PROCESS_TERMINATE, false, pid)
            .map_err(|e| AppError::Win32(format!("OpenProcess gagal untuk PID {pid}: {e}")))?
    };

    // Pastikan handle valid dan di-close setelah selesai
    struct HandleGuard(HANDLE);
    impl Drop for HandleGuard {
        fn drop(&mut self) {
            if !self.0.is_invalid() {
                unsafe { let _ = CloseHandle(self.0); }
            }
        }
    }
    let _guard = HandleGuard(handle);

    unsafe {
        TerminateProcess(handle, 1)
            .map_err(|e| AppError::Win32(format!("TerminateProcess gagal untuk PID {pid}: {e}")))?;
    }

    info!(pid, name = process_name, "Proses berhasil dihentikan");
    Ok(())
}

/// Fallback non-Windows: gunakan taskkill atau simulasi
#[cfg(not(target_os = "windows"))]
fn kill_process_win32(pid: u32, process_name: &str) -> AppResult<()> {
    warn!(pid, name = process_name, "[Non-Windows] Kill tidak tersedia, simulasi");
    Ok(())
}

/// Ambil nama komputer (hostname)
pub fn get_computer_name() -> String {
    hostname::get()
        .map(|h| h.to_string_lossy().to_string())
        .unwrap_or_else(|_| "UNKNOWN-PC".to_string())
}

/// Ambil nama pengguna yang sedang login
pub fn get_current_username() -> String {
    std::env::var("USERNAME")
        .or_else(|_| std::env::var("USER"))
        .unwrap_or_else(|_| "UNKNOWN".to_string())
}
=======
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
>>>>>>> bce0345919f371d153ccb843f2ddbfb5e8695c5f
