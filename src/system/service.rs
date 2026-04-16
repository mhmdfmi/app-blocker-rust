//! Service Module
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
        }
    }
}

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
    }
}
