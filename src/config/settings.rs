//! Settings Module
//! 
//! Pengaturan aplikasi yang dapat dikonfigurasi.

use serde::{Deserialize, Serialize};
use crate::utils::error::{AppResult, AppError};

/// Application settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    // General
    pub app_name: String,
    pub version: String,
    pub log_level: String,
    
    // Monitoring
    pub monitoring_interval_ms: u64,
    pub validation_delay_ms_min: u64,
    pub validation_delay_ms_max: u64,
    pub rate_limit_per_minute: u32,
    
    // Blacklist
    pub blacklist: Vec<String>,
    
    // Whitelist
    pub whitelist: Vec<String>,
    
    // Simulation
    pub simulation_mode: bool,
    pub simulate_process_kill: bool,
    pub simulate_overlay: bool,
    pub log_only_mode: bool,
    
    // Security
    pub max_auth_attempts: u32,
    pub lockout_duration_seconds: u64,
    pub memory_zeroing: bool,
    
    // Startup
    pub startup_delay_ms: u64,
    
    // Overlay
    pub overlay_title: String,
    pub overlay_message: String,
    pub failsafe_timeout_minutes: u64,
    
    // Resource limits
    pub max_cpu_percent: u32,
    pub max_memory_mb: u32,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            app_name: "AppBlocker".to_string(),
            version: "1.0.0".to_string(),
            log_level: "INFO".to_string(),
            
            monitoring_interval_ms: 1000,
            validation_delay_ms_min: 300,
            validation_delay_ms_max: 1000,
            rate_limit_per_minute: 3,
            
            blacklist: vec![
                "discord".to_string(),
                "spotify".to_string(),
                "telegram".to_string(),
                "steam".to_string(),
                "game".to_string(),
            ],
            
            whitelist: vec![
                "explorer.exe".to_string(),
                "chrome.exe".to_string(),
                "firefox.exe".to_string(),
            ],
            
            simulation_mode: true,
            simulate_process_kill: true,
            simulate_overlay: true,
            log_only_mode: true,
            
            max_auth_attempts: 5,
            lockout_duration_seconds: 300,
            memory_zeroing: true,
            
            startup_delay_ms: 5000,
            
            overlay_title: "Peringatan Keamanan".to_string(),
            overlay_message: "Aplikasi terlarang terdeteksi dan telah ditutup.".to_string(),
            failsafe_timeout_minutes: 30,
            
            max_cpu_percent: 20,
            max_memory_mb: 200,
        }
    }
}

impl Settings {
    /// Load dari TOML file
    pub fn load_from_file(path: &str) -> AppResult<Self> {
        let content = std::fs::read_to_string(path)?;
        let mut settings: Settings = toml::from_str(&content)?;
        
        // Override dengan environment variables jika ada
        settings.apply_env_overrides();
        
        Ok(settings)
    }
    
    /// Apply environment variable overrides
    fn apply_env_overrides(&mut self) {
        if let Ok(val) = std::env::var("SIMULATION_MODE") {
            self.simulation_mode = val.parse().unwrap_or(true);
        }
        if let Ok(val) = std::env::var("LOG_LEVEL") {
            self.log_level = val;
        }
    }
    
    /// Validate settings
    pub fn validate(&self) -> AppResult<()> {
        if self.monitoring_interval_ms == 0 {
            return Err(AppError::ConfigError(
                "monitoring_interval_ms must be > 0".to_string()
            ));
        }
        
        if self.max_auth_attempts == 0 {
            return Err(AppError::ConfigError(
                "max_auth_attempts must be > 0".to_string()
            ));
        }
        
        Ok(())
    }
    
    /// Add ke blacklist
    pub fn add_to_blacklist(&mut self, process: String) {
        if !self.blacklist.contains(&process) {
            self.blacklist.push(process);
        }
    }
    
    /// Remove dari blacklist
    pub fn remove_from_blacklist(&mut self, process: &str) {
        self.blacklist.retain(|p| p != process);
    }
    
    /// Check apakah dalam whitelist
    pub fn is_whitelisted(&self, process_name: &str) -> bool {
        let name_lower = process_name.to_lowercase();
        self.whitelist.iter().any(|w| name_lower.contains(&w.to_lowercase()))
    }
}
