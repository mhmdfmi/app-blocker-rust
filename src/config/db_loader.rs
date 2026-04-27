//! Database Config Loader
//! Load configuration from SQLite database and convert to AppConfig

use crate::config::settings::{
    // Core types used directly:
    AppConfig,
    AppMode,
    BlockedApp,
    ScheduleRule,
    // NOTE: The following types are accessed via AppConfig fields (e.g., app_config.app, app_config.monitoring)
    // but kept as imports for potential direct construction in future extensions:
    // AppMeta,      // Accessed as: app_config.app (type: AppMeta)
    // BlockingConfig,   // Accessed as: app_config.blocking (type: BlockingConfig)
    // LoggingConfig,     // Accessed as: app_config.logging (type: LoggingConfig)
    // MonitoringConfig,  // Accessed as: app_config.monitoring (type: MonitoringConfig)
    // OverlayConfig,     // Accessed as: app_config.overlay (type: OverlayConfig)
    // ScheduleConfig,    // Accessed as: app_config.schedule (type: ScheduleConfig)
    // SecurityConfig,   // Accessed as: app_config.security (type: SecurityConfig)
    // SimulationConfig, // Accessed as: app_config.simulation (type: SimulationConfig)
    // WatchdogConfig,    // Accessed as: app_config.watchdog (type: WatchdogConfig)
};
use crate::repository::{
    BlacklistRepository, ConfigRepository, ScheduleRepository, WhitelistRepository,
};
use crate::utils::error::{AppError, AppResult};
use sea_orm::DatabaseConnection;
use std::sync::{Arc, RwLock};
use tracing::{info, warn};

/// Database Config Loader
/// Loads configuration from SQLite database and provides in-memory cache
pub struct DbConfigLoader {
    db: DatabaseConnection,
    config_cache: Arc<RwLock<AppConfig>>,
}

impl DbConfigLoader {
    /// Create new database config loader
    pub fn new(db: DatabaseConnection) -> Self {
        Self {
            db,
            config_cache: Arc::new(RwLock::new(AppConfig::default())),
        }
    }

    /// Create new database config loader with initial load
    pub async fn new_with_load(db: DatabaseConnection) -> AppResult<Self> {
        let loader = Self::new(db);
        loader.load_from_db().await?;
        Ok(loader)
    }

