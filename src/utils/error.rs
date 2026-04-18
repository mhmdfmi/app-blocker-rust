/// Sistem error terpusat untuk seluruh aplikasi.
/// Semua error harus memiliki konteks yang jelas.
use thiserror::Error;

/// Error utama aplikasi - semua modul menggunakan tipe ini
#[derive(Debug, Error)]
pub enum AppError {
    /// Error terkait konfigurasi
    #[error("Kesalahan konfigurasi: {0}")]
    Config(String),

    /// Error terkait autentikasi
    #[error("Kesalahan autentikasi: {0}")]
    Auth(String),

    /// Error terkait proses Windows
    #[error("Kesalahan proses: {0}")]
    Process(String),

    /// Error terkait state machine
    #[error("Transisi state tidak valid: dari {from} ke {to}")]
    InvalidStateTransition { from: String, to: String },

    /// Error state sudah dalam kondisi ini
    #[error("State sudah dalam kondisi {0}")]
    StateAlreadySet(String),

    /// Error terkait UI overlay
    #[error("Kesalahan overlay UI: {0}")]
    Overlay(String),

    /// Error Win32 API
    #[error("Kesalahan Win32 API: {0}")]
    Win32(String),

    /// Error terkait channel komunikasi
    #[error("Kesalahan channel: {0}")]
    Channel(String),

    /// Error terkait I/O file
    #[error("Kesalahan I/O: {context}: {source}")]
    Io {
        context: String,
        #[source]
        source: std::io::Error,
    },

    /// Error terkait serialisasi/deserialisasi
    #[error("Kesalahan serialisasi: {0}")]
    Serialization(String),

    /// Error terkait sistem operasi
    #[error("Kesalahan sistem: {0}")]
    System(String),

    /// Error terkait logging
    #[error("Kesalahan logging: {0}")]
    Logging(String),

    /// Error thread watchdog
    #[error("Kesalahan watchdog: {0}")]
    Watchdog(String),

    /// Error deteksi proses mencurigakan
    #[error("Kesalahan deteksi: {0}")]
    Detection(String),

    /// Error tidak diketahui dengan konteks
    #[error("Kesalahan tidak diketahui: {0}")]
    Unknown(String),

    /// Error proses yang dilindungi
    #[error("Proses terlindungi tidak boleh dihentikan: {0}")]
    ProtectedProcess(String),

    /// Error timeout operasi
    #[error("Operasi timeout setelah {duration_ms}ms: {operation}")]
    Timeout { operation: String, duration_ms: u64 },

    /// Error integrity check
    #[error("Gagal verifikasi integritas: {0}")]
    IntegrityViolation(String),

    /// Error validasi input
    #[error("Input tidak valid: {0}")]
    Validation(String),
}

impl AppError {
    /// Membuat error I/O dengan konteks
    pub fn io(context: impl Into<String>, source: std::io::Error) -> Self {
        Self::Io {
            context: context.into(),
            source,
        }
    }

    /// Apakah error ini bersifat fatal (perlu shutdown)
    pub fn is_fatal(&self) -> bool {
        matches!(
            self,
            AppError::IntegrityViolation(_)
                | AppError::InvalidStateTransition { .. }
        )
    }

    /// Apakah error ini bisa di-retry
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            AppError::Process(_)
                | AppError::Channel(_)
                | AppError::Timeout { .. }
        )
    }
}

impl From<std::io::Error> for AppError {
    fn from(e: std::io::Error) -> Self {
        AppError::io("operasi I/O", e)
    }
}

impl From<serde_json::Error> for AppError {
    fn from(e: serde_json::Error) -> Self {
        AppError::Serialization(e.to_string())
    }
}

impl From<toml::de::Error> for AppError {
    fn from(e: toml::de::Error) -> Self {
        AppError::Serialization(format!("TOML: {e}"))
    }
}

impl From<serde_yaml::Error> for AppError {
    fn from(e: serde_yaml::Error) -> Self {
        AppError::Serialization(format!("YAML: {e}"))
    }
}

/// Tipe Result standar untuk seluruh aplikasi
pub type AppResult<T> = Result<T, AppError>;
