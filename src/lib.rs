/// App Blocker Library - ekspor semua modul publik.
/// Memungkinkan testing dan integrasi eksternal.
pub mod cli;
pub mod config;
pub mod constants;
pub mod core;
pub mod detection;
pub mod metrics;
pub mod security;
pub mod system;
pub mod ui;
pub mod utils;

// Re-export tipe utama
pub use config::ConfigManager;
pub use core::audit::{AuditEntry, AuditEventKind, GLOBAL_AUDIT};
pub use core::{AppEngine, AppEvent, AppState, StateManager};
pub use metrics::AppMetrics;
pub use security::auth::{Argon2AuthService, AuthManager, AuthStatus};
pub use system::{create_disable_flag, ProcessInfo, WindowsProcessService};
pub use utils::error::{AppError, AppResult};
