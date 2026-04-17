//! Overlay Module
use crate::core::events::{AppEvent, OverlayRequest};
use crate::security::auth::Authenticator;
use crate::utils::error::AppResult;
use parking_lot::Mutex;
use std::sync::Arc;
use std::thread;

pub struct OverlayManager {
    authenticator: Arc<Authenticator>,
    is_active: Arc<Mutex<bool>>,
    failsafe_timer: Arc<Mutex<Option<std::time::Instant>>>,
}

impl OverlayManager {
    pub fn new(authenticator: Arc<Authenticator>) -> Self {
        Self {
            authenticator,
            is_active: Arc::new(Mutex::new(false)),
            failsafe_timer: Arc::new(Mutex::new(None)),
        }
    }

    pub fn show_overlay(&self, request: OverlayRequest) -> AppResult<AppEvent> {
        tracing::info!("Showing overlay for process: {}", request.process_info.name);

        *self.is_active.lock() = true;
        *self.failsafe_timer.lock() = Some(std::time::Instant::now());

        if request.is_simulation {
            tracing::info!("[SIMULATED] Overlay displayed");
            *self.is_active.lock() = false;
            return Ok(AppEvent::UnlockSuccess {
                username: "simulation".to_string(),
                trace_id: request.trace_id.clone(),
            });
        }

        let result = self.create_overlay_window(&request);

        *self.is_active.lock() = false;

        result
    }

    fn create_overlay_window(&self, request: &OverlayRequest) -> AppResult<AppEvent> {
        // Simplified overlay - in production would use Win32 API
        tracing::debug!("Creating overlay window");

        // Block until unlock (simulated)
        thread::sleep(std::time::Duration::from_secs(2));

        tracing::info!("Overlay window created for: {}", request.process_info.name);

        Ok(AppEvent::UnlockSuccess {
            username: "[unlock_user]".to_string(),
            trace_id: request.trace_id.clone(),
        })
    }

    pub fn is_active(&self) -> bool {
        *self.is_active.lock()
    }

    pub fn check_failsafe(&self, timeout_minutes: u64) -> bool {
        if let Some(start) = *self.failsafe_timer.lock() {
            let elapsed = start.elapsed().as_secs() / 60;
            if elapsed >= timeout_minutes {
                tracing::warn!("Failsafe timeout reached!");
                return true;
            }
        }
        false
    }

    pub fn reset_failsafe(&self) {
        *self.failsafe_timer.lock() = None;
    }
}

pub struct OverlayThread {
    handle: Option<thread::JoinHandle<AppResult<AppEvent>>>,
}

impl OverlayThread {
    pub fn start(request: OverlayRequest, authenticator: Arc<Authenticator>) -> Self {
        let manager = OverlayManager::new(authenticator);
        let handle = thread::spawn(move || manager.show_overlay(request));
        Self {
            handle: Some(handle),
        }
    }

    pub fn join(self) -> AppResult<AppEvent> {
        if let Some(handle) = self.handle {
            handle.join().map_err(|_| {
                crate::utils::error::AppError::ThreadError("Overlay thread panicked".into())
            })?
        } else {
            Err(crate::utils::error::AppError::ThreadError(
                "No handle".into(),
            ))
        }
    }
}
