//! State Management Module
//! 
//! Mengelola state aplikasi dengan thread-safe menggunakan RwLock.
//!
//! State machine states:
//! - Monitoring: Pemrosesan normal, memonitor proses
//! - Blocking: Proses terlarang terdeteksi, akan diblokir
//! - Locked: Overlay ditampilkan, menunggu unlock
//! - Recovering: Cleanup setelah unlock
//! - SafeMode: Mode aman, blocking dinonaktifkan

use serde::{Deserialize, Serialize};
use parking_lot::RwLock;
use std::sync::Arc;
use crate::core::events::AppEvent;

/// State utama aplikasi
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AppState {
    /// Status state machine saat ini
    pub current_state: State,
    /// Apakah dalam kondisi terlock
    pub is_locked: bool,
    /// Apakah aman untuk memblokir proses
    pub is_safe_to_block: bool,
    /// Apakah cleanup sedang berlangsung
    pub cleanup_in_progress: bool,
    /// Apakah blocking dinonaktifkan
    pub blocking_disabled: bool,
    /// Overlay aktif
    pub overlay_active: bool,
    /// Jumlah percobaan unlock gagal
    pub failed_unlock_attempts: u32,
    /// ID proses yang diblokir terakhir
    pub last_blocked_pid: Option<u32>,
    /// Nama proses yang diblokir terakhir
    pub last_blocked_name: Option<String>,
    /// Trace ID untuk korelasi log
    pub trace_id: Option<String>,
}

/// State machine states
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum State {
    #[default]
    Monitoring,
    Blocking,
    Locked,
    Recovering,
    SafeMode,
}

impl State {
    /// Validasi apakah transisi diperbolehkan
    pub fn can_transition_to(&self, target: &State) -> bool {
        match (self, target) {
            (State::Monitoring, State::Blocking) => true,
            (State::Blocking, State::Locked) => true,
            (State::Locked, State::Recovering) => true,
            (State::Recovering, State::Monitoring) => true,
            (_, State::SafeMode) => true, // Semua state bisa ke SafeMode
            _ => false,
        }
    }
    
    /// Validasi invarian state
    pub fn validate_invariants(&self, state: &AppState) -> bool {
        match self {
            State::Monitoring => {
                !state.is_locked && !state.is_safe_to_block
            }
            State::Blocking => {
                !state.is_safe_to_block && !state.is_locked
            }
            State::Locked => {
                state.is_locked && state.overlay_active
            }
            State::Recovering => {
                !state.is_locked && state.cleanup_in_progress
            }
            State::SafeMode => state.blocking_disabled,
        }
    }
}

/// Thread-safe app state wrapper
pub type SharedState = Arc<RwLock<AppState>>;

impl AppState {
    /// Buat state baru
    pub fn new() -> Self {
        Self {
            current_state: State::Monitoring,
            ..Default::default()
        }
    }
    
    /// Update state dengan validasi
    pub fn transition_to(&mut self, new_state: State, event: &AppEvent) -> Result<(), StateError> {
        let old_state = self.current_state;
        
        if !old_state.can_transition_to(&new_state) {
            return Err(StateError::InvalidTransition {
                from: format!("{:?}", old_state),
                to: format!("{:?}", new_state),
                event: format!("{:?}", event),
            });
        }
        
        self.current_state = new_state;
        
        // Update flag berdasarkan state
        match new_state {
            State::Locked => {
                self.is_locked = true;
                self.overlay_active = true;
            }
            State::Recovering => {
                self.cleanup_in_progress = true;
            }
            State::SafeMode => {
                self.blocking_disabled = true;
            }
            _ => {}
        }
        
        tracing::info!(
            "State transition: {:?} -> {:?} via {:?}",
            old_state, new_state, event
        );
        
        Ok(())
    }
    
    /// Reset ke kondisi aman
    pub fn reset(&mut self) {
        self.current_state = State::Monitoring;
        self.is_locked = false;
        self.is_safe_to_block = false;
        self.cleanup_in_progress = false;
        self.overlay_active = false;
        self.trace_id = None;
    }
    
    /// Enable safe mode
    pub fn enter_safe_mode(&mut self) {
        self.current_state = State::SafeMode;
        self.blocking_disabled = true;
        self.is_locked = false;
        self.overlay_active = false;
    }
}

/// State transition error
#[derive(Debug, thiserror::Error)]
pub enum StateError {
    #[error("Invalid state transition from {from} to {to} via {event}")]
    InvalidTransition {
        from: String,
        to: String,
        event: String,
    },
    
    #[error("State invariant violation: {0}")]
    InvariantViolation(String),
}
