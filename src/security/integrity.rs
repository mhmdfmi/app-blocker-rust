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
