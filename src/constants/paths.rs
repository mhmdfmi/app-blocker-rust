//! Konstanta path sistem untuk aplikasi.
/// Menggunakan AppData\Local\AppBlocker sebagai base dengan subfolder terpisah
use std::path::PathBuf;

/// Nama folder aplikasi di AppData
pub const APP_FOLDER_NAME: &str = "AppBlocker";

/// Get the base directory - ALWAYS uses the directory where the exe is located
/// Ini memastikan app bisa dijalankan dari mana saja (bukan dari working directory).
pub fn get_app_dir() -> PathBuf {
    std::env::current_exe()
        .map(|p| p.parent().unwrap_or(&p).to_path_buf())
        .unwrap_or_else(|_| PathBuf::from("."))
}

/// Get AppData folder path (user's local app data)
/// Returns: C:\Users\<username>\AppData\Local\AppBlocker
pub fn get_appdata_dir() -> PathBuf {
    dirs::data_local_dir()
        .map(|p| p.join(APP_FOLDER_NAME))
        .unwrap_or_else(|| get_app_dir().join("data"))
}

/// Get database folder path in AppData
/// Returns: C:\Users\<username>\AppData\Local\AppBlocker\db
pub fn get_db_dir() -> PathBuf {
    get_appdata_dir().join("db")
}

/// Get database path in AppData
/// Returns: C:\Users\<username>\AppData\Local\AppBlocker\db\core.db
pub fn get_db_path() -> PathBuf {
    get_db_dir().join("core.db")
}

/// Get logs folder path in AppData
/// Returns: C:\Users\<username>\AppData\Local\AppBlocker\logs
pub fn get_logs_dir() -> PathBuf {
    get_appdata_dir().join("logs")
}

/// Get reports folder path in AppData
/// Returns: C:\Users\<username>\AppData\Local\AppBlocker\reports
pub fn get_reports_dir() -> PathBuf {
    get_appdata_dir().join("reports")
}

/// Get config path in AppData (legacy - now using database)
pub fn get_config_path() -> PathBuf {
    get_appdata_dir().join("config.toml")
}

/// Ensure all AppData subdirectories exist
pub fn ensure_appdata_dir() -> std::io::Result<PathBuf> {
    let dir = get_appdata_dir();
    std::fs::create_dir_all(&dir)?;
    std::fs::create_dir_all(get_db_dir())?;
    std::fs::create_dir_all(get_logs_dir())?;
    std::fs::create_dir_all(get_reports_dir())?;
    Ok(dir)
}

/// File kunci single instance
pub fn get_lock_file() -> PathBuf {
    get_appdata_dir().join("app.lock")
}

/// Flag disable darurat - jika file ini ada, blokir dihentikan
pub fn get_disable_flag_file() -> PathBuf {
    get_appdata_dir().join("disable")
}

/// File konfigurasi utama (relative to exe location)
pub fn get_default_config_path() -> PathBuf {
    get_app_dir().join("config/default.toml")
}

pub fn get_production_config_path() -> PathBuf {
    get_app_dir().join("config/production.toml")
}

// ===== BACKWARD COMPATIBILITY CONSTANTS ====
// Menggunakan path legacy untuk backward compatibility

/// Direktori utama (current working directory)
pub const APP_DIR: &str = ".";

/// File kunci single instance
pub const LOCK_FILE: &str = "app.lock";

/// Flag disable darurat
pub const DISABLE_FLAG_FILE: &str = "disable";

/// Direktori laporan audit
pub const REPORTS_DIR: &str = "reports";

/// Direktori log aplikasi
pub const LOGS_DIR: &str = "logs";

/// File konfigurasi utama
pub const DEFAULT_CONFIG_PATH: &str = "config/default.toml";

pub const PRODUCTION_CONFIG_PATH: &str = "config/production.toml";

/// File .env untuk kredensial
pub const ENV_FILE: &str = ".env";

/// Named mutex untuk single instance check
pub const INSTANCE_MUTEX_NAME: &str = "Global\\AppBlocker_SingleInstance_Mutex";

/// Proses yang dilindungi - tidak boleh pernah dihentikan
pub const PROTECTED_PROCESSES: &[&str] = &[
    "System",
    "winlogon.exe",
    "csrss.exe",
    "smss.exe",
    "services.exe",
    "lsass.exe",
    "explorer.exe",
    "svchost.exe",
    "wininit.exe",
    "dwm.exe",
];

/// Ekstensi file yang dipantau
pub const WATCHED_EXTENSIONS: &[&str] = &[".exe", ".lnk"];

/// Path direktori yang dipantau
pub const WATCH_PATHS: &[&str] = &[
    r"C:\Program Files",
    r"C:\Program Files (x86)",
    r"C:\ProgramData",
];
