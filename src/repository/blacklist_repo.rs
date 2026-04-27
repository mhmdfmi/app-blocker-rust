//! Blacklist Repository
//! CRUD operations untuk blacklist, processes, dan paths

use crate::models::blacklist::{Entity as BlacklistEntity, Model as Blacklist};
use crate::models::blacklist_path::{Entity as BlacklistPathEntity, Model as BlacklistPath};
use crate::models::blacklist_process::{
    Entity as BlacklistProcessEntity, Model as BlacklistProcess,
};
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter};

/// Blacklist with all related data
#[derive(Debug, Clone)]
pub struct BlacklistWithDetails {
    pub blacklist: Blacklist,
    pub processes: Vec<BlacklistProcess>,
    pub paths: Vec<BlacklistPath>,
}

/// Blacklist Repository - menangani semua operasi CRUD untuk blacklist
pub struct BlacklistRepository {
    db: DatabaseConnection,
}

impl BlacklistRepository {
    /// Create new blacklist repository
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    // ============ Blacklist CRUD ============

    /// Find blacklist by ID
    pub async fn find_by_id(&self, id: i32) -> Result<Option<Blacklist>, DbErr> {
        BlacklistEntity::find_by_id(id).one(&self.db).await
    }

    /// Find blacklist by name
    pub async fn find_by_name(&self, name: &str) -> Result<Option<Blacklist>, DbErr> {
        BlacklistEntity::find()
            .filter(crate::models::blacklist::Column::Name.eq(name))
            .one(&self.db)
            .await
    }

    /// Get all blacklists
    pub async fn find_all(&self) -> Result<Vec<Blacklist>, DbErr> {
        BlacklistEntity::find().all(&self.db).await
    }

    /// Get all enabled blacklists
    pub async fn find_enabled(&self) -> Result<Vec<Blacklist>, DbErr> {
        BlacklistEntity::find()
            .filter(crate::models::blacklist::Column::Enabled.eq(true))
            .all(&self.db)
            .await
    }

    /// Create new blacklist
    pub async fn create(
        &self,
        name: &str,
        description: Option<&str>,
        enabled: bool,
    ) -> Result<Blacklist, DbErr> {
        let blacklist = crate::models::blacklist::ActiveModel {
            name: sea_orm::Set(name.to_string()),
            description: sea_orm::Set(description.map(|s| s.to_string())),
            enabled: sea_orm::Set(enabled),
            ..Default::default()
        };

        let result = BlacklistEntity::insert(blacklist).exec(&self.db).await?;

        self.find_by_id(result.last_insert_id)
            .await?
            .ok_or(DbErr::Custom(
                "Failed to retrieve created blacklist".to_string(),
            ))
    }

    /// Update blacklist
    pub async fn update(
        &self,
        id: i32,
        name: Option<&str>,
        description: Option<&str>,
        enabled: Option<bool>,
    ) -> Result<Blacklist, DbErr> {
        let mut blacklist: crate::models::blacklist::ActiveModel = self
            .find_by_id(id)
            .await?
            .ok_or(DbErr::Custom("Blacklist not found".to_string()))?
            .into();

        if let Some(n) = name {
            blacklist.name = sea_orm::Set(n.to_string());
        }
        if let Some(d) = description {
            blacklist.description = sea_orm::Set(Some(d.to_string()));
        }
        if let Some(e) = enabled {
            blacklist.enabled = sea_orm::Set(e);
        }

        blacklist.update(&self.db).await?;

        self.find_by_id(id).await?.ok_or(DbErr::Custom(
            "Blacklist not found after update".to_string(),
        ))
    }

    /// Delete blacklist (cascade deletes processes and paths)
    pub async fn delete(&self, id: i32) -> Result<(), DbErr> {
        BlacklistEntity::delete_by_id(id).exec(&self.db).await?;
        Ok(())
    }

    // ============ Blacklist Process CRUD ============

    /// Add process to blacklist
    pub async fn add_process(
        &self,
        blacklist_id: i32,
        process_name: &str,
    ) -> Result<BlacklistProcess, DbErr> {
        let process = crate::models::blacklist_process::ActiveModel {
            blacklist_id: sea_orm::Set(blacklist_id),
            process_name: sea_orm::Set(process_name.to_string()),
            ..Default::default()
        };

        let result = BlacklistProcessEntity::insert(process)
            .exec(&self.db)
            .await?;

        BlacklistProcessEntity::find_by_id(result.last_insert_id)
            .one(&self.db)
            .await?
            .ok_or(DbErr::Custom(
                "Failed to retrieve created process".to_string(),
            ))
    }

    /// Get all processes for a blacklist
    pub async fn get_processes(&self, blacklist_id: i32) -> Result<Vec<BlacklistProcess>, DbErr> {
        BlacklistProcessEntity::find()
            .filter(crate::models::blacklist_process::Column::BlacklistId.eq(blacklist_id))
            .all(&self.db)
            .await
    }

