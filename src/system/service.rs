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
        }
    }
}

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
    }
}
