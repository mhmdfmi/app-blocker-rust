//! Bootstrap Module
//! Inisialisasi semua komponen aplikasi: database, repositories, config, engine
//!
//! Usage:
//! ```rust
//! use app_blocker_lib::bootstrap::AppBootstrap;
//!
//! let bootstrap = AppBootstrap::new()
//!     .await?;
//!
//! let app = bootstrap.run().await?;
//! ```

use crate::config::settings::AppConfig;
use crate::config::{ConfigManager, DbConfigLoader};
use crate::constants::paths;
// IMPORTS FOR CONFIGURED COMPONENTS:
use crate::core::events::ComponentId;
use crate::core::state::AppState; // For state management transitions

use crate::core::engine::AppEngine;
use crate::core::events::AppEvent;
use crate::core::state::StateManager;
use crate::core::watchdog::send_watchdog_heartbeat;
use crate::db;
use crate::repository::{
    BlacklistRepository, ConfigRepository, LogRepository, ScheduleRepository, UserRepository,
    WhitelistRepository,
};
use crate::security::auth::AuthManager;
use crate::system::process::WindowsProcessService;
use crate::utils::error::{AppError, AppResult};
use sea_orm::DatabaseConnection;
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::sync::{Arc, RwLock};
use tracing::{error, info, warn};

/// Application bootstrap - handles initialization of all components
pub struct AppBootstrap {
    /// Path to database file
    db_path: Option<PathBuf>,
    /// Path to fallback config TOML file
    config_path: Option<PathBuf>,
    /// Database connection (initialized after init)
    db: Option<DatabaseConnection>,
    /// Config loader from database
    config_loader: Option<DbConfigLoader>,
    /// TOML ConfigManager for hot reload fallback
    toml_config_manager: Option<Arc<ConfigManager>>,
    /// App config (Arc for sharing)
    config: Option<Arc<RwLock<AppConfig>>>,
    /// State manager
    state_manager: Option<Arc<StateManager>>,
    /// Event channel sender
    event_tx: Option<mpsc::Sender<AppEvent>>,
}

impl AppBootstrap {
    /// Create new bootstrap instance
    pub fn new() -> Self {
        Self {
            db_path: None,
            config_path: None,
            db: None,
            config_loader: None,
            toml_config_manager: None,
            config: None,
            state_manager: None,
            event_tx: None,
        }
    }

    /// Set database path
    pub fn with_db_path<P: AsRef<Path>>(mut self, path: P) -> Self {
        self.db_path = Some(path.as_ref().to_path_buf());
        self
    }

    /// Set fallback config path (TOML)
    pub fn with_config_path<P: AsRef<Path>>(mut self, path: P) -> Self {
        self.config_path = Some(path.as_ref().to_path_buf());
        self
    }