    /// Get all processes from enabled blacklists
    pub async fn get_all_enabled_processes(&self) -> Result<Vec<String>, DbErr> {
        let blacklists = self.find_enabled().await?;
        let mut processes = Vec::new();

        for blacklist in blacklists {
            let blacklist_processes = self.get_processes(blacklist.id).await?;
            for p in blacklist_processes {
                processes.push(p.process_name);
            }
        }

        Ok(processes)
    }

    /// Delete process from blacklist
    pub async fn delete_process(&self, id: i32) -> Result<(), DbErr> {
        BlacklistProcessEntity::delete_by_id(id)
            .exec(&self.db)
            .await?;
        Ok(())
    }

    /// Clear all processes from a blacklist
    pub async fn clear_processes(&self, blacklist_id: i32) -> Result<(), DbErr> {
        BlacklistProcessEntity::delete_many()
            .filter(crate::models::blacklist_process::Column::BlacklistId.eq(blacklist_id))
            .exec(&self.db)
            .await?;
        Ok(())
    }

    // ============ Blacklist Path CRUD ============

    /// Add path to blacklist
    pub async fn add_path(&self, blacklist_id: i32, path: &str) -> Result<BlacklistPath, DbErr> {
        let path_model = crate::models::blacklist_path::ActiveModel {
            blacklist_id: sea_orm::Set(blacklist_id),
            path: sea_orm::Set(path.to_string()),
            ..Default::default()
        };

        let result = BlacklistPathEntity::insert(path_model)
            .exec(&self.db)
            .await?;

        BlacklistPathEntity::find_by_id(result.last_insert_id)
            .one(&self.db)
            .await?
            .ok_or(DbErr::Custom("Failed to retrieve created path".to_string()))
    }

    /// Get all paths for a blacklist
    pub async fn get_paths(&self, blacklist_id: i32) -> Result<Vec<BlacklistPath>, DbErr> {
        BlacklistPathEntity::find()
            .filter(crate::models::blacklist_path::Column::BlacklistId.eq(blacklist_id))
            .all(&self.db)
            .await
    }

    /// Get all paths from enabled blacklists
    pub async fn get_all_enabled_paths(&self) -> Result<Vec<String>, DbErr> {
        let blacklists = self.find_enabled().await?;
        let mut paths = Vec::new();

        for blacklist in blacklists {
            let blacklist_paths = self.get_paths(blacklist.id).await?;
            for p in blacklist_paths {
                paths.push(p.path);
            }
        }

        Ok(paths)
    }

    /// Delete path from blacklist
    pub async fn delete_path(&self, id: i32) -> Result<(), DbErr> {
        BlacklistPathEntity::delete_by_id(id).exec(&self.db).await?;
        Ok(())
    }

    /// Clear all paths from a blacklist
    pub async fn clear_paths(&self, blacklist_id: i32) -> Result<(), DbErr> {
        BlacklistPathEntity::delete_many()
            .filter(crate::models::blacklist_path::Column::BlacklistId.eq(blacklist_id))
            .exec(&self.db)
            .await?;
        Ok(())
    }

    // ============ Combined Operations ============

    /// Get blacklist with all details (processes and paths)
    pub async fn find_with_details(&self, id: i32) -> Result<Option<BlacklistWithDetails>, DbErr> {
        let blacklist = self.find_by_id(id).await?;

        if let Some(bl) = blacklist {
            let processes = self.get_processes(bl.id).await?;
            let paths = self.get_paths(bl.id).await?;

            Ok(Some(BlacklistWithDetails {
                blacklist: bl,
                processes,
                paths,
            }))
        } else {
            Ok(None)
        }
    }

    /// Get all blacklists with all details
    pub async fn find_all_with_details(&self) -> Result<Vec<BlacklistWithDetails>, DbErr> {
        let blacklists = self.find_all().await?;
        let mut result = Vec::new();

        for bl in blacklists {
            let processes = self.get_processes(bl.id).await?;
            let paths = self.get_paths(bl.id).await?;

            result.push(BlacklistWithDetails {
                blacklist: bl,
                processes,
                paths,
            });
        }

        Ok(result)
    }

    /// Create blacklist with processes and paths
    pub async fn create_with_details(
        &self,
        name: &str,
        description: Option<&str>,
        enabled: bool,
        processes: Vec<&str>,
        paths: Vec<&str>,
    ) -> Result<BlacklistWithDetails, DbErr> {
        // Create blacklist
        let blacklist = self.create(name, description, enabled).await?;

        // Add processes
        for process in processes {
            self.add_process(blacklist.id, process).await?;
        }

        // Add paths
        for path in paths {
            self.add_path(blacklist.id, path).await?;
        }

        // Return with details
        self.find_with_details(blacklist.id)
            .await?
            .ok_or(DbErr::Custom(
                "Failed to retrieve blacklist details".to_string(),
            ))
    }
}