    /// Load configuration from database
    pub async fn load_from_db(&self) -> AppResult<()> {
        let config_repo = ConfigRepository::new(self.db.clone());
        let blacklist_repo = BlacklistRepository::new(self.db.clone());
        let whitelist_repo = WhitelistRepository::new(self.db.clone());
        let schedule_repo = ScheduleRepository::new(self.db.clone());

        // Build AppConfig from database
        let mut app_config = AppConfig::default();

        // Load App configs
        if let Ok(mode) = config_repo.get_value("app.mode").await {
            app_config.app.mode = match mode.as_deref() {
                Some("development") => AppMode::Development,
                Some("simulation") => AppMode::Simulation,
                _ => AppMode::Production,
            };
        }
        if let Ok(Some(val)) = config_repo.get_i32("app.startup_delay_seconds").await {
            app_config.app.startup_delay_seconds = val as u64;
        }
        if let Ok(Some(val)) = config_repo.get_i32("app.max_cpu_usage_percent").await {
            app_config.app.max_cpu_usage_percent = val as u32;
        }
        if let Ok(Some(val)) = config_repo.get_i32("app.max_memory_mb").await {
            app_config.app.max_memory_mb = val as u64;
        }

        // Load Monitoring configs
        if let Ok(Some(val)) = config_repo.get_i32("monitoring.scan_interval_ms").await {
            app_config.monitoring.scan_interval_ms = val as u64;
        }
        if let Ok(Some(val)) = config_repo
            .get_value("monitoring.validation_delay_ms")
            .await
        {
            if let Ok(arr) = serde_json::from_str::<Vec<u64>>(&val) {
                if arr.len() >= 2 {
                    app_config.monitoring.validation_delay_ms = (arr[0], arr[1]);
                }
            }
        }
        if let Ok(Some(val)) = config_repo.get_bool("monitoring.adaptive_interval").await {
            app_config.monitoring.adaptive_interval = val;
        }

        // Load Blocking configs
        if let Ok(Some(val)) = config_repo
            .get_i32("blocking.kill_rate_limit_per_minute")
            .await
        {
            app_config.blocking.kill_rate_limit_per_minute = val as u32;
        }
        if let Ok(Some(val)) = config_repo.get_value("blocking.grace_period_seconds").await {
            if let Ok(arr) = serde_json::from_str::<Vec<u64>>(&val) {
                if arr.len() >= 2 {
                    app_config.blocking.grace_period_seconds = (arr[0], arr[1]);
                }
            }
        }
        if let Ok(Some(val)) = config_repo
            .get_bool("blocking.behavior_scoring_enabled")
            .await
        {
            app_config.blocking.behavior_scoring_enabled = val;
        }
        if let Ok(Some(val)) = config_repo.get_i32("blocking.score_threshold").await {
            app_config.blocking.score_threshold = val as u32;
        }

        // Load Blacklist from database
        match blacklist_repo.find_all_with_details().await {
            Ok(blacklists) => {
                app_config.blocking.blacklist = blacklists
                    .into_iter()
                    .map(|b| BlockedApp {
                        name: b.blacklist.name,
                        process_names: b.processes.into_iter().map(|p| p.process_name).collect(),
                        paths: b.paths.into_iter().map(|p| p.path).collect(),
                        description: b.blacklist.description.unwrap_or_default(),
                    })
                    .collect();
            }
            Err(e) => {
                warn!("Failed to load blacklist from DB: {}", e);
            }
        }

        // Load Whitelist from database
        match whitelist_repo.get_enabled_names().await {
            Ok(names) => {
                app_config.blocking.whitelist = names;
            }
            Err(e) => {
                warn!("Failed to load whitelist from DB: {}", e);
            }
        }

        // Load Schedule configs
        if let Ok(Some(val)) = config_repo.get_bool("schedule.enabled").await {
            app_config.schedule.enabled = val;
        }
        if let Ok(Some(val)) = config_repo.get_value("schedule.timezone").await {
            app_config.schedule.timezone = val;
        }

        // Load Schedule rules from database
        if let Ok(Some(schedule_with_rules)) = schedule_repo.get_global_with_rules().await {
            app_config.schedule.rules = schedule_with_rules
                .rules
                .into_iter()
                .map(|r| {
                    let days: Vec<String> = serde_json::from_str(&r.days).unwrap_or_default();
                    ScheduleRule {
                        days,
                        start: r.start_time,
                        end: r.end_time,
                        action: r.action,
                    }
                })
                .collect();
        }

        // Load Overlay configs
        if let Ok(Some(val)) = config_repo.get_i32("overlay.focus_interval_ms").await {
            app_config.overlay.focus_interval_ms = val as u64;
        }
        if let Ok(Some(val)) = config_repo
            .get_i32("overlay.failsafe_timeout_minutes")
            .await
        {
            app_config.overlay.failsafe_timeout_minutes = val as u64;
        }
        if let Ok(Some(val)) = config_repo.get_i32("overlay.max_unlock_attempts").await {
            app_config.overlay.max_unlock_attempts = val as u32;
        }
        if let Ok(Some(val)) = config_repo
            .get_i32("overlay.lockout_duration_seconds")
            .await
        {
            app_config.overlay.lockout_duration_seconds = val as u64;
        }
        if let Ok(Some(val)) = config_repo.get_bool("overlay.show_process_info").await {
            app_config.overlay.show_process_info = val;
        }
        if let Ok(Some(val)) = config_repo.get_bool("overlay.show_timestamp").await {
            app_config.overlay.show_timestamp = val;
        }
        if let Ok(Some(val)) = config_repo.get_bool("overlay.show_pc_name").await {
            app_config.overlay.show_pc_name = val;
        }
        if let Ok(Some(val)) = config_repo.get_bool("overlay.show_attempt_counter").await {
            app_config.overlay.show_attempt_counter = val;
        }

        // Load Logging configs
        if let Ok(Some(val)) = config_repo.get_value("logging.path").await {
            app_config.logging.path = val;
        }
        if let Ok(Some(val)) = config_repo.get_value("logging.level").await {
            app_config.logging.level = val;
        }
        if let Ok(Some(val)) = config_repo.get_i32("logging.rotation_days").await {
            app_config.logging.rotation_days = val as u32;
        }
        if let Ok(Some(val)) = config_repo.get_bool("logging.structured").await {
            app_config.logging.structured = val;
        }

        // Load Security configs
        if let Ok(Some(val)) = config_repo.get_i32("security.max_auth_attempts").await {
            app_config.security.max_auth_attempts = val as u32;
        }
        if let Ok(Some(val)) = config_repo.get_i32("security.backoff_base_seconds").await {
            app_config.security.backoff_base_seconds = val as u64;
        }
        if let Ok(Some(val)) = config_repo
            .get_i32("security.lockout_duration_seconds")
            .await
        {
            app_config.security.lockout_duration_seconds = val as u64;
        }
        if let Ok(Some(val)) = config_repo.get_bool("security.memory_zero_on_drop").await {
            app_config.security.memory_zero_on_drop = val;
        }
        if let Ok(Some(val)) = config_repo.get_bool("security.anti_debugging").await {
            app_config.security.anti_debugging = val;
        }
        if let Ok(Some(val)) = config_repo
            .get_i32("security.check_disable_flag_interval_ms")
            .await
        {
            app_config.security.check_disable_flag_interval_ms = val as u64;
        }

        // Load Watchdog configs
        if let Ok(Some(val)) = config_repo.get_i32("watchdog.heartbeat_interval_ms").await {
            app_config.watchdog.heartbeat_interval_ms = val as u64;
        }
        if let Ok(Some(val)) = config_repo.get_i32("watchdog.max_missed_heartbeats").await {
            app_config.watchdog.max_missed_heartbeats = val as u32;
        }
        if let Ok(Some(val)) = config_repo.get_i32("watchdog.max_restart_attempts").await {
            app_config.watchdog.max_restart_attempts = val as u32;
        }
        if let Ok(Some(val)) = config_repo.get_i32("watchdog.deadlock_timeout_ms").await {
            app_config.watchdog.deadlock_timeout_ms = val as u64;
        }

        // Load Simulation configs
        if let Ok(Some(val)) = config_repo.get_bool("simulation.enabled").await {
            app_config.simulation.enabled = val;
        }
        if let Ok(Some(val)) = config_repo
            .get_bool("simulation.simulate_process_kill")
            .await
        {
            app_config.simulation.simulate_process_kill = val;
        }
        if let Ok(Some(val)) = config_repo.get_bool("simulation.simulate_overlay").await {
            app_config.simulation.simulate_overlay = val;
        }
        if let Ok(Some(val)) = config_repo.get_bool("simulation.log_only").await {
            app_config.simulation.log_only = val;
        }

        // Store in cache
        let mut cache = self
            .config_cache
            .write()
            .map_err(|e| AppError::Config(format!("Failed to write config cache: {}", e)))?;
        *cache = app_config;

        info!("Configuration loaded from database successfully");

        Ok(())
    }

