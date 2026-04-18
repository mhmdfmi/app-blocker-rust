<<<<<<< HEAD
/// Manajemen layanan Windows dan single-instance lock.
use crate::constants::paths::{DISABLE_FLAG_FILE, LOCK_FILE};
use crate::utils::error::{AppError, AppResult};
use std::path::Path;
use tracing::{info, warn};

/// Guard single-instance yang hapus lock file saat drop
pub struct SingleInstanceGuard {
    lock_path: std::path::PathBuf,
}

impl SingleInstanceGuard {
    fn new(lock_path: std::path::PathBuf) -> Self {
        Self { lock_path }
    }
}

impl Drop for SingleInstanceGuard {
    fn drop(&mut self) {
        if self.lock_path.exists() {
            if let Err(e) = std::fs::remove_file(&self.lock_path) {
                warn!(error = %e, path = %self.lock_path.display(), "Gagal hapus lock file");
            } else {
                info!("Lock file dilepas");
            }
=======
﻿//! Service Module
//! 
//! Modul untuk Windows service management.

use crate::utils::error::{AppResult, AppError};
use std::path::PathBuf;

/// Service configuration
#[derive(Debug, Clone)]
pub struct ServiceConfig {
    pub name: String,
    pub display_name: String,
    pub description: String,
    pub executable_path: PathBuf,
    pub auto_restart: bool,
    pub restart_delay_secs: u32,
    pub max_restart_retries: u32,
}

impl ServiceConfig {
    /// Default config
    pub fn default_config(exe_path: PathBuf) -> Self {
        Self {
            name: "AppBlocker".to_string(),
            display_name: "App Blocker Service".to_string(),
            description: "Windows Application Blocker Service".to_string(),
            executable_path: exe_path,
            auto_restart: true,
            restart_delay_secs: 5,
            max_restart_retries: 3,
>>>>>>> bce0345919f371d153ccb843f2ddbfb5e8695c5f
        }
    }
}

<<<<<<< HEAD
/// Periksa dan dapatkan single-instance lock
///
/// Returns Ok(guard) jika berhasil, Err jika instance lain sudah berjalan
pub fn acquire_single_instance_lock() -> AppResult<SingleInstanceGuard> {
    let lock_path = Path::new(LOCK_FILE);

    // Buat direktori jika belum ada
    if let Some(parent) = lock_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| AppError::io("Buat direktori lock", e))?;
    }

    // Periksa apakah lock file sudah ada
    if lock_path.exists() {
        // Periksa apakah PID di file masih valid
        match std::fs::read_to_string(lock_path) {
            Ok(content) => {
                if let Ok(pid) = content.trim().parse::<u32>() {
                    if is_pid_running(pid) {
                        return Err(AppError::System(format!(
                            "Instance lain sudah berjalan (PID: {pid})"
                        )));
                    }
                    // PID tidak valid, hapus lock lama
                    warn!(pid, "Lock file lama dengan PID tidak aktif, menghapus");
                }
            }
            Err(_) => {
                warn!("Lock file tidak bisa dibaca, menghapus");
            }
        }
        std::fs::remove_file(lock_path)
            .map_err(|e| AppError::io("Hapus lock file lama", e))?;
    }

    // Tulis PID saat ini ke lock file
    let current_pid = std::process::id();
    std::fs::write(lock_path, current_pid.to_string())
        .map_err(|e| AppError::io("Tulis lock file", e))?;

    info!(pid = current_pid, "Single-instance lock berhasil diperoleh");
    Ok(SingleInstanceGuard::new(lock_path.to_path_buf()))
}

/// Periksa apakah flag disable darurat aktif
pub fn is_disable_flag_active() -> bool {
    Path::new(DISABLE_FLAG_FILE).exists()
}

/// Buat flag disable darurat
pub fn create_disable_flag() -> AppResult<()> {
    let flag_path = Path::new(DISABLE_FLAG_FILE);
    if let Some(parent) = flag_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| AppError::io("Buat dir disable flag", e))?;
    }
    std::fs::write(flag_path, "disabled")
        .map_err(|e| AppError::io("Buat disable flag", e))?;
    warn!("Flag disable darurat dibuat - blokir dihentikan");
    Ok(())
}

