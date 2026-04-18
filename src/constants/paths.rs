/// Konstanta path sistem untuk aplikasi.

/// Direktori instalasi utama
pub const APP_DIR: &str = r"C:\AppBlocker";

/// File kunci single instance
pub const LOCK_FILE: &str = r"C:\AppBlocker\app.lock";

/// Flag disable darurat - jika file ini ada, blokir dihentikan
pub const DISABLE_FLAG_FILE: &str = r"C:\AppBlocker\disable";

/// Direktori laporan audit
pub const REPORTS_DIR: &str = r"C:\AppBlocker\reports";

/// Direktori log aplikasi
pub const LOGS_DIR: &str = r"C:\AppBlocker\logs";

/// File konfigurasi utama
pub const DEFAULT_CONFIG_PATH: &str = r"config\default.toml";
pub const PRODUCTION_CONFIG_PATH: &str = r"config\production.toml";

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
