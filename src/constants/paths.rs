//! Konstanta path sistem untuk aplikasi.
/// Menggunakan path relatif terhadap direktori kerja (working directory) agar bisa dipindahkan.
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

/// Get database path in AppData
pub fn get_db_path() -> PathBuf {
    get_appdata_dir().join("core.db")
}

/// Get config path in AppData
pub fn get_config_path() -> PathBuf {
    get_appdata_dir().join("config.toml")
}

/// Ensure AppData directory exists
pub fn ensure_appdata_dir() -> std::io::Result<PathBuf> {
    let dir = get_appdata_dir();
    std::fs::create_dir_all(&dir)?;
    Ok(dir)
}

/// Direktori log aplikasi
pub fn get_logs_dir() -> PathBuf {
    get_app_dir().join("logs")
}
/// Direktori laporan audit
pub fn get_reports_dir() -> PathBuf {
    get_app_dir().join("reports")
}

/// File kunci single instance
pub fn get_lock_file() -> PathBuf {
    get_app_dir().join("app.lock")
}

/// Flag disable darurat - jika file ini ada, blokir dihentikan
pub fn get_disable_flag_file() -> PathBuf {
    get_app_dir().join("disable")
}

/// File konfigurasi utama (relative to exe location)
pub fn get_default_config_path() -> PathBuf {
    get_app_dir().join("config/default.toml")
}

pub fn get_production_config_path() -> PathBuf {
    get_app_dir().join("config/production.toml")
}

// ===== BACKWARD COMPATIBILITY CONSTANTS ====
// Menggunakan path relatif terhadap direktori kerja saat ini

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
