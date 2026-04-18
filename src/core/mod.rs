<<<<<<< HEAD
/// Modul inti aplikasi - engine, monitor, state, events, watchdog, audit
pub mod audit;
pub mod engine;
pub mod events;
pub mod monitor;
pub mod state;
pub mod watchdog;

pub use audit::{audit, init_global_audit, AuditEntry, AuditEventKind, AuditWriter};
pub use engine::{AppEngine, OverlayCallback, OverlayRequest};
pub use events::{AppEvent, ComponentId};
pub use state::{AppState, StateData, StateManager};
pub use watchdog::WatchdogThread;
=======
﻿//! Core Module
//! 
//! Modul inti yang berisi state machine, engine, dan monitor.

pub mod engine;
pub mod monitor;
pub mod state;
pub mod events;
>>>>>>> bce0345919f371d153ccb843f2ddbfb5e8695c5f
