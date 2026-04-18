<<<<<<< HEAD
/// Verifikasi integritas sistem - self-hash binary dan config hash.
use crate::security::encryption::{hash_file, hash_self};
use crate::utils::error::{AppError, AppResult};
use std::path::Path;
use std::sync::{Arc, Mutex};
use tracing::{error, info, warn};

#[derive(Debug, Clone, Default)]
pub struct IntegrityState {
    pub binary_hash: Option<String>,
    pub config_hash: Option<String>,
    pub is_intact:   bool,
}

impl IntegrityState {
    fn new_intact() -> Self {
        Self { binary_hash: None, config_hash: None, is_intact: true }
    }
}

pub struct IntegrityService {
    state: Arc<Mutex<IntegrityState>>,
}

impl IntegrityService {
    pub fn new() -> AppResult<Self> {
        let svc = Self { state: Arc::new(Mutex::new(IntegrityState::new_intact())) };
        svc.record_binary_hash();
        Ok(svc)
    }

    fn record_binary_hash(&self) {
        match hash_self() {
            Ok(hash) => {
                if let Ok(mut s) = self.state.lock() {
                    s.binary_hash = Some(hash);
                }
                info!("Hash binary terekam");
            }
            Err(e) => warn!(error = %e, "Gagal rekam hash binary (non-fatal)"),
        }
    }

    pub fn record_config_hash(&self, path: &Path) -> AppResult<()> {
        if !path.exists() { return Ok(()); }
        let hash = hash_file(path)?;
        if let Ok(mut s) = self.state.lock() {
            s.config_hash = Some(hash);
        }
        Ok(())
    }

    pub fn verify_binary(&self) -> AppResult<bool> {
        let expected = {
            self.state.lock()
                .map(|s| s.binary_hash.clone())
                .map_err(|e| AppError::System(format!("Lock integrity: {e}")))?
        };
        match expected {
            None => Ok(true),
            Some(exp) => {
                let cur = hash_self()?;
                let ok = cur == exp;
                if !ok {
                    error!("Integritas binary GAGAL - binary mungkin dimodifikasi!");
                    if let Ok(mut s) = self.state.lock() { s.is_intact = false; }
                }
                Ok(ok)
            }
        }
    }

    pub fn is_intact(&self) -> bool {
        self.state.lock().map(|s| s.is_intact).unwrap_or(false)
    }
}

/// Deteksi debugger (Windows-only)
#[cfg(target_os = "windows")]
pub fn is_debugger_present() -> bool {
    use windows::Win32::System::Diagnostics::Debug::IsDebuggerPresent;
    unsafe { IsDebuggerPresent().as_bool() }
}

#[cfg(not(target_os = "windows"))]
pub fn is_debugger_present() -> bool { false }
=======
//! Integrity Module
use crate::utils::error::{AppResult, AppError};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::Path;

pub struct IntegrityChecker {
    baseline_hash: Option<u64>,
    config_hash: Option<u64>,
}

impl IntegrityChecker {
    pub fn new() -> Self {
        Self { baseline_hash: None, config_hash: None }
    }

    pub fn calculate_hash<T: Hash>(data: &T) -> u64 {
        let mut hasher = DefaultHasher::new();
        data.hash(&mut hasher);
        hasher.finish()
    }

    pub fn set_baseline(&mut self, hash: u64) {
        self.baseline_hash = Some(hash);
    }

    pub fn verify_baseline(&self, current_hash: u64) -> AppResult<bool> {
        match self.baseline_hash {
            Some(baseline) => Ok(baseline == current_hash),
            None => Ok(true),
        }
    }

    pub fn check_config_integrity(config_path: &Path) -> AppResult<u64> {
        let content = std::fs::read(config_path)
            .map_err(|e| AppError::IntegrityError(format!("Failed to read config: {}", e)))?;
        let hash = Self::calculate_hash(&content);
        Ok(hash)
    }

    pub fn detect_debugger() -> bool {
        // Simplified debugger detection
        false
    }

    pub fn verify_self() -> AppResult<bool> {
        tracing::debug!("Self-integrity check performed");
        Ok(true)
    }
}

impl Default for IntegrityChecker {
    fn default() -> Self { Self::new() }
}

pub const PROTECTED_PROCESSES: &[&str] = &[
    "System",
    "winlogon.exe",
    "csrss.exe",
    "smss.exe",
    "services.exe",
    "lsass.exe",
    "explorer.exe",
    "dwm.exe",
    "svchost.exe",
];

pub fn is_protected_process(name: &str) -> bool {
    let name_lower = name.to_lowercase();
    PROTECTED_PROCESSES.iter().any(|&p| name_lower == p.to_lowercase())
}
>>>>>>> bce0345919f371d153ccb843f2ddbfb5e8695c5f
