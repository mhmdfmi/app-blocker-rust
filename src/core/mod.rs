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
