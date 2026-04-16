//! Error Module
//! 
//! Custom error types untuk seluruh aplikasi.

use thiserror::Error;

/// Result type dengan error kustom
pub type AppResult<T> = Result<T, AppError>;

/// Main error enum
#[derive(Debug, Error)]
pub enum AppError {
    #[error("Process error: {0}")]
    ProcessError(String),
    
    #[error("Authentication error: {0}")]
    AuthError(String),
    
    #[error("Encryption error: {0}")]
    EncryptionError(String),
    
    #[error("Integrity error: {0}")]
    IntegrityError(String),
    
    #[error("Configuration error: {0}")]
    ConfigError(String),
    
    #[error("Thread error: {0}")]
    ThreadError(String),
    
    #[error("State error: {0}")]
    StateError(String),
    
    #[error("Service error: {0}")]
    ServiceError(String),
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("Parse error: {0}")]
    ParseError(String),
    
    #[error("Unknown error: {0}")]
    Unknown(String),
}

impl AppError {
    /// Create error dengan context
    pub fn with_context<E: std::fmt::Display>(self, context: E) -> Self {
        AppError::Unknown(format!("{}: {}", context, self))
    }
    
    /// Get error type untuk telemetry
    pub fn error_type(&self) -> &'static str {
        match self {
            AppError::ProcessError(_) => "process_error",
            AppError::AuthError(_) => "auth_error",
            AppError::EncryptionError(_) => "encryption_error",
            AppError::IntegrityError(_) => "integrity_error",
            AppError::ConfigError(_) => "config_error",
            AppError::ThreadError(_) => "thread_error",
            AppError::StateError(_) => "state_error",
            AppError::ServiceError(_) => "service_error",
            AppError::IoError(_) => "io_error",
            AppError::ParseError(_) => "parse_error",
            AppError::Unknown(_) => "unknown",
        }
    }
}

// Implement From untuk konversi antar error types
impl From<toml::de::Error> for AppError {
    fn from(e: toml::de::Error) -> Self {
        AppError::ParseError(e.to_string())
    }
}

impl From<toml::ser::Error> for AppError {
    fn from(e: toml::ser::Error) -> Self {
        AppError::ParseError(e.to_string())
    }
}