    /// Initialize database and run migrations
    pub async fn init_database(mut self) -> AppResult<Self> {
        // Use AppData path by default
        let db_path = self.db_path.clone().unwrap_or_else(paths::get_db_path);

        info!(path = %db_path.display(), "Initializing database...");

        // Ensure AppData directory exists
        paths::ensure_appdata_dir()
            .map_err(|e| AppError::io("Failed to create AppData directory", e))?;

        // Create parent directories if needed
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| AppError::io(format!("Create data dir: {}", db_path.display()), e))?;
        }

        // Initialize database (create tables, run migrations, seed data)
        let db = db::init::init_database(&db_path.clone())
            .await
            .map_err(|e| {
                error!(error = %e, "Failed to initialize database");
                AppError::Config(format!("Database init failed: {}", e))
            })?;

        info!("Database initialized successfully");

        self.db = Some(db);
        Ok(self)
    }

    /// Load configuration from database
    pub async fn load_config(mut self) -> AppResult<Self> {
        let db = self
            .db
            .as_ref()
            .ok_or_else(|| AppError::Config("Database not initialized".to_string()))?;

        info!("Loading configuration from database...");

        // Create config loader and load from DB
        let config_loader = DbConfigLoader::new_with_load(db.clone()).await?;

        // Get config arc for sharing
        let config = config_loader.get_arc();

        // Also load TOML config for hot reload fallback
        // Check AppData config first, then fallback to provided path
        let toml_path = self
            .config_path
            .clone()
            .unwrap_or_else(paths::get_config_path);

        let mut toml_config_manager = None;
        if toml_path.exists() {
            info!(path = %toml_path.display(), "Loading fallback config from TOML");
            match ConfigManager::load(&toml_path) {
                Ok(mgr) => {
                    toml_config_manager = Some(Arc::new(mgr));
                    info!("TOML config loaded for hot reload fallback");
                }
                Err(e) => {
                    warn!(error = %e, "Failed to load TOML config, using DB config only");
                }
            }
        } else {
            // Auto-create default config in AppData if not exists
            info!(path = %toml_path.display(), "Creating default config file");
            if let Err(e) = Self::create_default_config(&toml_path) {
                warn!(error = %e, "Failed to create default config");
            }
        }

        info!("Configuration loaded from database");

        self.config_loader = Some(config_loader);
        self.toml_config_manager = toml_config_manager;
        self.config = Some(config);
        Ok(self)
    }

    /// Initialize state manager
    pub fn init_state_manager(mut self) -> Self {
        let state_manager = Arc::new(StateManager::new());
        info!("State manager initialized");
        self.state_manager = Some(state_manager);
        self
    }

    /// Initialize state to monitoring (default starting state)
    pub fn init_initial_state(&self) -> AppResult<()> {
        if let Some(state_manager) = &self.state_manager {
            state_manager.transition_to(AppState::Monitoring, "initial_state")?;
            info!("Initial state set to Monitoring");
            Ok(())
        } else {
            Err(AppError::Config(
                "State manager not initialized".to_string(),
            ))
        }
    }

    /// Reload TOML config for hot reload fallback
    pub fn reload_toml_config(&self) -> AppResult<bool> {
        if let Some(mgr) = &self.toml_config_manager {
            let reloaded = mgr.hot_reload()?;
            if reloaded {
                info!("TOML config hot reloaded successfully");
            }
            Ok(reloaded)
        } else {
            Ok(false)
        }
    }

    /// Get TOML config manager for external access
    pub fn get_toml_config_manager(&self) -> Option<Arc<ConfigManager>> {
        self.toml_config_manager.clone()
    }

    /// Initialize event channel
    pub fn init_event_channel(mut self) -> Self {
        let (tx, _rx) = mpsc::channel();
        self.event_tx = Some(tx);
        info!("Event channel initialized");
        self
    }

    /// Build repositories (returns tuple)
    pub fn build_repositories(
        &self,
    ) -> AppResult<(
        ConfigRepository,
        BlacklistRepository,
        WhitelistRepository,
        ScheduleRepository,
        LogRepository,
        UserRepository,
    )> {
        let db = self
            .db
            .as_ref()
            .ok_or_else(|| AppError::Config("Database not initialized".to_string()))?;

        Ok((
            ConfigRepository::new(db.clone()),
            BlacklistRepository::new(db.clone()),
            WhitelistRepository::new(db.clone()),
            ScheduleRepository::new(db.clone()),
            LogRepository::new(db.clone()),
            UserRepository::new(db.clone()),
        ))
    }

    /// Build the engine with all dependencies
    pub fn build_engine(&self) -> AppResult<AppEngine> {
        let config = self
            .config
            .as_ref()
            .ok_or_else(|| AppError::Config("Config not loaded".to_string()))?
            .clone();

        let state_manager = self
            .state_manager
            .as_ref()
            .ok_or_else(|| AppError::Config("State manager not initialized".to_string()))?
            .clone();

        let event_tx = self
            .event_tx
            .as_ref()
            .ok_or_else(|| AppError::Config("Event channel not initialized".to_string()))?
            .clone();

        let (_, event_rx) = mpsc::channel();

        // Create process service
        let process_service = Box::new(WindowsProcessService::new(false));

        // Create auth manager - use default for now, can be enhanced to load from DB
        let auth_svc = crate::security::auth::Argon2AuthService::with_default_password()
            .map(|(svc, _)| svc)
            .unwrap_or_else(|_| {
                // Fallback to default if no password is set
                let (svc, _) =
                    crate::security::auth::Argon2AuthService::with_default_password().unwrap();
                svc
            });

        let config_guard = self.config.as_ref().unwrap().read().unwrap();
        let max_attempts = config_guard.security.max_auth_attempts;
        let lockout_duration = config_guard.security.lockout_duration_seconds;
        drop(config_guard);

        let auth_manager = AuthManager::new(Box::new(auth_svc), max_attempts, lockout_duration);

        let engine = AppEngine::new(
            event_rx,
            event_tx,
            state_manager,
            config,
            process_service,
            auth_manager,
        );

        Ok(engine)
    }

    /// Create default config file in AppData if not exists
    fn create_default_config(path: &Path) -> std::io::Result<()> {
        const DEFAULT_CONFIG: &str = include_str!("../config/default.toml");
        std::fs::write(path, DEFAULT_CONFIG)
    }

    /// Build complete application
    pub async fn build(mut self) -> AppResult<Application> {
        // Step 1: Initialize database
        self = self.init_database().await?;

        // Step 2: Load config from DB
        self = self.load_config().await?;

        // Step 3: Initialize state manager
        self = self.init_state_manager();

        // Step 4: Initialize event channel
        self = self.init_event_channel();

        // Step 5: Build repositories
        let repositories = self.build_repositories()?;

        // Step 6: Build engine
        let engine = self.build_engine()?;

        info!("Application bootstrap complete");

        Ok(Application {
            db: self.db.unwrap(),
            config: self.config.unwrap(),
            config_loader: self.config_loader.unwrap(),
            state_manager: self.state_manager.unwrap(),
            event_tx: self.event_tx.unwrap(),
            repositories,
            engine,
        })
    }

    /// Quick initialization with just database
    pub async fn quick_init(db_path: &str) -> AppResult<Application> {
        Self::new().with_db_path(db_path).build().await
    }
}

