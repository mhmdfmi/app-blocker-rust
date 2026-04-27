//! Config Repository
//! CRUD operations untuk konfigurasi key-value

use crate::models::config::{Entity as ConfigEntity, Model as Config};
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter};

/// Config Repository - menangani semua operasi CRUD untuk konfigurasi
pub struct ConfigRepository {
    db: DatabaseConnection,
}

impl ConfigRepository {
    /// Create new config repository
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    /// Get config value by key
    pub async fn get(&self, key: &str) -> Result<Option<Config>, DbErr> {
        ConfigEntity::find_by_id(key).one(&self.db).await
    }

    /// Get config value as string
    pub async fn get_value(&self, key: &str) -> Result<Option<String>, DbErr> {
        let config = self.get(key).await?;
        Ok(config.map(|c| c.value))
    }

    /// Get config value as i32
    pub async fn get_i32(&self, key: &str) -> Result<Option<i32>, DbErr> {
        let value = self.get_value(key).await?;
        Ok(value.and_then(|v| v.parse().ok()))
    }

    /// Get config value as bool
    pub async fn get_bool(&self, key: &str) -> Result<Option<bool>, DbErr> {
        let value = self.get_value(key).await?;
        Ok(value.and_then(|v| match v.to_lowercase().as_str() {
            "true" | "1" | "yes" => Some(true),
            "false" | "0" | "no" => Some(false),
            _ => None,
        }))
    }

    /// Get all configs
    pub async fn find_all(&self) -> Result<Vec<Config>, DbErr> {
        ConfigEntity::find().all(&self.db).await
    }

    /// Set config value (insert or update) - simplified version
    pub async fn set(
        &self,
        key: &str,
        value: &str,
        description: Option<&str>,
    ) -> Result<Config, DbErr> {
        // Check if exists
        if let Some(existing) = self.get(key).await? {
            // Update
            let mut config: crate::models::config::ActiveModel = existing.into();
            config.value = sea_orm::Set(value.to_string());
            config.description = sea_orm::Set(description.map(|s| s.to_string()));
            config.updated_at =
                sea_orm::Set(chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string());
            config.update(&self.db).await?;
        } else {
            // Insert
            let config = crate::models::config::ActiveModel {
                key: sea_orm::Set(key.to_string()),
                value: sea_orm::Set(value.to_string()),
                description: sea_orm::Set(description.map(|s| s.to_string())),
                updated_at: sea_orm::Set(
                    chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
                ),
            };
            ConfigEntity::insert(config).exec(&self.db).await?;
        }

        self.get(key)
            .await?
            .ok_or(DbErr::Custom("Failed to retrieve config".to_string()))
    }

    /// Delete config
    pub async fn delete(&self, key: &str) -> Result<(), DbErr> {
        ConfigEntity::delete_by_id(key.to_string())
            .exec(&self.db)
            .await?;
        Ok(())
    }

    /// Get config by prefix (e.g., "app.", "monitoring.")
    pub async fn get_by_prefix(&self, prefix: &str) -> Result<Vec<Config>, DbErr> {
        ConfigEntity::find()
            .filter(crate::models::config::Column::Key.like(format!("{}%", prefix)))
            .all(&self.db)
            .await
    }

    /// Get all app configs as a HashMap
    pub async fn get_app_configs(
        &self,
    ) -> Result<std::collections::HashMap<String, String>, DbErr> {
        let configs = self.get_by_prefix("").await?;
        let mut map = std::collections::HashMap::new();
        for config in configs {
            map.insert(config.key, config.value);
        }
        Ok(map)
    }

    /// Get nested config (e.g., "monitoring.scan_interval_ms")
    pub async fn get_nested(&self, section: &str, key: &str) -> Result<Option<String>, DbErr> {
        let full_key = format!("{}.{}", section, key);
        self.get_value(&full_key).await
    }

    /// Set nested config
    pub async fn set_nested(&self, section: &str, key: &str, value: &str) -> Result<Config, DbErr> {
        let full_key = format!("{}.{}", section, key);
        self.set(&full_key, value, None).await
    }
}
