/// Modul sistem - process, user, hooks, service, student mode
pub mod hooks;
pub mod process;
pub mod service;
pub mod student_mode;
pub mod user;

pub use process::{ProcessInfo, ProcessService, WindowsProcessService};
pub use service::{
    acquire_single_instance_lock, create_disable_flag, is_disable_flag_active, SingleInstanceGuard,
};
pub use student_mode::{apply_restrictions, restore_restrictions, StudentModeConfig};
pub use user::{get_computer_name, get_current_session, get_username, UserSession};
