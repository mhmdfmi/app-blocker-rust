<<<<<<< HEAD
/// State machine sebagai pusat kontrol sistem.
/// Validasi ketat sebelum setiap transisi.
use crate::utils::error::{AppError, AppResult};
use chrono::{DateTime, Utc};
use std::sync::{Arc, RwLock};
use tracing::{info, warn};

/// State utama sistem
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum AppState {
    /// Memantau proses, tidak ada blokir aktif
    Monitoring,
    /// Proses terdeteksi, sedang memproses blokir
    Blocking,
    /// Overlay aktif, menunggu autentikasi admin
    Locked,
    /// Autentikasi berhasil, sedang membersihkan state
    Recovering,
    /// Mode aman - monitoring saja, tidak ada blokir
    SafeMode,
}

impl std::fmt::Display for AppState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppState::Monitoring => write!(f, "Monitoring"),
            AppState::Blocking => write!(f, "Blocking"),
            AppState::Locked => write!(f, "Locked"),
            AppState::Recovering => write!(f, "Recovering"),
            AppState::SafeMode => write!(f, "SafeMode"),
        }
    }
}

impl AppState {
    /// Periksa apakah transisi ke state berikutnya diizinkan
    pub fn can_transition_to(&self, target: &AppState) -> bool {
        match (self, target) {
            // Transisi normal
            (AppState::Monitoring, AppState::Blocking) => true,
            (AppState::Blocking, AppState::Locked) => true,
            (AppState::Locked, AppState::Recovering) => true,
            (AppState::Recovering, AppState::Monitoring) => true,
            // Semua state bisa masuk SafeMode
            (_, AppState::SafeMode) => true,
            // SafeMode bisa kembali ke Monitoring (manual recovery)
            (AppState::SafeMode, AppState::Monitoring) => true,
            // Semua transisi lain dilarang
            _ => false,
        }
    }

    /// Apakah state ini mengizinkan blokir
    pub fn allows_blocking(&self) -> bool {
        matches!(self, AppState::Monitoring)
    }

    /// Apakah overlay harus aktif di state ini
    pub fn requires_overlay(&self) -> bool {
        matches!(self, AppState::Locked)
    }

    /// Apakah state ini adalah mode darurat
    pub fn is_safe_mode(&self) -> bool {
        matches!(self, AppState::SafeMode)
    }
}

/// Data runtime yang terkait dengan state saat ini
#[derive(Debug, Clone, Default)]
pub struct StateData {
    /// Apakah overlay sedang ditampilkan
    pub overlay_active: bool,
    /// Apakah proses aman untuk diblokir
    pub is_safe_to_block: bool,
    /// PID proses yang sedang diblokir
    pub blocked_pid: Option<u32>,
    /// Nama proses yang diblokir
    pub blocked_name: Option<String>,
    /// Waktu masuk state saat ini
    pub state_entered_at: Option<DateTime<Utc>>,
    /// Jumlah error berturut-turut
    pub consecutive_errors: u32,
    /// Apakah blokir dinonaktifkan
    pub blocking_disabled: bool,
}

/// State manager thread-safe
pub struct StateManager {
    current: Arc<RwLock<AppState>>,
    data: Arc<RwLock<StateData>>,
    /// History transisi untuk audit
    history: Arc<RwLock<Vec<TransitionRecord>>>,
}

/// Catatan transisi state
#[derive(Debug, Clone)]
pub struct TransitionRecord {
    pub from: String,
    pub to: String,
    pub timestamp: DateTime<Utc>,
    pub reason: String,
}

impl StateManager {
    /// Buat StateManager baru dimulai dari Monitoring
    pub fn new() -> Self {
        Self {
            current: Arc::new(RwLock::new(AppState::Monitoring)),
            data: Arc::new(RwLock::new(StateData {
                state_entered_at: Some(Utc::now()),
                ..Default::default()
            })),
            history: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Baca state saat ini tanpa lock lama
    pub fn current_state(&self) -> AppResult<AppState> {
        self.current
            .read()
            .map(|s| s.clone())
            .map_err(|e| AppError::System(format!("Gagal baca state: {e}")))
    }

    /// Transisi ke state baru dengan validasi
    pub fn transition_to(&self, target: AppState, reason: &str) -> AppResult<()> {
        let current = self.current_state()?;

        if !current.can_transition_to(&target) {
            return Err(AppError::InvalidStateTransition {
                from: current.to_string(),
                to: target.to_string(),
            });
        }

        // Rekam history
        {
            let record = TransitionRecord {
                from: current.to_string(),
                to: target.to_string(),
                timestamp: Utc::now(),
                reason: reason.to_string(),
            };
            if let Ok(mut history) = self.history.write() {
                history.push(record);
                // Simpan maksimal 100 history
                if history.len() > 100 {
                    history.remove(0);
                }
            }
        }

        info!(
            from = %current,
            to = %target,
            reason,
            "Transisi state"
        );

        // Update state
        let mut state_guard = self.current
            .write()
            .map_err(|e| AppError::System(format!("Gagal write state: {e}")))?;
        *state_guard = target;
        drop(state_guard);

        // Update timestamp state_entered_at
        if let Ok(mut data) = self.data.write() {
            data.state_entered_at = Some(Utc::now());
        }

        Ok(())
    }

    /// Baca data state saat ini
    pub fn read_data<F, T>(&self, f: F) -> AppResult<T>
    where
        F: FnOnce(&StateData) -> T,
    {
        self.data
            .read()
            .map(|d| f(&d))
            .map_err(|e| AppError::System(format!("Gagal baca state data: {e}")))
    }

    /// Update data state
    pub fn update_data<F>(&self, f: F) -> AppResult<()>
    where
        F: FnOnce(&mut StateData),
    {
        self.data
            .write()
            .map(|mut d| f(&mut d))
            .map_err(|e| AppError::System(format!("Gagal update state data: {e}")))
    }

    /// Reset state data ke kondisi bersih
    pub fn reset_data(&self) -> AppResult<()> {
        self.update_data(|d| {
            d.overlay_active = false;
            d.is_safe_to_block = false;
            d.blocked_pid = None;
            d.blocked_name = None;
            d.consecutive_errors = 0;
            d.state_entered_at = Some(Utc::now());
        })
    }

    /// Dapatkan Arc state untuk berbagi dengan thread lain
    pub fn state_arc(&self) -> Arc<RwLock<AppState>> {
        Arc::clone(&self.current)
    }

    /// Masuk safe mode dengan paksa (bisa dari state manapun)
    pub fn force_safe_mode(&self, reason: &str) -> AppResult<()> {
        warn!(reason, "Paksa masuk SafeMode");

        let mut state_guard = self.current
            .write()
            .map_err(|e| AppError::System(format!("Gagal force safe mode: {e}")))?;
        *state_guard = AppState::SafeMode;
        drop(state_guard);

        self.update_data(|d| {
            d.blocking_disabled = true;
            d.state_entered_at = Some(Utc::now());
        })?;

        Ok(())
    }

    /// Ambil history transisi (untuk debug/audit)
    pub fn get_history(&self) -> Vec<TransitionRecord> {
        self.history
            .read()
            .map(|h| h.clone())
            .unwrap_or_default()
    }
}

impl Default for StateManager {
    fn default() -> Self {
        Self::new()
    }
=======
﻿//! State Management Module
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
>>>>>>> bce0345919f371d153ccb843f2ddbfb5e8695c5f
}
