<<<<<<< HEAD
/// Sistem logging terstruktur untuk seluruh aplikasi.
/// Mendukung output konsol dan file dengan rotasi harian.
use crate::utils::error::{AppError, AppResult};
use std::path::Path;
use tracing::Level;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{
    fmt::{self, time::ChronoLocal},
    layer::SubscriberExt,
    util::SubscriberInitExt,
    EnvFilter,
};

/// Guard untuk memastikan log di-flush saat drop
pub struct LogGuard {
    _file_guard: WorkerGuard,
}

/// Inisialisasi sistem logging dengan output ke konsol dan file
///
/// # Arguments
/// * `log_dir` - Direktori untuk menyimpan file log
/// * `log_level` - Level logging (trace/debug/info/warn/error)
pub fn init_logger(log_dir: &Path, log_level: &str) -> AppResult<LogGuard> {
    // Buat direktori log jika belum ada
    std::fs::create_dir_all(log_dir)
        .map_err(|e| AppError::io(format!("Gagal membuat direktori log: {}", log_dir.display()), e))?;

    // Setup file appender dengan rotasi harian
    let file_appender = tracing_appender::rolling::daily(log_dir, "app_blocker.log");
    let (non_blocking_file, file_guard) = tracing_appender::non_blocking(file_appender);

    // Parse log level
    let level = parse_log_level(log_level);

    // Filter berdasarkan level
    let filter = EnvFilter::new(format!("app_blocker={level},app_blocker_lib={level}"));

    // Layer untuk output file (JSON format untuk machine-readable)
    let file_layer = fmt::layer()
        .with_writer(non_blocking_file)
        .with_ansi(false)
        .with_target(true)
        .with_thread_ids(true)
        .with_file(true)
        .with_line_number(true)
        .json();

    // Layer untuk output konsol (human-readable)
    let console_layer = fmt::layer()
        .with_writer(std::io::stdout)
        .with_ansi(true)
        .with_target(false)
        .with_timer(ChronoLocal::new("%Y-%m-%d %H:%M:%S%.3f".to_string()));

    // Inisialisasi subscriber global
    tracing_subscriber::registry()
        .with(filter)
        .with(file_layer)
        .with(console_layer)
        .try_init()
        .map_err(|e| AppError::Logging(format!("Gagal inisialisasi logger: {e}")))?;

    tracing::info!(
        version = env!("CARGO_PKG_VERSION"),
        log_dir = %log_dir.display(),
        level = log_level,
        "Logger diinisialisasi"
    );

    Ok(LogGuard {
        _file_guard: file_guard,
    })
}

/// Parse string level ke tracing Level
fn parse_log_level(level: &str) -> Level {
    match level.to_lowercase().as_str() {
        "trace" => Level::TRACE,
        "debug" => Level::DEBUG,
        "warn" => Level::WARN,
        "error" => Level::ERROR,
        _ => Level::INFO,
    }
}

/// Macro helper untuk log dengan trace_id
#[macro_export]
macro_rules! log_event {
    ($level:ident, trace_id = $trace_id:expr, $($field:tt)*) => {
        tracing::$level!(trace_id = %$trace_id, $($field)*)
    };
=======
﻿//! Logger Module
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
>>>>>>> bce0345919f371d153ccb843f2ddbfb5e8695c5f
}
