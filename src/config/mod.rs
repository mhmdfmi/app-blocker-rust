/// Modul konfigurasi dengan hot reload dan validasi otomatis.
pub mod db_loader;
pub mod db_reload_watcher;
pub mod env_loader;
pub mod hot_reload;
pub mod settings;
pub mod validator;

pub use db_loader::DbConfigLoader;
pub use db_reload_watcher::spawn_db_config_watcher_with_loader;

use crate::utils::error::{AppError, AppResult};
use settings::AppConfig;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use tracing::{error, info, warn};

/// Manager konfigurasi thread-safe dengan dukungan hot reload
pub struct ConfigManager {
    config: Arc<RwLock<AppConfig>>,
    config_path: PathBuf,
    last_checksum: Arc<RwLock<String>>,
}

impl ConfigManager {
    /// Muat konfigurasi dari file
    pub fn load(config_path: &Path) -> AppResult<Self> {
        let config = load_config_from_file(config_path)?;
        let checksum = compute_checksum(&config)?;

        info!(
            path = %config_path.display(),
            mode = %config.app.mode,
            "Konfigurasi berhasil dimuat"
        );

        Ok(Self {
            config: Arc::new(RwLock::new(config)),
            config_path: config_path.to_path_buf(),
            last_checksum: Arc::new(RwLock::new(checksum)),
        })
    }

    /// Dapatkan konfigurasi saat ini (clone untuk menghindari lock lama)
    pub fn get(&self) -> AppResult<AppConfig> {
        self.config
            .read()
            .map(|c| c.clone())
            .map_err(|e| AppError::Config(format!("Gagal baca konfigurasi: {e}")))
    }

    /// Reload konfigurasi dengan safe swap (rollback jika gagal)
    pub fn hot_reload(&self) -> AppResult<bool> {
        match load_config_from_file(&self.config_path) {
            Ok(new_config) => {
                let new_checksum = compute_checksum(&new_config)?;

                let current_checksum = self
                    .last_checksum
                    .read()
                    .map_err(|e| AppError::Config(format!("Lock checksum: {e}")))?;

                if *current_checksum == new_checksum {
                    return Ok(false);
                }
                drop(current_checksum);

                let mut config_guard = self
                    .config
                    .write()
                    .map_err(|e| AppError::Config(format!("Lock write config: {e}")))?;
                *config_guard = new_config;
                drop(config_guard);

                let mut checksum_guard = self
                    .last_checksum
                    .write()
                    .map_err(|e| AppError::Config(format!("Lock write checksum: {e}")))?;
                *checksum_guard = new_checksum;

                info!("Konfigurasi berhasil di-reload");
                Ok(true)
            }
            Err(e) => {
                error!(error = %e, "Gagal reload konfigurasi, mempertahankan konfigurasi lama");
                Err(e)
            }
        }
    }

    /// Dapatkan Arc ke config untuk digunakan di thread lain
    pub fn get_arc(&self) -> Arc<RwLock<AppConfig>> {
        Arc::clone(&self.config)
    }

    /// Path file konfigurasi
    pub fn config_path(&self) -> &Path {
        &self.config_path
    }
}

/// Muat konfigurasi dari file TOML
fn load_config_from_file(path: &Path) -> AppResult<AppConfig> {
    if !path.exists() {
        warn!(
            path = %path.display(),
            "File konfigurasi tidak ditemukan, menggunakan default"
        );
        let default_config = AppConfig::default();
        validator::validate_config(&default_config)?;
        return Ok(default_config);
    }

    let content = std::fs::read_to_string(path)
        .map_err(|e| AppError::io(format!("Baca config {}", path.display()), e))?;

    let config: AppConfig = toml::from_str(&content)
        .map_err(|e| AppError::Config(format!("Parse config TOML: {e}")))?;

    validator::validate_config(&config)?;
    Ok(config)
}

/// Hitung checksum konfigurasi untuk deteksi perubahan
fn compute_checksum(config: &AppConfig) -> AppResult<String> {
    use sha2::{Digest, Sha256};
    let json = serde_json::to_string(config)
        .map_err(|e| AppError::Serialization(format!("Checksum config: {e}")))?;
    let hash = Sha256::digest(json.as_bytes());
    Ok(hex::encode(hash))
}
