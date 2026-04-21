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
    let monitor_cfg = &config.monitoring;

    if monitor_cfg.scan_interval_ms < 100 {
        return Err(AppError::Validation(
            "scan_interval_ms harus minimal 100ms".to_string(),
        ));
    }

    if monitor_cfg.scan_interval_ms > 60_000 {
        return Err(AppError::Validation(
            "scan_interval_ms harus maksimal 60000ms".to_string(),
        ));
    }

    let (min_delay, max_delay) = monitor_cfg.validation_delay_ms;
    if min_delay > max_delay {
        return Err(AppError::Validation(
            "validation_delay_ms: min tidak boleh lebih besar dari max".to_string(),
        ));
    }

    Ok(())
}

fn validate_blocking(config: &AppConfig) -> AppResult<()> {
    let blocking_cfg = &config.blocking;

    if blocking_cfg.kill_rate_limit_per_minute == 0 {
        return Err(AppError::Validation(
            "kill_rate_limit_per_minute harus > 0".to_string(),
        ));
    }

    if blocking_cfg.score_threshold > 100 {
        return Err(AppError::Validation(
            "score_threshold harus antara 0-100".to_string(),
        ));
    }

    // Validasi setiap entri blacklist
    for app in &blocking_cfg.blacklist {
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
    let overlay_cfg = &config.overlay;

    if overlay_cfg.focus_interval_ms < 100 {
        return Err(AppError::Validation(
            "focus_interval_ms harus minimal 100ms".to_string(),
        ));
    }

    if overlay_cfg.max_unlock_attempts == 0 {
        return Err(AppError::Validation(
            "max_unlock_attempts harus > 0".to_string(),
        ));
    }

    if overlay_cfg.failsafe_timeout_minutes == 0 {
        return Err(AppError::Validation(
            "failsafe_timeout_minutes harus > 0".to_string(),
        ));
    }

    Ok(())
}

fn validate_security(config: &AppConfig) -> AppResult<()> {
    let security_cfg = &config.security;

    if security_cfg.max_auth_attempts == 0 {
        return Err(AppError::Validation(
            "max_auth_attempts harus > 0".to_string(),
        ));
    }

    if security_cfg.lockout_duration_seconds == 0 {
        return Err(AppError::Validation(
            "lockout_duration_seconds harus > 0".to_string(),
        ));
    }

    Ok(())
}

fn validate_watchdog(config: &AppConfig) -> AppResult<()> {
    let watchdog_cfg = &config.watchdog;

    if watchdog_cfg.heartbeat_interval_ms < 100 {
        return Err(AppError::Validation(
            "heartbeat_interval_ms harus minimal 100ms".to_string(),
        ));
    }

    if watchdog_cfg.max_restart_attempts == 0 {
        return Err(AppError::Validation(
            "max_restart_attempts harus > 0".to_string(),
        ));
    }

    Ok(())
}
