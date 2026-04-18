<<<<<<< HEAD
/// Validator skema konfigurasi - memastikan semua nilai dalam rentang yang aman.
use crate::config::settings::AppConfig;
use crate::utils::error::{AppError, AppResult};

/// Validasi seluruh konfigurasi
pub fn validate_config(config: &AppConfig) -> AppResult<()> {
    validate_monitoring(config)?;
    validate_blocking(config)?;
    validate_overlay(config)?;
    validate_security(config)?;
    validate_watchdog(config)?;
    Ok(())
}

fn validate_monitoring(config: &AppConfig) -> AppResult<()> {
    let m = &config.monitoring;

    if m.scan_interval_ms < 100 {
        return Err(AppError::Validation(
            "scan_interval_ms harus minimal 100ms".to_string(),
        ));
    }

    if m.scan_interval_ms > 60_000 {
        return Err(AppError::Validation(
            "scan_interval_ms harus maksimal 60000ms".to_string(),
        ));
    }

    let (min_delay, max_delay) = m.validation_delay_ms;
    if min_delay > max_delay {
        return Err(AppError::Validation(
            "validation_delay_ms: min tidak boleh lebih besar dari max".to_string(),
        ));
    }

    Ok(())
}

fn validate_blocking(config: &AppConfig) -> AppResult<()> {
    let b = &config.blocking;

    if b.kill_rate_limit_per_minute == 0 {
        return Err(AppError::Validation(
            "kill_rate_limit_per_minute harus > 0".to_string(),
        ));
    }

    if b.score_threshold > 100 {
        return Err(AppError::Validation(
            "score_threshold harus antara 0-100".to_string(),
        ));
    }

    // Validasi setiap entri blacklist
    for app in &b.blacklist {
        if app.name.is_empty() {
            return Err(AppError::Validation(
                "Nama aplikasi di blacklist tidak boleh kosong".to_string(),
            ));
        }
        if app.process_names.is_empty() {
            return Err(AppError::Validation(format!(
                "Blacklist '{}' harus memiliki minimal satu process_name",
                app.name
            )));
        }
    }

    Ok(())
}

fn validate_overlay(config: &AppConfig) -> AppResult<()> {
    let o = &config.overlay;

    if o.focus_interval_ms < 100 {
        return Err(AppError::Validation(
            "focus_interval_ms harus minimal 100ms".to_string(),
        ));
    }

    if o.max_unlock_attempts == 0 {
        return Err(AppError::Validation(
            "max_unlock_attempts harus > 0".to_string(),
        ));
    }

    if o.failsafe_timeout_minutes == 0 {
        return Err(AppError::Validation(
            "failsafe_timeout_minutes harus > 0".to_string(),
        ));
    }

    Ok(())
}

fn validate_security(config: &AppConfig) -> AppResult<()> {
    let s = &config.security;

    if s.max_auth_attempts == 0 {
        return Err(AppError::Validation(
            "max_auth_attempts harus > 0".to_string(),
        ));
    }

    if s.lockout_duration_seconds == 0 {
        return Err(AppError::Validation(
            "lockout_duration_seconds harus > 0".to_string(),
        ));
    }

    Ok(())
}

fn validate_watchdog(config: &AppConfig) -> AppResult<()> {
    let w = &config.watchdog;

    if w.heartbeat_interval_ms < 100 {
        return Err(AppError::Validation(
            "heartbeat_interval_ms harus minimal 100ms".to_string(),
        ));
    }

    if w.max_restart_attempts == 0 {
        return Err(AppError::Validation(
            "max_restart_attempts harus > 0".to_string(),
        ));
    }

=======
//! Validator Module
use crate::utils::error::{AppResult, AppError};
use crate::system::user::UserInfo;
pub fn validate_permissions() -> AppResult<()> {
    let user = UserInfo::current().map_err(|e| AppError::AuthError(e.to_string()))?;
    if !user.is_admin { tracing::warn!("No admin"); }
    Ok(())
}
pub fn validate_config() -> AppResult<()> { Ok(()) }
pub fn validate_blacklist(blacklist: &[String]) -> AppResult<()> {
    if blacklist.is_empty() { return Err(AppError::ConfigError("empty".into())); }
>>>>>>> bce0345919f371d153ccb843f2ddbfb5e8695c5f
    Ok(())
}
