/// App Blocker Library - ekspor semua modul publik.
/// Memungkinkan testing dan integrasi eksternal.
pub mod bootstrap;
pub mod cli;
pub mod config;
pub mod constants;
pub mod core;
pub mod db;
pub mod detection;
pub mod metrics;
pub mod models;
pub mod repository;
pub mod security;
pub mod system;
pub mod ui;
pub mod utils;

// Re-export tipe utama
pub use bootstrap::{AppBootstrap, Application};
pub use config::{ConfigManager, DbConfigLoader};
pub use core::audit::{AuditEntry, AuditEventKind, GLOBAL_AUDIT};
pub use core::{AppEngine, AppEvent, AppState, StateManager};
pub use metrics::AppMetrics;
pub use repository::{
    BlacklistRepository, ConfigRepository, LogRepository, ScheduleRepository, UserRepository,
    WhitelistRepository,
};
pub use security::auth::{Argon2AuthService, AuthManager, AuthStatus};
pub use system::{create_disable_flag, ProcessInfo, WindowsProcessService};
pub use utils::error::{AppError, AppResult};
