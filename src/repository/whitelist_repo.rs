//! Whitelist Repository
//! CRUD operations untuk whitelist

use crate::models::whitelist::{Entity as WhitelistEntity, Model as Whitelist};
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter};

/// Whitelist Repository - menangani semua operasi CRUD untuk whitelist
pub struct WhitelistRepository {
    db: DatabaseConnection,
}

impl WhitelistRepository {
    /// Create new whitelist repository
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    /// Find whitelist by ID
    pub async fn find_by_id(&self, id: i32) -> Result<Option<Whitelist>, DbErr> {
        WhitelistEntity::find_by_id(id).one(&self.db).await
    }

    /// Find whitelist by process name
    pub async fn find_by_process_name(
        &self,
        process_name: &str,
    ) -> Result<Option<Whitelist>, DbErr> {
        WhitelistEntity::find()
            .filter(crate::models::whitelist::Column::ProcessName.eq(process_name))
            .one(&self.db)
            .await
    }

    /// Get all whitelisted processes
    pub async fn find_all(&self) -> Result<Vec<Whitelist>, DbErr> {
        WhitelistEntity::find().all(&self.db).await
    }

    /// Get all enabled whitelisted processes
    pub async fn find_enabled(&self) -> Result<Vec<Whitelist>, DbErr> {
        WhitelistEntity::find()
            .filter(crate::models::whitelist::Column::Enabled.eq(true))
            .all(&self.db)
            .await
    }

    /// Get all enabled process names as strings
    pub async fn get_enabled_names(&self) -> Result<Vec<String>, DbErr> {
        let whitelists = self.find_enabled().await?;
        Ok(whitelists.into_iter().map(|w| w.process_name).collect())
    }

    /// Check if process is whitelisted
    pub async fn is_whitelisted(&self, process_name: &str) -> Result<bool, DbErr> {
        let whitelist = self.find_by_process_name(process_name).await?;
        Ok(whitelist.map(|w| w.enabled).unwrap_or(false))
    }

    /// Create new whitelist entry
    pub async fn create(
        &self,
        process_name: &str,
        description: Option<&str>,
        enabled: bool,
    ) -> Result<Whitelist, DbErr> {
        let whitelist = crate::models::whitelist::ActiveModel {
            process_name: sea_orm::Set(process_name.to_string()),
            description: sea_orm::Set(description.map(|s| s.to_string())),
            enabled: sea_orm::Set(enabled),
            ..Default::default()
        };

        WhitelistEntity::insert(whitelist).exec(&self.db).await?;

        self.find_by_process_name(process_name)
            .await?
            .ok_or(DbErr::Custom(
                "Failed to retrieve created whitelist".to_string(),
            ))
    }

    /// Update whitelist
    pub async fn update(
        &self,
        id: i32,
        process_name: Option<&str>,
        description: Option<&str>,
        enabled: Option<bool>,
    ) -> Result<Whitelist, DbErr> {
        let mut whitelist: crate::models::whitelist::ActiveModel = self
            .find_by_id(id)
            .await?
            .ok_or(DbErr::Custom("Whitelist not found".to_string()))?
            .into();

        if let Some(pn) = process_name {
            whitelist.process_name = sea_orm::Set(pn.to_string());
        }
        if let Some(d) = description {
            whitelist.description = sea_orm::Set(Some(d.to_string()));
        }
        if let Some(e) = enabled {
            whitelist.enabled = sea_orm::Set(e);
        }

        whitelist.update(&self.db).await?;

        self.find_by_id(id).await?.ok_or(DbErr::Custom(
            "Whitelist not found after update".to_string(),
        ))
    }

    /// Delete whitelist
    pub async fn delete(&self, id: i32) -> Result<(), DbErr> {
        WhitelistEntity::delete_by_id(id).exec(&self.db).await?;
        Ok(())
    }

    /// Add multiple processes to whitelist
    pub async fn create_bulk(&self, process_names: Vec<&str>) -> Result<Vec<Whitelist>, DbErr> {
        let mut results = Vec::new();

        for name in process_names {
            // Skip if already exists
            if self.find_by_process_name(name).await?.is_none() {
                let whitelist = self.create(name, None, true).await?;
                results.push(whitelist);
            }
        }

        Ok(results)
    }
}
