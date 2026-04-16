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
        }
    }
}
