/// Sistem event utama menggunakan enum type-safe (bukan string-based).
/// Semua komunikasi antar thread melalui event ini.
use crate::system::process::ProcessInfo;
use chrono::{DateTime, Utc};
use uuid::Uuid;

/// Event utama yang mengalir dari monitor → engine → UI
#[derive(Debug, Clone)]
pub enum AppEvent {
    /// Proses terlarang terdeteksi
    ProcessDetected {
        trace_id: Uuid,
        info: ProcessInfo,
        score: u32,
        detected_at: DateTime<Utc>,
    },

    /// Proses berhasil diblokir/dihentikan
    ProcessBlocked {
        trace_id: Uuid,
        pid: u32,
        name: String,
        killed_at: DateTime<Utc>,
    },

    /// Proses gagal diblokir (non-fatal)
    ProcessBlockFailed {
        trace_id: Uuid,
        pid: u32,
        name: String,
        reason: String,
    },

    /// Minta tampilkan overlay
    OverlayRequested {
        trace_id: Uuid,
        info: ProcessInfo,
        triggered_at: DateTime<Utc>,
    },

    /// Autentikasi berhasil, minta unlock
    UnlockSuccess {
        trace_id: Uuid,
        username: String,
        unlocked_at: DateTime<Utc>,
    },

    /// Autentikasi gagal
    UnlockFailed {
        trace_id: Uuid,
        attempts: u32,
        max_attempts: u32,
    },

    /// Permintaan shutdown dari CLI atau sinyal OS
    ShutdownRequested {
        reason: String,
    },

    /// Emergency unlock via shortcut khusus
    EmergencyUnlock {
        trace_id: Uuid,
    },

    /// Permintaan masuk safe mode
    EnterSafeMode {
        reason: String,
    },

    /// Heartbeat dari komponen - digunakan watchdog
    Heartbeat {
        component: ComponentId,
    },

    /// Notifikasi thread mati
    ThreadDied {
        component: ComponentId,
        reason: String,
    },

    /// Config berhasil di-reload
    ConfigReloaded,

    /// Flag disable darurat terdeteksi
    DisableFlagDetected,
}

impl AppEvent {
    /// Buat trace_id baru untuk event baru
    pub fn new_trace_id() -> Uuid {
        Uuid::new_v4()
    }

    /// Nama event untuk logging
    pub fn name(&self) -> &'static str {
        match self {
            AppEvent::ProcessDetected { .. } => "ProcessDetected",
            AppEvent::ProcessBlocked { .. } => "ProcessBlocked",
            AppEvent::ProcessBlockFailed { .. } => "ProcessBlockFailed",
            AppEvent::OverlayRequested { .. } => "OverlayRequested",
            AppEvent::UnlockSuccess { .. } => "UnlockSuccess",
            AppEvent::UnlockFailed { .. } => "UnlockFailed",
            AppEvent::ShutdownRequested { .. } => "ShutdownRequested",
            AppEvent::EmergencyUnlock { .. } => "EmergencyUnlock",
            AppEvent::EnterSafeMode { .. } => "EnterSafeMode",
            AppEvent::Heartbeat { .. } => "Heartbeat",
            AppEvent::ThreadDied { .. } => "ThreadDied",
            AppEvent::ConfigReloaded => "ConfigReloaded",
            AppEvent::DisableFlagDetected => "DisableFlagDetected",
        }
    }

    /// Apakah event ini bersifat kritis dan harus diproses segera
    pub fn is_critical(&self) -> bool {
        matches!(
            self,
            AppEvent::ShutdownRequested { .. }
                | AppEvent::EmergencyUnlock { .. }
                | AppEvent::EnterSafeMode { .. }
                | AppEvent::DisableFlagDetected
        )
    }
}

/// Identifikasi komponen sistem
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ComponentId {
    Monitor,
    Engine,
    UiOverlay,
    Watchdog,
    ConfigWatcher,
}

impl std::fmt::Display for ComponentId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ComponentId::Monitor => write!(f, "Monitor"),
            ComponentId::Engine => write!(f, "Engine"),
            ComponentId::UiOverlay => write!(f, "UiOverlay"),
            ComponentId::Watchdog => write!(f, "Watchdog"),
            ComponentId::ConfigWatcher => write!(f, "ConfigWatcher"),
        }
    }
}
