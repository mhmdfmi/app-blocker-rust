//! Repository Module
//! Version: 1.2.0
//! Abstraction layer untuk akses database

pub mod blacklist_repo;
pub mod config_repo;
pub mod log_repo;
pub mod schedule_repo;
pub mod user_repo;
pub mod whitelist_repo;

// Re-export repositories
pub use blacklist_repo::BlacklistRepository;
pub use config_repo::ConfigRepository;
pub use log_repo::{LogRepository, PaginatedLogs};
pub use schedule_repo::ScheduleRepository;
pub use user_repo::UserRepository;
pub use whitelist_repo::WhitelistRepository;

// Re-export models
pub use crate::models::blacklist::Model as Blacklist;
pub use crate::models::config::Model as Config;
pub use crate::models::log_entity::Model as Log;
pub use crate::models::schedule::Model as Schedule;
pub use crate::models::user::Model as User;
pub use crate::models::whitelist::Model as Whitelist;

use crate::utils::error::AppResult;
use sea_orm::DatabaseConnection;

// ============================================================================
// Repository Traits - Define abstraction interface for data access
// ============================================================================

/// Base trait for all repositories
pub trait Repository<T>: Send + Sync {
    /// Get the database connection
    fn db(&self) -> &DatabaseConnection;
}

/// User Repository Trait
pub trait UserRepositoryTrait: Repository<User> {
    /// Find user by username
    fn find_by_username(&self, username: &str) -> AppResult<Option<User>>;

    /// Find user by ID
    fn find_by_id(&self, id: i32) -> AppResult<Option<User>>;

    /// Find all users
    fn find_all(&self) -> AppResult<Vec<User>>;

    /// Create new user
    fn create(&self, username: &str, password_hash: &str, role: &str) -> AppResult<User>;

    /// Update user
    fn update(
        &self,
        id: i32,
        username: &str,
        password_hash: Option<&str>,
        role: &str,
    ) -> AppResult<User>;

    /// Delete user
    fn delete(&self, id: i32) -> AppResult<()>;
}

/// Config Repository Trait
pub trait ConfigRepositoryTrait: Repository<Config> {
    /// Get config value by key
    fn get_value(&self, key: &str) -> AppResult<Option<String>>;

    /// Set config value
    fn set_value(&self, key: &str, value: &str, description: Option<&str>) -> AppResult<()>;

    /// Get all configs
    fn find_all(&self) -> AppResult<Vec<Config>>;

    /// Delete config
    fn delete(&self, key: &str) -> AppResult<()>;
}

/// Blacklist Repository Trait
pub trait BlacklistRepositoryTrait: Repository<Blacklist> {
    /// Find all blacklists
    fn find_all(&self) -> AppResult<Vec<Blacklist>>;

    /// Find blacklist by ID
    fn find_by_id(&self, id: i32) -> AppResult<Option<Blacklist>>;

    /// Find enabled blacklists only
    fn find_enabled(&self) -> AppResult<Vec<Blacklist>>;

    /// Create new blacklist
    fn create(&self, name: &str, description: Option<&str>, enabled: bool) -> AppResult<Blacklist>;

    /// Update blacklist
    fn update(
        &self,
        id: i32,
        name: &str,
        description: Option<&str>,
        enabled: bool,
    ) -> AppResult<Blacklist>;

    /// Delete blacklist
    fn delete(&self, id: i32) -> AppResult<()>;

    /// Get processes for blacklist
    fn get_processes(
        &self,
        blacklist_id: i32,
    ) -> AppResult<Vec<crate::models::blacklist_process::Model>>;

    /// Add process to blacklist
    fn add_process(&self, blacklist_id: i32, process_name: &str) -> AppResult<()>;

    /// Remove process from blacklist
    fn remove_process(&self, blacklist_id: i32, process_name: &str) -> AppResult<()>;

    /// Get paths for blacklist
    fn get_paths(&self, blacklist_id: i32) -> AppResult<Vec<crate::models::blacklist_path::Model>>;

    /// Add path to blacklist
    fn add_path(&self, blacklist_id: i32, path: &str) -> AppResult<()>;

    /// Remove path from blacklist
    fn remove_path(&self, blacklist_id: i32, path: &str) -> AppResult<()>;
}

/// Whitelist Repository Trait
pub trait WhitelistRepositoryTrait: Repository<Whitelist> {
    /// Find all whitelist entries
    fn find_all(&self) -> AppResult<Vec<Whitelist>>;

    /// Find whitelist by process name
    fn find_by_process(&self, process_name: &str) -> AppResult<Option<Whitelist>>;

    /// Create new whitelist entry
    fn create(&self, process_name: &str) -> AppResult<Whitelist>;

    /// Delete whitelist entry
    fn delete(&self, id: i32) -> AppResult<()>;

    /// Check if process is whitelisted
    fn is_whitelisted(&self, process_name: &str) -> AppResult<bool>;
}

pub use schedule_repo::ScheduleWithRules;

/// Schedule Repository Trait
pub trait ScheduleRepositoryTrait: Repository<Schedule> {
    /// Get global schedule
    fn get_global(&self) -> AppResult<Option<Schedule>>;

    /// Get schedule with rules
    fn get_global_with_rules(&self) -> AppResult<Option<ScheduleWithRules>>;

    /// Create or update schedule
    fn upsert(&self, enabled: bool, timezone: &str) -> AppResult<Schedule>;

    /// Add schedule rule
    fn add_rule(
        &self,
        schedule_id: i32,
        days: &[&str],
        start_time: &str,
        end_time: &str,
        action: &str,
    ) -> AppResult<()>;

    /// Remove schedule rule
    fn remove_rule(&self, rule_id: i32) -> AppResult<()>;

    /// Get all rules for schedule
    fn get_rules(&self, schedule_id: i32) -> AppResult<Vec<crate::models::schedule_rule::Model>>;
}

/// Log Repository Trait
pub trait LogRepositoryTrait: Repository<Log> {
    /// Find all logs
    fn find_all(&self) -> AppResult<Vec<Log>>;

    /// Find log by ID
    fn find_by_id(&self, id: i32) -> AppResult<Option<Log>>;

    /// Find logs by action
    fn find_by_action(&self, action: &str) -> AppResult<Vec<Log>>;

    /// Get blocked logs
    fn get_blocked(&self) -> AppResult<Vec<Log>>;

    /// Get allowed logs
    fn get_allowed(&self) -> AppResult<Vec<Log>>;

    /// Create new log entry
    fn create(&self, params: crate::repository::log_repo::CreateLogParams) -> AppResult<Log>;

    /// Log a blocked process
    fn log_blocked(&self, process_name: &str, reason: &str, score: Option<i32>) -> AppResult<Log>;

    /// Log an allowed process
    fn log_allowed(&self, process_name: &str, reason: Option<&str>) -> AppResult<Log>;

    /// Delete log
    fn delete(&self, id: i32) -> AppResult<()>;

    /// Clear all logs
    fn clear_all(&self) -> AppResult<()>;

    /// Clear logs older than N days
    fn clear_older_than_days(&self, days: i64) -> AppResult<u64>;

    /// Paginate logs
    fn paginate(&self, page: i64, page_size: i64) -> AppResult<PaginatedLogs>;
}
