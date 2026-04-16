//! Environment Loader Module

use crate::utils::error::{AppResult, AppError};
use std::env;
use std::path::Path;

/// Load environment variables dari .env file
pub fn load_env() -> AppResult<()> {
    // Cari .env file
    let env_paths = vec![
        Path::new(".env"),
        Path::new(".env.local"),
        Path::new("config/.env"),
    ];
    
    for env_path in env_paths {
        if env_path.exists() {
            load_env_file(env_path)?;
            tracing::debug!("Loaded environment from: {:?}", env_path);
            return Ok(());
        }
    }
    
    // Tidak ada .env file, tapi tidak error
    tracing::warn!("No .env file found, using system environment");
    Ok(())
}

/// Load file dan set environment variables
fn load_env_file(path: &Path) -> AppResult<()> {
    let content = std::fs::read_to_string(path)?;
    
    for line in content.lines() {
        let line = line.trim();
        
        // Skip comments dan empty lines
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        
        // Parse KEY=VALUE
        if let Some((key, value)) = line.split_once('=') {
            let key = key.trim();
            let value = value.trim();
            
            // Remove quotes jika ada
            let value = value.trim_matches('"').trim_matches('\'');
            
            // Set environment variable
            env::set_var(key, value);
        }
    }
    
    Ok(())
}

/// Get environment variable dengan default
pub fn get_env<T: std::str::FromStr>(key: &str, default: T) -> T {
    env::var(key)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}

/// Get required environment variable
pub fn get_env_required(key: &str) -> AppResult<String> {
    env::var(key).map_err(|_| {
        AppError::ConfigError(format!("Required environment variable '{}' not set", key))
    })
}
