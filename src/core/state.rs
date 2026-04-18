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
}
