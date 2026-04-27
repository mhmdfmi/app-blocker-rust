//! Bootstrap Module
//! Inisialisasi semua komponen aplikasi: database, repositories, config, engine
//!
//! ALL config from database - no TOML/.env
//!
//! Usage:
//! ```rust
//! use app_blocker_lib::bootstrap::AppBootstrap;
//!
//! let app = AppBootstrap::new()
//!     .run()
//!     .await?;
//! ```

use crate::config::settings::AppConfig;
use crate::config::DbConfigLoader;
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
use crate::security::auth::{Argon2AuthService, AuthManager, DEFAULT_PASSWORD};
use crate::system::process::WindowsProcessService;
use crate::utils::error::{AppError, AppResult};
use sea_orm::DatabaseConnection;
use std::sync::mpsc;
use std::sync::{Arc, RwLock};
use tracing::{error, info};

/// Application bootstrap - handles initialization of all components
/// ALL config from database - no TOML/.env
pub struct AppBootstrap {
    /// Database connection (initialized after init)
    db: Option<DatabaseConnection>,
    /// Config loader from database
    config_loader: Option<DbConfigLoader>,
    /// App config (Arc for sharing)
    config: Option<Arc<RwLock<AppConfig>>>,
    /// State manager
    state_manager: Option<Arc<StateManager>>,
    /// Event channel sender
    event_tx: Option<mpsc::Sender<AppEvent>>,
    /// Password hash loaded from DB
    password_hash: Option<String>,
}

impl AppBootstrap {
    /// Create new bootstrap instance
    pub fn new() -> Self {
        Self {
            db: None,
            config_loader: None,
            config: None,
            state_manager: None,
            event_tx: None,
            password_hash: None,
        }
    }

    /// Run bootstrap - init DB, load config, return Application
    pub async fn run(mut self) -> AppResult<Application> {
        // Step 1: Initialize database
        self = self.init_database().await?;

        // Step 2: Load config from DB (including password hash)
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

        // Extract values dengan safe handling
        let db = self
            .db
            .take()
            .ok_or_else(|| AppError::Config("Database not initialized".to_string()))?;
        let config = self
            .config
            .take()
            .ok_or_else(|| AppError::Config("Config not loaded".to_string()))?;
        let config_loader = self
            .config_loader
            .take()
            .ok_or_else(|| AppError::Config("Config loader not initialized".to_string()))?;
        let state_manager = self
            .state_manager
            .take()
            .ok_or_else(|| AppError::Config("State manager not initialized".to_string()))?;
        let event_tx = self
            .event_tx
            .take()
            .ok_or_else(|| AppError::Config("Event channel not initialized".to_string()))?;
        let password_hash = self
            .password_hash
            .take()
            .ok_or_else(|| AppError::Config("Password hash not loaded".to_string()))?;

        Ok(Application {
            db,
            config,
            config_loader,
            state_manager,
            event_tx,
            password_hash,
            repositories,
            engine,
        })
    }

    /// Initialize database and run migrations
    pub async fn init_database(mut self) -> AppResult<Self> {
        let db_path = paths::get_db_path();

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

    /// Load configuration from database (including password hash)
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

        // Load password hash from DB (or generate if not exists)
        let config_repo = ConfigRepository::new(db.clone());
        let password_hash = match config_repo.get_value("security.password_hash").await {
            Ok(Some(hash)) if !hash.is_empty() => hash,
            _ => {
                // Generate hash baru dari default password
                info!(
                    "Password hash belum ada di DB, generate dari default password {}...",
                    DEFAULT_PASSWORD
                );
                let (_svc, hash) = Argon2AuthService::with_default_password()?;
                // Simpan ke DB
                let _ = config_repo
                    .set("security.password_hash", &hash, Some("Admin password hash"))
                    .await;
                hash
            }
        };

        info!("Configuration loaded from database - password hash ready");

        self.config_loader = Some(config_loader);
        self.config = Some(config);
        self.password_hash = Some(password_hash);
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

    /// Build the engine with all dependencies (using password hash from DB)
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

        // Create auth manager - use password hash from DB
        let password_hash = self
            .password_hash
            .as_ref()
            .ok_or_else(|| AppError::Config("Password hash not loaded".to_string()))?;

        let auth_svc = Argon2AuthService::new(password_hash.clone())?;

        // Safe read config with error handling
        let config_guard = self
            .config
            .as_ref()
            .ok_or_else(|| AppError::Config("Config not loaded".to_string()))?
            .read()
            .map_err(|e| AppError::Config(format!("Failed to read config: {}", e)))?;

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

    /// Get database connection
    pub fn get_db(&self) -> AppResult<DatabaseConnection> {
        self.db
            .clone()
            .ok_or_else(|| AppError::Config("Database not initialized".to_string()))
    }

    /// Get config arc
    pub fn get_config_arc(&self) -> AppResult<Arc<RwLock<AppConfig>>> {
        self.config
            .clone()
            .ok_or_else(|| AppError::Config("Config not loaded".to_string()))
    }

    /// Quick initialization with just database
    pub async fn quick_init() -> AppResult<Application> {
        Self::new().run().await
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
    /// Password hash from DB (for auth)
    pub password_hash: String,
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
