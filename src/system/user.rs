//! User Module
use crate::utils::error::{AppResult, AppError};
use std::env;

#[derive(Debug, Clone)]
pub struct UserInfo {
    pub username: String,
    pub domain: Option<String>,
    pub is_admin: bool,
}

impl UserInfo {
    pub fn current() -> AppResult<Self> {
        let username = env::var("USERNAME").or_else(|_| env::var("USER")).unwrap_or_else(|_| "unknown".to_string());
        let is_admin = Self::check_admin();
        Ok(Self {
            username,
            domain: env::var("USERDOMAIN").ok(),
            is_admin,
        })
    }

    pub fn check_admin() -> bool {
        #[cfg(target_os = "windows")]
        {
            // Simplified admin check
            let output = std::process::Command::new("net").args(["session"]).output();
            output.map(|o| o.status.success()).unwrap_or(false)
        }
        #[cfg(not(target_os = "windows"))]
        false
    }

    pub fn formatted(&self) -> String {
        match &self.domain {
            Some(d) => format!("{}\\{}", d, self.username),
            None => self.username.clone(),
        }
    }
}
