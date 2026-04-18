<<<<<<< HEAD
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
=======
//! Events Module
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use parking_lot::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AppEvent {
    ProcessDetected(ProcessInfo),
    ProcessBlocked(ProcessInfo),
    OverlayRequested(OverlayRequest),
    UnlockSuccess { username: String, trace_id: String },
    UnlockFailed { attempt: u32, reason: String, trace_id: String },
    ShutdownRequested,
    EmergencyUnlock,
    ProcessTerminated { pid: u32, name: String },
    Error(ErrorEvent),
    ThreadUnresponsive { thread_name: String },
    RecoveryComplete,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    pub path: Option<String>,
    pub username: Option<String>,
    pub timestamp: String,
    pub suspicious_score: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverlayRequest {
    pub process_info: ProcessInfo,
    pub trace_id: String,
    pub is_simulation: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorEvent {
    pub error_type: ErrorType,
    pub message: String,
    pub trace_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ErrorType {
    ProcessKillFailure,
    OverlayCrash,
    ThreadPanic,
    DeadlockDetected,
    ChannelDisconnected,
    StateCorruption,
    ConfigError,
    AuthError,
}

impl AppEvent {
    pub fn trace_id(&self) -> Option<String> {
        match self {
            AppEvent::ProcessDetected(info) => Some(info.timestamp.clone()),
            AppEvent::ProcessBlocked(info) => Some(info.timestamp.clone()),
            AppEvent::OverlayRequested(req) => Some(req.trace_id.clone()),
            AppEvent::UnlockSuccess { trace_id, .. } => Some(trace_id.clone()),
            AppEvent::UnlockFailed { trace_id, .. } => Some(trace_id.clone()),
            AppEvent::Error(err) => err.trace_id.clone(),
            _ => None,
>>>>>>> bce0345919f371d153ccb843f2ddbfb5e8695c5f
        }
    }
}
