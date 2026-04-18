use crate::constants::paths::DISABLE_FLAG_FILE;
// use crate::constants::paths::LOCK_FILE;  // Jika Anda ingin menggunakan file lock untuk single-instance
/// Manajemen layanan Windows dan single-instance lock.
use crate::utils::error::{AppError, AppResult};
use std::io::Write;
use std::path::{Path, PathBuf};
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
        }
    }
}

/// Service manager
pub struct ServiceManager;

#[derive(Debug, Clone)]
pub struct ServiceConfig {
    pub name: String,
    pub display_name: String,
    pub description: String,
    pub executable_path: std::path::PathBuf,
}

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
            return Err(AppError::ServiceError(format!(
                "sc create failed: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
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
            return Err(AppError::ServiceError(format!(
                "sc delete failed: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
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
            return Err(AppError::ServiceError(format!(
                "sc start failed: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
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
            return Err(AppError::ServiceError(format!(
                "sc stop failed: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
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
    }
}

/// Acquire single instance lock - standalone function
pub fn acquire_single_instance_lock(lock_path: PathBuf) -> AppResult<SingleInstanceGuard> {
    // Jika file sudah ada, baca isinya (opsional) dan kembalikan error
    if lock_path.exists() {
        // coba baca PID lama untuk logging (tidak fatal)
        match std::fs::read_to_string(&lock_path) {
            Ok(s) => {
                let s = s.trim();
                return Err(AppError::System(format!(
                    "Instance lock exists (pid={}). Jika yakin tidak ada proses lain, hapus {} dan coba lagi.",
                    s,
                    lock_path.display()
                )));
            }
            Err(_) => {
                return Err(AppError::System(format!(
                    "Instance lock exists: {}",
                    lock_path.display()
                )));
            }
        }
    }

    // Tulis PID ke file lock
    let pid = std::process::id();
    let mut f = std::fs::File::create(&lock_path).map_err(|e| {
        AppError::System(format!(
            "Gagal buat lock file {}: {}",
            lock_path.display(),
            e
        ))
    })?;
    writeln!(f, "{}", pid).map_err(|e| {
        AppError::System(format!(
            "Gagal tulis ke lock file {}: {}",
            lock_path.display(),
            e
        ))
    })?;
    // flush agar data tersimpan
    if let Err(e) = f.sync_all() {
        // tidak fatal, tapi log/return error jika Anda ingin lebih ketat
        tracing::warn!(error = %e, path = %lock_path.display(), "Gagal sync lock file");
    }

    Ok(SingleInstanceGuard::new(lock_path))
}

/// Periksa apakah file disable-flag ada.
/// Fungsi ini menggunakan konstanta DISABLE_FLAG_FILE; jika tidak ada, ganti path sesuai proyek Anda.
pub fn is_disable_flag_active() -> bool {
    // Jika Anda ingin path yang dapat dikonfigurasi, ubah fungsi ini untuk membaca dari config.
    let flag_path: &Path = Path::new(DISABLE_FLAG_FILE);
    flag_path.exists()
}

pub fn create_disable_flag() -> AppResult<()> {
    let flag_path: &Path = Path::new(DISABLE_FLAG_FILE);
    std::fs::write(flag_path, "disabled").map_err(|e| {
        AppError::System(format!(
            "Gagal buat disable flag {}: {}",
            flag_path.display(),
            e
        ))
    })?;
    Ok(())
}
