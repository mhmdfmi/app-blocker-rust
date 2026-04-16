//! Time Module

use chrono::{DateTime, Utc, Local, TimeZone};

/// Get current UTC timestamp
pub fn now_utc() -> DateTime<Utc> {
    Utc::now()
}

/// Get current local timestamp
pub fn now_local() -> DateTime<Local> {
    Local::now()
}

/// Format timestamp untuk logging
pub fn format_timestamp(dt: &DateTime<Utc>) -> String {
    dt.format("%Y-%m-%d %H:%M:%S%.3f UTC").to_string()
}

/// Format untuk filename
pub fn format_for_filename(dt: &DateTime<Utc>) -> String {
    dt.format("%Y%m%d_%H%M%S").to_string()
}

/// Sleep dengan durasi
pub fn sleep_ms(ms: u64) {
    std::thread::sleep(std::time::Duration::from_millis(ms));
}