impl Default for AppBootstrap {
    fn default() -> Self {
        Self::new()
    }
}

/// Complete application instance with all components
pub struct Application {
    /// Database connection
    pub db: DatabaseConnection,
    /// Configuration (Arc for sharing)
    pub config: Arc<RwLock<AppConfig>>,
    /// Database config loader (for hot reload)
    pub config_loader: DbConfigLoader,
    /// State manager
    pub state_manager: Arc<StateManager>,
    /// Event channel sender
    pub event_tx: mpsc::Sender<AppEvent>,
    /// All repositories
    pub repositories: (
        ConfigRepository,
        BlacklistRepository,
        WhitelistRepository,
        ScheduleRepository,
        LogRepository,
        UserRepository,
    ),
    /// Engine instance
    pub engine: AppEngine,
}

impl Application {
    /// Reload configuration from database
    pub async fn reload_config(&self) -> AppResult<()> {
        self.config_loader.reload().await
    }

    /// Send heartbeat for a specific component to watchdog
    /// Usage:
    ///   app.send_heartbeat(ComponentId::Monitor);
    ///   app.send_heartbeat(ComponentId::Engine);
    ///   app.send_heartbeat(ComponentId::UiOverlay);
    ///   app.send_heartbeat(ComponentId::ConfigWatcher);
    #[inline]
    pub fn send_heartbeat(&self, component: ComponentId) {
        send_watchdog_heartbeat(component);
    }

    /// Get config snapshot
    pub fn get_config(&self) -> AppResult<AppConfig> {
        self.config
            .read()
            .map(|c| c.clone())
            .map_err(|e| AppError::Config(format!("Read config: {}", e)))
    }

    /// Get specific repository
    pub fn config_repo(&self) -> &ConfigRepository {
        &self.repositories.0
    }

    pub fn blacklist_repo(&self) -> &BlacklistRepository {
        &self.repositories.1
    }

    pub fn whitelist_repo(&self) -> &WhitelistRepository {
        &self.repositories.2
    }

    pub fn schedule_repo(&self) -> &ScheduleRepository {
        &self.repositories.3
    }

    pub fn log_repo(&self) -> &LogRepository {
        &self.repositories.4
    }

    pub fn user_repo(&self) -> &UserRepository {
        &self.repositories.5
    }
}
