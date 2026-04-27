//! SeaORM Models for App Blocker
//! Version: 1.2.0

pub mod blacklist;
pub mod blacklist_path;
pub mod blacklist_process;
pub mod config;
pub mod log_entity;
pub mod schedule;
pub mod schedule_rule;
pub mod user;
pub mod whitelist;

// Re-export entities for easier access
pub use blacklist::BlacklistEntity;
pub use blacklist_path::BlacklistPathEntity;
pub use blacklist_process::BlacklistProcessEntity;
pub use config::Entity as ConfigEntity;
pub use config::Model as Config;
pub use log_entity::audit_log::Entity as AuditLogEntity;
pub use log_entity::LogEntity;
pub use schedule::ScheduleEntity;
pub use schedule_rule::ScheduleRuleEntity;
pub use user::Entity as UserEntity;
pub use user::Model as User;
pub use whitelist::Entity as WhitelistEntity;
pub use whitelist::Model as Whitelist;