    /// Get current config (from cache)
    pub fn get(&self) -> AppResult<AppConfig> {
        self.config_cache
            .read()
            .map(|c| c.clone())
            .map_err(|e| AppError::Config(format!("Failed to read config cache: {}", e)))
    }

    /// Get Arc to config for threading
    pub fn get_arc(&self) -> Arc<RwLock<AppConfig>> {
        Arc::clone(&self.config_cache)
    }

    /// Reload config from database
    pub async fn reload(&self) -> AppResult<()> {
        self.load_from_db().await
    }

    /// Get database connection
    pub fn db(&self) -> &DatabaseConnection {
        &self.db
    }
}

/// Convert AppConfig to HashMap for API response
impl AppConfig {
    pub fn to_config_map(&self) -> std::collections::HashMap<String, String> {
        let mut map = std::collections::HashMap::new();

        // App
        map.insert("app.mode".to_string(), self.app.mode.to_string());
        map.insert(
            "app.startup_delay_seconds".to_string(),
            self.app.startup_delay_seconds.to_string(),
        );
        map.insert(
            "app.max_cpu_usage_percent".to_string(),
            self.app.max_cpu_usage_percent.to_string(),
        );
        map.insert(
            "app.max_memory_mb".to_string(),
            self.app.max_memory_mb.to_string(),
        );

        // Monitoring
        map.insert(
            "monitoring.scan_interval_ms".to_string(),
            self.monitoring.scan_interval_ms.to_string(),
        );
        map.insert(
            "monitoring.validation_delay_ms".to_string(),
            serde_json::to_string(&(
                self.monitoring.validation_delay_ms.0,
                self.monitoring.validation_delay_ms.1,
            ))
            .unwrap_or_default(),
        );
        map.insert(
            "monitoring.adaptive_interval".to_string(),
            self.monitoring.adaptive_interval.to_string(),
        );

        // Blocking
        map.insert(
            "blocking.kill_rate_limit_per_minute".to_string(),
            self.blocking.kill_rate_limit_per_minute.to_string(),
        );
        map.insert(
            "blocking.grace_period_seconds".to_string(),
            serde_json::to_string(&(
                self.blocking.grace_period_seconds.0,
                self.blocking.grace_period_seconds.1,
            ))
            .unwrap_or_default(),
        );
        map.insert(
            "blocking.behavior_scoring_enabled".to_string(),
            self.blocking.behavior_scoring_enabled.to_string(),
        );
        map.insert(
            "blocking.score_threshold".to_string(),
            self.blocking.score_threshold.to_string(),
        );

        // Schedule
        map.insert(
            "schedule.enabled".to_string(),
            self.schedule.enabled.to_string(),
        );
        map.insert(
            "schedule.timezone".to_string(),
            self.schedule.timezone.clone(),
        );

        // Overlay
        map.insert(
            "overlay.focus_interval_ms".to_string(),
            self.overlay.focus_interval_ms.to_string(),
        );
        map.insert(
            "overlay.failsafe_timeout_minutes".to_string(),
            self.overlay.failsafe_timeout_minutes.to_string(),
        );
        map.insert(
            "overlay.max_unlock_attempts".to_string(),
            self.overlay.max_unlock_attempts.to_string(),
        );
        map.insert(
            "overlay.lockout_duration_seconds".to_string(),
            self.overlay.lockout_duration_seconds.to_string(),
        );
        map.insert(
            "overlay.show_process_info".to_string(),
            self.overlay.show_process_info.to_string(),
        );
        map.insert(
            "overlay.show_timestamp".to_string(),
            self.overlay.show_timestamp.to_string(),
        );
        map.insert(
            "overlay.show_pc_name".to_string(),
            self.overlay.show_pc_name.to_string(),
        );
        map.insert(
            "overlay.show_attempt_counter".to_string(),
            self.overlay.show_attempt_counter.to_string(),
        );

        // Logging
        map.insert("logging.path".to_string(), self.logging.path.clone());
        map.insert("logging.level".to_string(), self.logging.level.clone());
        map.insert(
            "logging.rotation_days".to_string(),
            self.logging.rotation_days.to_string(),
        );
        map.insert(
            "logging.structured".to_string(),
            self.logging.structured.to_string(),
        );

        // Security
        map.insert(
            "security.max_auth_attempts".to_string(),
            self.security.max_auth_attempts.to_string(),
        );
        map.insert(
            "security.backoff_base_seconds".to_string(),
            self.security.backoff_base_seconds.to_string(),
        );
        map.insert(
            "security.lockout_duration_seconds".to_string(),
            self.security.lockout_duration_seconds.to_string(),
        );
        map.insert(
            "security.memory_zero_on_drop".to_string(),
            self.security.memory_zero_on_drop.to_string(),
        );
        map.insert(
            "security.anti_debugging".to_string(),
            self.security.anti_debugging.to_string(),
        );
        map.insert(
            "security.check_disable_flag_interval_ms".to_string(),
            self.security.check_disable_flag_interval_ms.to_string(),
        );

        // Watchdog
        map.insert(
            "watchdog.heartbeat_interval_ms".to_string(),
            self.watchdog.heartbeat_interval_ms.to_string(),
        );
        map.insert(
            "watchdog.max_missed_heartbeats".to_string(),
            self.watchdog.max_missed_heartbeats.to_string(),
        );
        map.insert(
            "watchdog.max_restart_attempts".to_string(),
            self.watchdog.max_restart_attempts.to_string(),
        );
        map.insert(
            "watchdog.deadlock_timeout_ms".to_string(),
            self.watchdog.deadlock_timeout_ms.to_string(),
        );

        // Simulation
        map.insert(
            "simulation.enabled".to_string(),
            self.simulation.enabled.to_string(),
        );
        map.insert(
            "simulation.simulate_process_kill".to_string(),
            self.simulation.simulate_process_kill.to_string(),
        );
        map.insert(
            "simulation.simulate_overlay".to_string(),
            self.simulation.simulate_overlay.to_string(),
        );
        map.insert(
            "simulation.log_only".to_string(),
            self.simulation.log_only.to_string(),
        );

        map
    }
}
