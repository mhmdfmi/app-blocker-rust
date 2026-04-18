/// Watchdog thread - memantau kesehatan semua komponen dan restart jika perlu.
use crate::core::events::{AppEvent, ComponentId};
use crate::core::state::StateManager;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::Sender;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tracing::{debug, error, info, warn};

#[derive(Debug)]
struct ComponentHealth {
    last_heartbeat: Instant,
    missed_count:   u32,
    restart_count:  u32,
}

impl ComponentHealth {
    fn new() -> Self {
        Self { last_heartbeat: Instant::now(), missed_count: 0, restart_count: 0 }
    }

    fn record_heartbeat(&mut self) {
        self.last_heartbeat = Instant::now();
        self.missed_count   = 0;
    }

    /// Kembalikan false jika missed terlalu banyak
    fn check(&mut self, timeout: Duration, max_missed: u32) -> bool {
        if self.last_heartbeat.elapsed() > timeout {
            self.missed_count += 1;
            self.missed_count < max_missed
        } else {
            true
        }
    }
}

pub struct WatchdogThread {
    event_tx:   Sender<AppEvent>,
    state_mgr:  Arc<StateManager>,
    health:     HashMap<ComponentId, ComponentHealth>,
    hb_interval: Duration,
    max_missed:  u32,
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
        let mut health = HashMap::new();
        health.insert(ComponentId::Monitor, ComponentHealth::new());
        health.insert(ComponentId::Engine,  ComponentHealth::new());

        Self {
            event_tx,
            state_mgr,
            health,
            hb_interval: Duration::from_millis(hb_interval_ms),
            max_missed,
            max_restart,
        }
    }

    pub fn record_heartbeat(&mut self, component: &ComponentId) {
        if let Some(h) = self.health.get_mut(component) {
            h.record_heartbeat();
            debug!(component = %component, "Heartbeat");
        }
    }

    pub fn run(mut self, shutdown_flag: Arc<AtomicBool>) {
        info!("Watchdog thread dimulai");
        // Delay awal agar thread lain sempat start
        std::thread::sleep(Duration::from_secs(5));

        loop {
            if shutdown_flag.load(Ordering::SeqCst) {
                break;
            }

            let timeout = self.hb_interval * (self.max_missed + 1);
            let max_r   = self.max_restart;
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
                        h.restart_count  += 1;
                        h.missed_count    = 0;
                        h.last_heartbeat  = Instant::now();
                    }
                }
            }

            std::thread::sleep(self.hb_interval);
        }

        info!("Watchdog thread selesai");
    }
}
