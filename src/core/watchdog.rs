/// Watchdog thread - memantau kesehatan semua komponen dan restart jika perlu.
/// Menggunakan dedicated heartbeat channel untuk menerima heartbeat dari komponen.
use crate::core::events::{AppEvent, ComponentId};
use crate::core::state::StateManager;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::Sender;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tracing::{debug, error, info, warn};

/// Kirim heartbeat untuk watchdog.
/// Dipanggil oleh Engine/Monitor untuk memberi tahu watchdog masih hidup.
/// Ini memperbaiki masalah sebelumnya: heartbeat dikirim via event channel tapi
/// watchdog tidak pernah menerimanya (false positive "thread mati").
pub fn send_watchdog_heartbeat(component: ComponentId) {
    // Heartbeat akan diterima via static channel di watchdog
    if let Some(tx) = HEARTBEAT_TX.get() {
        let _ = tx.send(component);
    }
}

// Static channel - sekali saja saat watchdog dimulai
static HEARTBEAT_TX: std::sync::OnceLock<std::sync::mpsc::Sender<ComponentId>> =
    std::sync::OnceLock::new();

#[derive(Debug)]
struct ComponentHealth {
    last_heartbeat: Instant,
    missed_count: u32,
    restart_count: u32,
}

impl ComponentHealth {
    fn new() -> Self {
        Self {
            last_heartbeat: Instant::now(),
            missed_count: 0,
            restart_count: 0,
        }
    }

    fn record_heartbeat(&mut self) {
        self.last_heartbeat = Instant::now();
        self.missed_count = 0;
    }

    /// Check jika heartbeat belum diterima dalam timeout yang ditentukan.
    /// Mengembalikan false jika timeout sudah exceed batas max_missed.
    fn check(&mut self, timeout: Duration, max_missed: u32) -> bool {
        if self.last_heartbeat.elapsed() > timeout {
            self.missed_count += 1;
            self.missed_count < max_missed
        } else {
            // Reset missed_count jika heartbeat masih diterima dalam timeout
            self.missed_count = 0;
            true
        }
    }
}

/// Watchdog yang mendengarkan heartbeat dari komponen.
/// Sebelumnya, watchdog hanya check timestamp tanpa menerima event heartbeat,
/// menyebabkan false positive "thread mati".
pub struct WatchdogThread {
    /// Channel untuk mengirim event (ke event handler utama)
    event_tx: Sender<AppEvent>,
    /// Channel untuk menerima heartbeat dari komponen (Engine, Monitor)
    heartbeat_rx: std::sync::mpsc::Receiver<ComponentId>,
    state_mgr: Arc<StateManager>,
    health: HashMap<ComponentId, ComponentHealth>,
    hb_interval: Duration,
    max_missed: u32,
    max_restart: u32,
}

impl WatchdogThread {
    pub fn new(
        event_tx: Sender<AppEvent>,
        state_mgr: Arc<StateManager>,
        hb_interval_ms: u64,
        max_missed: u32,
        max_restart: u32,
    ) -> Self {
        // Buat channel untuk heartbeat
        let (hb_tx, hb_rx) = std::sync::mpsc::channel();
        // Simpan tx ke static agar komponen bisa kirim heartbeat - perbaikan utama
        let _ = HEARTBEAT_TX.set(hb_tx);

        let mut health = HashMap::new();
        health.insert(ComponentId::Monitor, ComponentHealth::new());
        health.insert(ComponentId::Engine, ComponentHealth::new());

        Self {
            event_tx,
            heartbeat_rx: hb_rx,
            state_mgr,
            health,
            hb_interval: Duration::from_millis(hb_interval_ms),
            max_missed,
            max_restart,
        }
    }

    /// Buat watchdog dengan heartbeat channel yang bisa dishare.
    /// Mengembalikan tuple (watchdog, heartbeat_sender).
    pub fn new_with_hb_channel(
        event_tx: Sender<AppEvent>,
        state_mgr: Arc<StateManager>,
        hb_interval_ms: u64,
        max_missed: u32,
        max_restart: u32,
    ) -> (Self, std::sync::mpsc::Sender<ComponentId>) {
        let (hb_tx, hb_rx) = std::sync::mpsc::channel();

        let mut health = HashMap::new();
        health.insert(ComponentId::Monitor, ComponentHealth::new());
        health.insert(ComponentId::Engine, ComponentHealth::new());

        let wd = Self {
            event_tx,
            heartbeat_rx: hb_rx,
            state_mgr,
            health,
            hb_interval: Duration::from_millis(hb_interval_ms),
            max_missed,
            max_restart,
        };

        (wd, hb_tx)
    }

    pub fn run(mut self, shutdown_flag: Arc<AtomicBool>) {
        info!("Watchdog thread dimulai");
        // Initial delay diperbesar agar channel HEARTBEAT_TX sempat terinisialisasi
        // Sebelum threads lain mulai kirim heartbeat.
        // Juga kasih waktu Engine/Monitor untuk start loop mereka.
        std::thread::sleep(Duration::from_secs(12));

        loop {
            if shutdown_flag.load(Ordering::SeqCst) {
                break;
            }

            // Coba terima heartbeat dengan timeout non-blocking
            // Ini adalah perbaikan utama: sebelumnya watchdog tidak pernah menerima heartbeat
            while let Ok(component) = self.heartbeat_rx.try_recv() {
                if let Some(h) = self.health.get_mut(&component) {
                    h.record_heartbeat();
                    debug!(component = %component, "Heartbeat diterima");
                }
            }

            let timeout = self.hb_interval * (self.max_missed + 1);
            let max_r = self.max_restart;
            let mut dead: Vec<ComponentId> = Vec::new();

            for (comp, health) in self.health.iter_mut() {
                if !health.check(timeout, self.max_missed) {
                    warn!(
                        component = %comp,
                        missed    = health.missed_count,
                        restarts  = health.restart_count,
                        "Komponen tidak responsif"
                    );
                    dead.push(comp.clone());
                }
            }

            for comp in dead {
                // Log state sebelum restart
                if let Ok(state) = self.state_mgr.current_state() {
                    debug!(component = %comp, state = ?state, "State sebelum restart");
                }

                let restarts = self.health.get(&comp).map(|h| h.restart_count).unwrap_or(0);
                if restarts >= max_r {
                    error!(component = %comp, "Gagal restart maks kali, masuk SafeMode");
                    let _ = self.event_tx.send(AppEvent::EnterSafeMode {
                        reason: format!("{comp}_gagal_restart"),
                    });
                } else {
                    let _ = self.event_tx.send(AppEvent::ThreadDied {
                        component: comp.clone(),
                        reason: "missed_heartbeat".to_string(),
                    });
                    if let Some(h) = self.health.get_mut(&comp) {
                        h.restart_count += 1;
                        h.missed_count = 0;
                        h.last_heartbeat = Instant::now();
                    }
                }
            }

            std::thread::sleep(self.hb_interval);
        }

        info!("Watchdog thread selesai");
    }
}
