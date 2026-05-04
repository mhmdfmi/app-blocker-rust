//! Database Initialization, Migration and Seeder
//! Version: 1.2.1

use argon2::{
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use rand::rngs::OsRng;
use sea_orm::{ConnectionTrait, DatabaseConnection, DbErr, TransactionTrait};

use serde::Deserialize;
use std::path::Path;
use tracing::{error, info};

use crate::db::connection::create_db_pool;

/// Initialize database: create connection, run migrations, and seed data
///
/// # Arguments
/// * `db_path` - Path to the database file
///
/// # Returns
/// * `Result<DatabaseConnection, DbErr>` - Initialized database connection
pub async fn init_database(db_path: &Path) -> Result<DatabaseConnection, DbErr> {
    info!("Initializing database at: {:?}", db_path);

    // Step 1: Create database connection
    let db = create_db_pool(db_path).await?;

    // Step 2: Run migrations
    run_migrations(&db).await?;

    // Step 3: Seed default data from config/default.toml
    seed_default_data(&db).await?;

    info!("Database initialization complete");

    Ok(db)
}

/// Run database migrations (create tables)
///
/// # Arguments
/// * `db` - Database connection
///
/// # Returns
/// * `Result<(), DbErr>` - Error if migration fails
pub async fn run_migrations(db: &DatabaseConnection) -> Result<(), DbErr> {
    info!("Running database migrations...");

    // Create USERS table
    db.execute_unprepared(
        r#"
        CREATE TABLE IF NOT EXISTS users (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            username TEXT NOT NULL UNIQUE,
            password_hash TEXT NOT NULL,
            role TEXT NOT NULL DEFAULT 'user' CHECK(role IN ('admin', 'user')),
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            updated_at TEXT NOT NULL DEFAULT (datetime('now'))
        )
        "#,
    )
    .await
    .map_err(|e| {
        error!("Failed to create users table: {}", e);
        DbErr::Custom(format!("Failed to create users table: {}", e))
    })?;

    // Create CONFIGS table
    db.execute_unprepared(
        r#"
        CREATE TABLE IF NOT EXISTS configs (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL,
            description TEXT,
            updated_at TEXT NOT NULL DEFAULT (datetime('now'))
        )
        "#,
    )
    .await
    .map_err(|e| {
        error!("Failed to create configs table: {}", e);
        DbErr::Custom(format!("Failed to create configs table: {}", e))
    })?;

    // Create BLACKLIST table (updated with required fields: category, publisher, risk_level)
    db.execute_unprepared(
        r#"
        CREATE TABLE IF NOT EXISTS blacklist (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            description TEXT NOT NULL,
            category TEXT NOT NULL,
            publisher TEXT,
            risk_level TEXT NOT NULL DEFAULT 'medium' CHECK(risk_level IN ('low', 'medium', 'high', 'critical')),
            enabled INTEGER NOT NULL DEFAULT 1 CHECK(enabled IN (0, 1)),
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            updated_at TEXT NOT NULL DEFAULT (datetime('now'))
        )
        "#,
    )
    .await
    .map_err(|e| {
        error!("Failed to create blacklist table: {}", e);
        DbErr::Custom(format!("Failed to create blacklist table: {}", e))
    })?;

    // Create BLACKLIST_PROCESSES table
    db.execute_unprepared(
        r#"
        CREATE TABLE IF NOT EXISTS blacklist_processes (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            blacklist_id INTEGER NOT NULL,
            process_name TEXT NOT NULL,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            FOREIGN KEY (blacklist_id) REFERENCES blacklist(id) ON DELETE CASCADE
        )
        "#,
    )
    .await
    .map_err(|e| {
        error!("Failed to create blacklist_processes table: {}", e);
        DbErr::Custom(format!("Failed to create blacklist_processes table: {}", e))
    })?;

    // Create BLACKLIST_PATHS table
    db.execute_unprepared(
        r#"
        CREATE TABLE IF NOT EXISTS blacklist_paths (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            blacklist_id INTEGER NOT NULL,
            path TEXT NOT NULL,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            FOREIGN KEY (blacklist_id) REFERENCES blacklist(id) ON DELETE CASCADE
        )
        "#,
    )
    .await
    .map_err(|e| {
        error!("Failed to create blacklist_paths table: {}", e);
        DbErr::Custom(format!("Failed to create blacklist_paths table: {}", e))
    })?;

    // Create WHITELIST table
    db.execute_unprepared(
        r#"
        CREATE TABLE IF NOT EXISTS whitelist (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            process_name TEXT NOT NULL UNIQUE,
            description TEXT,
            enabled INTEGER NOT NULL DEFAULT 1 CHECK(enabled IN (0, 1)),
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            updated_at TEXT NOT NULL DEFAULT (datetime('now'))
        )
        "#,
    )
    .await
    .map_err(|e| {
        error!("Failed to create whitelist table: {}", e);
        DbErr::Custom(format!("Failed to create whitelist table: {}", e))
    })?;

    // Create SCHEDULE table
    db.execute_unprepared(
        r#"
        CREATE TABLE IF NOT EXISTS schedule (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            enabled INTEGER NOT NULL DEFAULT 1 CHECK(enabled IN (0, 1)),
            timezone TEXT NOT NULL DEFAULT 'Asia/Jakarta',
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            updated_at TEXT NOT NULL DEFAULT (datetime('now'))
        )
        "#,
    )
    .await
    .map_err(|e| {
        error!("Failed to create schedule table: {}", e);
        DbErr::Custom(format!("Failed to create schedule table: {}", e))
    })?;

    // Create SCHEDULE_RULES table
    db.execute_unprepared(
        r#"
        CREATE TABLE IF NOT EXISTS schedule_rules (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            schedule_id INTEGER NOT NULL,
            days TEXT NOT NULL,
            start_time TEXT NOT NULL,
            end_time TEXT NOT NULL,
            action TEXT NOT NULL,
            enabled INTEGER NOT NULL DEFAULT 1 CHECK(enabled IN (0, 1)),
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            updated_at TEXT NOT NULL DEFAULT (datetime('now')),
            FOREIGN KEY (schedule_id) REFERENCES schedule(id) ON DELETE CASCADE
        )
        "#,
    )
    .await
    .map_err(|e| {
        error!("Failed to create schedule_rules table: {}", e);
        DbErr::Custom(format!("Failed to create schedule_rules table: {}", e))
    })?;

    // Create LOGS table
    db.execute_unprepared(
        r#"
        CREATE TABLE IF NOT EXISTS logs (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            timestamp TEXT NOT NULL DEFAULT (datetime('now')),
            process_name TEXT NOT NULL,
            process_path TEXT,
            action TEXT NOT NULL CHECK(action IN ('blocked', 'allowed', 'warning', 'error')),
            reason TEXT,
            score INTEGER,
            device_id TEXT,
            user_id INTEGER,
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        )
        "#,
    )
    .await
    .map_err(|e| {
        error!("Failed to create logs table: {}", e);
        DbErr::Custom(format!("Failed to create logs table: {}", e))
    })?;

    // Create AUDIT_LOGS table
    db.execute_unprepared(
        r#"
        CREATE TABLE IF NOT EXISTS audit_logs (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            timestamp TEXT NOT NULL DEFAULT (datetime('now')),
            event_type TEXT NOT NULL,
            user_id INTEGER,
            username TEXT,
            ip_address TEXT,
            details TEXT,
            success INTEGER NOT NULL DEFAULT 1 CHECK(success IN (0, 1)),
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        )
        "#,
    )
    .await
    .map_err(|e| {
        error!("Failed to create audit_logs table: {}", e);
        DbErr::Custom(format!("Failed to create audit_logs table: {}", e))
    })?;

    // Create indexes
    db.execute_unprepared("CREATE INDEX IF NOT EXISTS idx_users_username ON users(username)")
        .await?;
    db.execute_unprepared("CREATE INDEX IF NOT EXISTS idx_blacklist_enabled ON blacklist(enabled)")
        .await?;
    db.execute_unprepared("CREATE INDEX IF NOT EXISTS idx_logs_timestamp ON logs(timestamp)")
        .await?;
    db.execute_unprepared("CREATE INDEX IF NOT EXISTS idx_logs_action ON logs(action)")
        .await?;

    info!("Database migrations completed successfully");

    Ok(())
}

/// Config structures for parsing default.toml
#[derive(Debug, Deserialize)]
struct TomlConfig {
    app: AppConfig,
    monitoring: MonitoringConfig,
    blocking: BlockingConfig,
    schedule: ScheduleConfig,
    overlay: OverlayConfig,
    logging: LoggingConfig,
    security: SecurityConfig,
    watchdog: WatchdogConfig,
    simulation: SimulationConfig,
}

#[derive(Debug, Deserialize)]
struct AppConfig {
    mode: String,
    startup_delay_seconds: u64,
    max_cpu_usage_percent: u32,
    max_memory_mb: u64,
}

#[derive(Debug, Deserialize)]
struct MonitoringConfig {
    scan_interval_ms: u64,
    validation_delay_ms: Vec<u64>,
    adaptive_interval: bool,
}

#[derive(Debug, Deserialize)]
struct BlockingConfig {
    kill_rate_limit_per_minute: u32,
    grace_period_seconds: Vec<u64>,
    behavior_scoring_enabled: bool,
    score_threshold: u32,
    whitelist: Vec<String>,
    blacklist: Vec<BlacklistItem>,
}

#[derive(Debug, Deserialize)]
struct ScheduleConfig {
    enabled: bool,
    timezone: String,
    rules: Vec<ScheduleRuleConfig>,
}

#[derive(Debug, Deserialize)]
struct ScheduleRuleConfig {
    days: Vec<String>,
    start: String,
    end: String,
    action: String,
}

#[derive(Debug, Deserialize)]
struct OverlayConfig {
    focus_interval_ms: u64,
    failsafe_timeout_minutes: u64,
    max_unlock_attempts: u32,
    lockout_duration_seconds: u64,
    show_process_info: bool,
    show_timestamp: bool,
    show_pc_name: bool,
    show_attempt_counter: bool,
}

#[derive(Debug, Deserialize)]
struct LoggingConfig {
    path: String,
    level: String,
    rotation_days: u32,
    structured: bool,
}

#[derive(Debug, Deserialize)]
struct SecurityConfig {
    max_auth_attempts: u32,
    backoff_base_seconds: u64,
    lockout_duration_seconds: u64,
    memory_zero_on_drop: bool,
    anti_debugging: bool,
    check_disable_flag_interval_ms: u64,
}

#[derive(Debug, Deserialize)]
struct WatchdogConfig {
    heartbeat_interval_ms: u64,
    max_missed_heartbeats: u32,
    max_restart_attempts: u32,
    deadlock_timeout_ms: u64,
}

#[derive(Debug, Deserialize)]
struct SimulationConfig {
    enabled: bool,
    simulate_process_kill: bool,
    simulate_overlay: bool,
    log_only: bool,
}

/// Seed default data from config/default.toml
pub async fn seed_default_data(db: &DatabaseConnection) -> Result<(), DbErr> {
    info!("Seeding default data from config/default.toml...");

    // Disable foreign key constraints during seeding to avoid constraint failures
    // on existing databases that might have partial data
    db.execute_unprepared("PRAGMA foreign_keys = OFF").await?;

    // Load config from default.toml
    let config_path = Path::new("config/default.toml");
    let toml_content = match std::fs::read_to_string(config_path) {
        Ok(content) => content,
        Err(e) => {
            info!(
                "Failed to read config/default.toml: {}, using hardcoded defaults",
                e
            );
            seed_hardcoded_defaults(db).await?;
            return Ok(());
        }
    };

    // Parse TOML
    let config: TomlConfig = match toml::from_str(&toml_content) {
        Ok(c) => c,
        Err(e) => {
            error!(
                "Failed to parse config/default.toml: {}, using hardcoded defaults",
                e
            );
            seed_hardcoded_defaults(db).await?;
            return Ok(());
        }
    };

    // 1. Seed Admin User (password: Admin12345!)
    // Generate argon2id hash dynamically
    let admin_password_hash = generate_argon2_hash("Admin12345!")?;
    let _ = db.execute_unprepared(&format!(
        "INSERT OR IGNORE INTO users (username, password_hash, role) VALUES ('admin', '{}', 'admin')",
        admin_password_hash
    )).await;

    // 2. Seed Configs (key-value)
    seed_configs(db, &config).await?;

    // 3. Seed Blacklist
    seed_blacklist(db, &config).await?;

    // 4. Seed Whitelist
    seed_whitelist(db, &config).await?;

    // 5. Seed Schedule
    seed_schedule(db, &config).await?;

    info!("Default data seeding completed");

    Ok(())
}

/// Seed config key-value pairs
async fn seed_configs(db: &DatabaseConnection, config: &TomlConfig) -> Result<(), DbErr> {
    // App configs
    seed_config_key(db, "app.mode", &config.app.mode, "Application mode").await?;
    seed_config_key(
        db,
        "app.startup_delay_seconds",
        &config.app.startup_delay_seconds.to_string(),
        "Startup delay in seconds",
    )
    .await?;
    seed_config_key(
        db,
        "app.max_cpu_usage_percent",
        &config.app.max_cpu_usage_percent.to_string(),
        "Max CPU usage percentage",
    )
    .await?;
    seed_config_key(
        db,
        "app.max_memory_mb",
        &config.app.max_memory_mb.to_string(),
        "Max memory in MB",
    )
    .await?;

    // Monitoring configs
    seed_config_key(
        db,
        "monitoring.scan_interval_ms",
        &config.monitoring.scan_interval_ms.to_string(),
        "Scan interval in milliseconds",
    )
    .await?;
    seed_config_key(
        db,
        "monitoring.validation_delay_ms",
        &serde_json::to_string(&config.monitoring.validation_delay_ms).unwrap(),
        "Validation delay in milliseconds",
    )
    .await?;
    seed_config_key(
        db,
        "monitoring.adaptive_interval",
        &config.monitoring.adaptive_interval.to_string(),
        "Enable adaptive scan interval",
    )
    .await?;

    // Blocking configs
    seed_config_key(
        db,
        "blocking.kill_rate_limit_per_minute",
        &config.blocking.kill_rate_limit_per_minute.to_string(),
        "Kill rate limit per minute",
    )
    .await?;
    seed_config_key(
        db,
        "blocking.grace_period_seconds",
        &serde_json::to_string(&config.blocking.grace_period_seconds).unwrap(),
        "Grace period in seconds",
    )
    .await?;
    seed_config_key(
        db,
        "blocking.behavior_scoring_enabled",
        &config.blocking.behavior_scoring_enabled.to_string(),
        "Enable behavior scoring",
    )
    .await?;
    seed_config_key(
        db,
        "blocking.score_threshold",
        &config.blocking.score_threshold.to_string(),
        "Score threshold for blocking",
    )
    .await?;

    // Overlay configs
    seed_config_key(
        db,
        "overlay.focus_interval_ms",
        &config.overlay.focus_interval_ms.to_string(),
        "Overlay focus interval",
    )
    .await?;
    seed_config_key(
        db,
        "overlay.failsafe_timeout_minutes",
        &config.overlay.failsafe_timeout_minutes.to_string(),
        "Failsafe timeout",
    )
    .await?;
    seed_config_key(
        db,
        "overlay.max_unlock_attempts",
        &config.overlay.max_unlock_attempts.to_string(),
        "Max unlock attempts",
    )
    .await?;
    seed_config_key(
        db,
        "overlay.lockout_duration_seconds",
        &config.overlay.lockout_duration_seconds.to_string(),
        "Lockout duration",
    )
    .await?;
    seed_config_key(
        db,
        "overlay.show_process_info",
        &config.overlay.show_process_info.to_string(),
        "Show process info",
    )
    .await?;
    seed_config_key(
        db,
        "overlay.show_timestamp",
        &config.overlay.show_timestamp.to_string(),
        "Show timestamp",
    )
    .await?;
    seed_config_key(
        db,
        "overlay.show_pc_name",
        &config.overlay.show_pc_name.to_string(),
        "Show PC name",
    )
    .await?;
    seed_config_key(
        db,
        "overlay.show_attempt_counter",
        &config.overlay.show_attempt_counter.to_string(),
        "Show attempt counter",
    )
    .await?;

    // Logging configs
    seed_config_key(db, "logging.path", &config.logging.path, "Log file path").await?;
    seed_config_key(db, "logging.level", &config.logging.level, "Log level").await?;
    seed_config_key(
        db,
        "logging.rotation_days",
        &config.logging.rotation_days.to_string(),
        "Log rotation days",
    )
    .await?;
    seed_config_key(
        db,
        "logging.structured",
        &config.logging.structured.to_string(),
        "Use structured logging",
    )
    .await?;

    // Security configs
    seed_config_key(
        db,
        "security.max_auth_attempts",
        &config.security.max_auth_attempts.to_string(),
        "Max auth attempts",
    )
    .await?;
    seed_config_key(
        db,
        "security.backoff_base_seconds",
        &config.security.backoff_base_seconds.to_string(),
        "Backoff base seconds",
    )
    .await?;
    seed_config_key(
        db,
        "security.lockout_duration_seconds",
        &config.security.lockout_duration_seconds.to_string(),
        "Security lockout duration",
    )
    .await?;
    seed_config_key(
        db,
        "security.memory_zero_on_drop",
        &config.security.memory_zero_on_drop.to_string(),
        "Zero memory on drop",
    )
    .await?;
    seed_config_key(
        db,
        "security.anti_debugging",
        &config.security.anti_debugging.to_string(),
        "Enable anti-debugging",
    )
    .await?;
    seed_config_key(
        db,
        "security.check_disable_flag_interval_ms",
        &config.security.check_disable_flag_interval_ms.to_string(),
        "Check disable flag interval",
    )
    .await?;

    // Watchdog configs
    seed_config_key(
        db,
        "watchdog.heartbeat_interval_ms",
        &config.watchdog.heartbeat_interval_ms.to_string(),
        "Watchdog heartbeat interval",
    )
    .await?;
    seed_config_key(
        db,
        "watchdog.max_missed_heartbeats",
        &config.watchdog.max_missed_heartbeats.to_string(),
        "Max missed heartbeats",
    )
    .await?;
    seed_config_key(
        db,
        "watchdog.max_restart_attempts",
        &config.watchdog.max_restart_attempts.to_string(),
        "Max restart attempts",
    )
    .await?;
    seed_config_key(
        db,
        "watchdog.deadlock_timeout_ms",
        &config.watchdog.deadlock_timeout_ms.to_string(),
        "Deadlock timeout",
    )
    .await?;

    // Simulation configs
    seed_config_key(
        db,
        "simulation.enabled",
        &config.simulation.enabled.to_string(),
        "Enable simulation mode",
    )
    .await?;
    seed_config_key(
        db,
        "simulation.simulate_process_kill",
        &config.simulation.simulate_process_kill.to_string(),
        "Simulate process kill",
    )
    .await?;
    seed_config_key(
        db,
        "simulation.simulate_overlay",
        &config.simulation.simulate_overlay.to_string(),
        "Simulate overlay",
    )
    .await?;
    seed_config_key(
        db,
        "simulation.log_only",
        &config.simulation.log_only.to_string(),
        "Log only mode",
    )
    .await?;

    Ok(())
}

/// Helper to seed a single config key
async fn seed_config_key(
    db: &DatabaseConnection,
    key: &str,
    value: &str,
    description: &str,
) -> Result<(), DbErr> {
    db.execute_unprepared(&format!(
        "INSERT OR REPLACE INTO configs (key, value, description, updated_at) VALUES ('{}', '{}', '{}', datetime('now'))",
        key, value, description
    )).await?;
    Ok(())
}

/// Parse and seed blacklist from TOML content
// Keterangan Perubahan (R021): Fix QueryResult.values() error; gunakan tx.query_one().try_get(\"\", \"id\"); area seed_blacklist; tes cargo check OK.
// Tujuan: Kompilasi bersih, seeding atomic. Dampak: DB init aman. Tes: cargo check/build pass.
async fn seed_blacklist(db: &DatabaseConnection, config: &TomlConfig) -> Result<(), DbErr> {
    info!(
        "Seeding blacklist from parsed config: {} items",
        config.blocking.blacklist.len()
    );

    let tx = db.begin().await?;

    tx.execute_unprepared("DELETE FROM blacklist_processes")
        .await?;
    tx.execute_unprepared("DELETE FROM blacklist_paths").await?;
    tx.execute_unprepared("DELETE FROM blacklist").await?;

    for item in &config.blocking.blacklist {
        info!(
            "  Seeding: {} - processes={:?}, paths={:?}",
            item.name, item.process_names, item.paths
        );

        tx.execute_unprepared(&format!(
            "INSERT INTO blacklist (name, description, category, risk_level, enabled, created_at, updated_at) VALUES ('{}', '{}', 'game', 'medium', 1, datetime('now'), datetime('now'))",
            item.name.replace("'", "''"),
            item.description.replace("'", "''")
        )).await?;

        let row_id_res = tx
            .query_one(sea_orm::Statement::from_string(
                sea_orm::DatabaseBackend::Sqlite,
                "SELECT last_insert_rowid() as id",
            ))
            .await?;

        let blacklist_id_i64: i64 = row_id_res
            .ok_or(DbErr::Custom("No row ID returned".into()))?
            .try_get("", "id")
            .map_err(|e| DbErr::Custom(format!("Extract row ID failed: {}", e)))?;

        let blacklist_id = blacklist_id_i64 as i32;

        for process in &item.process_names {
            tx.execute_unprepared(&format!(
                "INSERT INTO blacklist_processes (blacklist_id, process_name, created_at) VALUES ({}, '{}', datetime('now'))",
                blacklist_id,
                process.replace("'", "''")
            )).await?;
        }

        for path in &item.paths {
            tx.execute_unprepared(&format!(
                "INSERT INTO blacklist_paths (blacklist_id, path, created_at) VALUES ({}, '{}', datetime('now'))",
                blacklist_id,
                path.replace("'", "''")
            )).await?;
        }
    }

    tx.commit().await?;

    info!("Blacklist seeding complete");
    Ok(())
}

#[derive(Debug, Clone, Deserialize)]
struct BlacklistItem {
    name: String,
    description: String,
    process_names: Vec<String>,
    paths: Vec<String>,
}

/// Seed whitelist from config
async fn seed_whitelist(db: &DatabaseConnection, config: &TomlConfig) -> Result<(), DbErr> {
    for process in &config.blocking.whitelist {
        db.execute_unprepared(&format!(
            "INSERT OR IGNORE INTO whitelist (process_name, enabled, created_at, updated_at) VALUES ('{}', 1, datetime('now'), datetime('now'))",
            process.replace("'", "''")
        )).await?;
    }
    Ok(())
}

/// Seed schedule from config
async fn seed_schedule(db: &DatabaseConnection, config: &TomlConfig) -> Result<(), DbErr> {
    let enabled = if config.schedule.enabled { 1 } else { 0 };
    db.execute_unprepared(&format!(
        "INSERT OR IGNORE INTO schedule (id, enabled, timezone, created_at, updated_at) VALUES (1, {}, '{}', datetime('now'), datetime('now'))",
        enabled,
        config.schedule.timezone
    )).await?;

    for rule in &config.schedule.rules {
        let days_json = serde_json::to_string(&rule.days).unwrap_or_else(|_| "[]".to_string());
        db.execute_unprepared(&format!(
            "INSERT OR IGNORE INTO schedule_rules (schedule_id, days, start_time, end_time, action, enabled, created_at, updated_at) VALUES (1, '{}', '{}', '{}', '{}', 1, datetime('now'), datetime('now'))",
            days_json,
            rule.start,
            rule.end,
            rule.action
        )).await?;
    }

    Ok(())
}

/// Fallback function with hardcoded defaults
async fn seed_hardcoded_defaults(db: &DatabaseConnection) -> Result<(), DbErr> {
    info!("Seeding hardcoded defaults...");

    let admin_password_hash = "$argon2id$v=19$m=19456,t=2,p=1$phv4zmAQxu/cwVdRY9wgLg$441jRs24dn+kSxOf4K21qGzrsqb2rbtPsFdR5rvCMug";
    let _ = db.execute_unprepared(&format!(
        "INSERT OR IGNORE INTO users (username, password_hash, role) VALUES ('admin', '{}', 'admin')",
        admin_password_hash
    )).await;

    let _ = db
        .execute_unprepared(
            "INSERT OR IGNORE INTO schedule (id, enabled, timezone) VALUES (1, 1, 'Asia/Jakarta')",
        )
        .await;

    let _ = db.execute_unprepared(
        r#"INSERT OR IGNORE INTO schedule_rules (schedule_id, days, start_time, end_time, action, enabled)
           VALUES (1, '["Monday","Tuesday","Wednesday","Thursday","Friday"]', '07:00', '15:00', 'block_games', 1)"#
    ).await;

    let _ = db.execute_unprepared(
        r#"INSERT OR IGNORE INTO schedule_rules (schedule_id, days, start_time, end_time, action, enabled)
           VALUES (1, '["Saturday"]', '07:00', '12:00', 'block_games', 1)"#
    ).await;

    Ok(())
}

/// Helper function to insert config key-value pair
pub async fn insert_config(
    db: &DatabaseConnection,
    key: &str,
    value: &str,
    description: Option<&str>,
) -> Result<(), DbErr> {
    db.execute_unprepared(
        &format!(
            "INSERT OR REPLACE INTO configs (key, value, description, updated_at) VALUES ('{}', '{}', '{}', datetime('now'))",
            key,
            value,
            description.unwrap_or("")
        )
    ).await?;

    Ok(())
}

/// Generate argon2id password hash for a given password
pub fn generate_argon2_hash(password: &str) -> Result<String, DbErr> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();

    argon2
        .hash_password(password.as_bytes(), &salt)
        .map(|hash| hash.to_string())
        .map_err(|e| {
            error!("Failed to generate password hash: {}", e);
            DbErr::Custom(format!("Failed to generate password hash: {}", e))
        })
}

/// Verify password against an argon2id hash
pub fn verify_password(password: &str, hash: &str) -> Result<bool, DbErr> {
    let parsed_hash = PasswordHash::new(hash).map_err(|e| {
        error!("Invalid password hash format: {}", e);
        DbErr::Custom(format!("Invalid password hash format: {}", e))
    })?;

    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_init_database() {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");

        let result = init_database(&db_path).await;

        assert!(result.is_ok());
        assert!(db_path.exists());
    }
}
