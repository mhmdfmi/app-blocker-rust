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
