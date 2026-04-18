/// Sistem metrik runtime untuk observability.
use serde::Serialize;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

pub struct AppMetrics {
    pub processes_scanned:  AtomicU64,
    pub processes_killed:   AtomicU64,
    pub overlays_triggered: AtomicU64,
    pub auth_attempts:      AtomicU64,
    pub auth_successes:     AtomicU64,
    pub error_count:        AtomicU64,
    pub started_at:         u64,
}

impl AppMetrics {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            processes_scanned:  AtomicU64::new(0),
            processes_killed:   AtomicU64::new(0),
            overlays_triggered: AtomicU64::new(0),
            auth_attempts:      AtomicU64::new(0),
            auth_successes:     AtomicU64::new(0),
            error_count:        AtomicU64::new(0),
            started_at: crate::utils::time::now_utc().timestamp() as u64,
        })
    }

    pub fn inc_scanned(&self)      { self.processes_scanned.fetch_add(1,  Ordering::Relaxed); }
    pub fn inc_killed(&self)       { self.processes_killed.fetch_add(1,   Ordering::Relaxed); }
    pub fn inc_overlay(&self)      { self.overlays_triggered.fetch_add(1, Ordering::Relaxed); }
    pub fn inc_auth_attempt(&self) { self.auth_attempts.fetch_add(1,      Ordering::Relaxed); }
    pub fn inc_auth_success(&self) { self.auth_successes.fetch_add(1,     Ordering::Relaxed); }
    pub fn inc_error(&self)        { self.error_count.fetch_add(1,        Ordering::Relaxed); }

    pub fn snapshot(&self) -> MetricsSnapshot {
        let now = crate::utils::time::now_utc().timestamp() as u64;
        MetricsSnapshot {
            uptime_seconds:     now.saturating_sub(self.started_at),
            processes_scanned:  self.processes_scanned.load(Ordering::Relaxed),
            processes_killed:   self.processes_killed.load(Ordering::Relaxed),
            overlays_triggered: self.overlays_triggered.load(Ordering::Relaxed),
            auth_attempts:      self.auth_attempts.load(Ordering::Relaxed),
            auth_successes:     self.auth_successes.load(Ordering::Relaxed),
            error_count:        self.error_count.load(Ordering::Relaxed),
        }
    }

    pub fn log_snapshot(&self) {
        let s = self.snapshot();
        tracing::info!(
            uptime     = s.uptime_seconds,
            scanned    = s.processes_scanned,
            killed     = s.processes_killed,
            overlays   = s.overlays_triggered,
            auth_ok    = s.auth_successes,
            errors     = s.error_count,
            "Metrik sistem"
        );
    }
}

#[derive(Debug, Serialize)]
pub struct MetricsSnapshot {
    pub uptime_seconds:     u64,
    pub processes_scanned:  u64,
    pub processes_killed:   u64,
    pub overlays_triggered: u64,
    pub auth_attempts:      u64,
    pub auth_successes:     u64,
    pub error_count:        u64,
}
