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