/// Hapus flag disable
pub fn remove_disable_flag() -> AppResult<()> {
    let flag_path = Path::new(DISABLE_FLAG_FILE);
    if flag_path.exists() {
        std::fs::remove_file(flag_path)
            .map_err(|e| AppError::io("Hapus disable flag", e))?;
        info!("Flag disable dihapus - blokir diaktifkan kembali");
    }
    Ok(())
}

/// Periksa apakah PID masih berjalan
fn is_pid_running(pid: u32) -> bool {
    #[cfg(target_os = "windows")]
    {
        use windows::Win32::Foundation::CloseHandle;
        use windows::Win32::System::Threading::{OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION};
        unsafe {
            match OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid) {
                Ok(handle) => {
                    let _ = CloseHandle(handle);
                    true
                }
                Err(_) => false,
            }
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        // Fallback: cek via /proc di Linux atau asumsi tidak berjalan
        Path::new(&format!("/proc/{pid}")).exists()
=======
/// Service manager
pub struct ServiceManager;

impl ServiceManager {
    /// Install service (requires admin)
    pub fn install(config: &ServiceConfig) -> AppResult<()> {
        // Using sc.exe command for service installation
        let output = std::process::Command::new("sc")
            .args([
                "create",
                &config.name,
                "binPath=",
                &config.executable_path.to_string_lossy(),
                "DisplayName=",
                &config.display_name,
                "start=",
                "auto",
            ])
            .output()
            .map_err(|e| AppError::ServiceError(format!("Failed to create service: {}", e)))?;
        
        if !output.status.success() {
            return Err(AppError::ServiceError(
                format!("sc create failed: {}", String::from_utf8_lossy(&output.stderr))
            ));
        }
        
        // Set description
        let _ = std::process::Command::new("sc")
            .args(["description", &config.name, &config.description])
            .output();
        
        tracing::info!("Service '{}' installed successfully", config.name);
        Ok(())
    }
    
    /// Uninstall service
    pub fn uninstall(service_name: &str) -> AppResult<()> {
        // Stop service first
        let _ = std::process::Command::new("sc")
            .args(["stop", service_name])
            .output();
        
        // Delete service
        let output = std::process::Command::new("sc")
            .args(["delete", service_name])
            .output()
            .map_err(|e| AppError::ServiceError(format!("Failed to delete service: {}", e)))?;
        
        if !output.status.success() {
            return Err(AppError::ServiceError(
                format!("sc delete failed: {}", String::from_utf8_lossy(&output.stderr))
            ));
        }
        
        tracing::info!("Service '{}' uninstalled successfully", service_name);
        Ok(())
    }
    
    /// Start service
    pub fn start(service_name: &str) -> AppResult<()> {
        let output = std::process::Command::new("sc")
            .args(["start", service_name])
            .output()
            .map_err(|e| AppError::ServiceError(format!("Failed to start service: {}", e)))?;
        
        if !output.status.success() {
            return Err(AppError::ServiceError(
                format!("sc start failed: {}", String::from_utf8_lossy(&output.stderr))
            ));
        }
        
        tracing::info!("Service '{}' started successfully", service_name);
        Ok(())
    }
    
    /// Stop service
    pub fn stop(service_name: &str) -> AppResult<()> {
        let output = std::process::Command::new("sc")
            .args(["stop", service_name])
            .output()
            .map_err(|e| AppError::ServiceError(format!("Failed to stop service: {}", e)))?;
        
        if !output.status.success() {
            return Err(AppError::ServiceError(
                format!("sc stop failed: {}", String::from_utf8_lossy(&output.stderr))
            ));
        }
        
        tracing::info!("Service '{}' stopped successfully", service_name);
        Ok(())
    }
    
    /// Check service status
    pub fn status(service_name: &str) -> AppResult<String> {
        let output = std::process::Command::new("sc")
            .args(["query", service_name])
            .output()
            .map_err(|e| AppError::ServiceError(format!("Failed to query service: {}", e)))?;
        
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
>>>>>>> bce0345919f371d153ccb843f2ddbfb5e8695c5f
    }
}
