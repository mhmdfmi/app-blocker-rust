//! Logger Module
//! 
//! Modul untuk structured logging dengan file rotation.

use crate::utils::error::AppResult;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use std::path::PathBuf;

/// Inisialisasi logger global
pub fn init_logger() -> AppResult<()> {
    // Ambil log path dari config atau gunakan default
    let log_dir = get_log_directory();
    
    // Buat directory jika belum ada
    std::fs::create_dir_all(&log_dir)?;
    
    // Setup file appender dengan rotation
    let file_appender = RollingFileAppender::new(
        Rotation::DAILY,
        &log_dir,
        "app_blocker.log",
    );
    
    // Setup subscriber dengan file dan stdout
    let file_layer = fmt::layer()
        .with_writer(file_appender)
        .with_ansi(false)
        .with_target(true)
        .with_line_number(true);
    
    let stdout_layer = fmt::layer()
        .with_writer(std::io::stdout)
        .with_ansi(true)
        .with_target(true);
    
    // Environment filter
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info,app_blocker=debug"));
    
    tracing_subscriber::registry()
        .with(filter)
        .with(file_layer)
        .with(stdout_layer)
        .init();
    
    tracing::info!("Logger initialized at: {:?}", log_dir);
    Ok(())
}

/// Get log directory
fn get_log_directory() -> PathBuf {
    // Cek environment variable atau gunakan default
    std::env::var("APP_BLOCKER_LOG_PATH")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("C:\\AppBlocker\\logs"))
}

/// Flush logs (dipanggil sebelum shutdown)
pub fn flush_logs() {
    tracing::info!("Flushing logs...");
}
