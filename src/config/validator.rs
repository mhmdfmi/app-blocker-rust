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
    Ok(())
}
