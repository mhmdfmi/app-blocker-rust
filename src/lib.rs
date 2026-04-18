<<<<<<< HEAD
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
pub use core::{AppEngine, AppEvent, AppState, StateManager};
pub use core::audit::{AuditEntry, AuditEventKind, GLOBAL_AUDIT};
pub use metrics::AppMetrics;
pub use security::auth::{AuthManager, AuthStatus, Argon2AuthService};
pub use system::{ProcessInfo, WindowsProcessService};
pub use utils::error::{AppError, AppResult};
=======
//! App Blocker
pub mod core;
pub mod ui;
pub mod security;
pub mod system;
pub mod utils;
pub mod config;
pub mod constants;
pub mod detection;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const APP_NAME: &str = "AppBlocker";
>>>>>>> bce0345919f371d153ccb843f2ddbfb5e8695c5f
