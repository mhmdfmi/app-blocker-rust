//! Log Repository
//! CRUD operations untuk logs dan audit logs

use crate::models::log_entity::audit_log::{Entity as AuditLogEntity, Model as AuditLog};
use crate::models::log_entity::{Entity as LogEntity, Model as Log};
use sea_orm::{
    // ActiveModelTrait is not needed - sea_orm::Set is a derive macro
    ColumnTrait,
    DatabaseConnection,
    DbErr,
    EntityTrait,
    PaginatorTrait,
    QueryFilter,
    QueryOrder,
    QuerySelect,
};

/// Pagination result
#[derive(Debug, Clone)]
pub struct PaginatedLogs {
    pub logs: Vec<Log>,
    pub total: i64,
    pub page: i64,
    pub page_size: i64,
    pub total_pages: i64,
}

/// Parameters for creating a new log entry
#[derive(Debug, Clone)]
pub struct CreateLogParams {
    pub process_name: String,
    pub action: String,
    pub reason: Option<String>,
    pub process_path: Option<String>,
    pub score: Option<i32>,
    pub device_id: Option<String>,
    pub user_id: Option<i32>,
}

/// Log Repository - menangani semua operasi CRUD untuk logs
pub struct LogRepository {
    db: DatabaseConnection,
}

impl LogRepository {
    /// Create new log repository
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    // ============ Log CRUD ============

    /// Find log by ID
    pub async fn find_by_id(&self, id: i32) -> Result<Option<Log>, DbErr> {
        LogEntity::find_by_id(id).one(&self.db).await
    }

    /// Get all logs
    pub async fn find_all(&self) -> Result<Vec<Log>, DbErr> {
        LogEntity::find()
            .order_by_desc(crate::models::log_entity::Column::Timestamp)
            .all(&self.db)
            .await
    }

    /// Get logs with pagination
    pub async fn find_paginated(&self, page: i64, page_size: i64) -> Result<PaginatedLogs, DbErr> {
        let offset = (page - 1) * page_size;

        // Get total count
        let total = LogEntity::find().count(&self.db).await?;

        // Get logs with pagination
        let logs = LogEntity::find()
            .order_by_desc(crate::models::log_entity::Column::Timestamp)
            .limit(page_size as u64)
            .offset(offset as u64)
            .all(&self.db)
            .await?;

        let total_pages = (total as f64 / page_size as f64).ceil() as i64;

        Ok(PaginatedLogs {
            logs,
            total: total as i64,
            page,
            page_size,
            total_pages,
        })
    }

    /// Get logs by action (blocked, allowed, warning, error)
    pub async fn find_by_action(&self, action: &str) -> Result<Vec<Log>, DbErr> {
        LogEntity::find()
            .filter(crate::models::log_entity::Column::Action.eq(action))
            .order_by_desc(crate::models::log_entity::Column::Timestamp)
            .all(&self.db)
            .await
    }

    /// Get logs by process name
    pub async fn find_by_process(&self, process_name: &str) -> Result<Vec<Log>, DbErr> {
        LogEntity::find()
            .filter(
                crate::models::log_entity::Column::ProcessName.like(format!("%{}%", process_name)),
            )
            .order_by_desc(crate::models::log_entity::Column::Timestamp)
            .all(&self.db)
            .await
    }

    /// Get logs by date range
    pub async fn find_by_date_range(&self, start: &str, end: &str) -> Result<Vec<Log>, DbErr> {
        LogEntity::find()
            .filter(
                crate::models::log_entity::Column::Timestamp
                    .gte(start.to_string())
                    .and(crate::models::log_entity::Column::Timestamp.lte(end.to_string())),
            )
            .order_by_desc(crate::models::log_entity::Column::Timestamp)
            .all(&self.db)
            .await
    }

    /// Get recent logs (last N)
    pub async fn get_recent(&self, limit: i64) -> Result<Vec<Log>, DbErr> {
        LogEntity::find()
            .order_by_desc(crate::models::log_entity::Column::Timestamp)
            .limit(limit as u64)
            .all(&self.db)
            .await
    }

    /// Get blocked logs
    pub async fn get_blocked(&self) -> Result<Vec<Log>, DbErr> {
        self.find_by_action("blocked").await
    }

    /// Get allowed logs
    pub async fn get_allowed(&self) -> Result<Vec<Log>, DbErr> {
        self.find_by_action("allowed").await
    }

    /// Create new log entry
    pub async fn create(&self, params: CreateLogParams) -> Result<Log, DbErr> {
        let log = crate::models::log_entity::ActiveModel {
            timestamp: sea_orm::Set(chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string()),
            process_name: sea_orm::Set(params.process_name),
            process_path: sea_orm::Set(params.process_path),
            action: sea_orm::Set(params.action),
            reason: sea_orm::Set(params.reason),
            score: sea_orm::Set(params.score),
            device_id: sea_orm::Set(params.device_id),
            user_id: sea_orm::Set(params.user_id),
            ..Default::default()
        };

        let result = LogEntity::insert(log).exec(&self.db).await?;

        self.find_by_id(result.last_insert_id)
            .await?
            .ok_or(DbErr::Custom("Failed to retrieve created log".to_string()))
    }

    /// Log a blocked process
    pub async fn log_blocked(
        &self,
        process_name: &str,
        reason: &str,
        score: Option<i32>,
    ) -> Result<Log, DbErr> {
        self.create(CreateLogParams {
            process_name: process_name.to_string(),
            action: "blocked".to_string(),
            reason: Some(reason.to_string()),
            process_path: None,
            score,
            device_id: None,
            user_id: None,
        })
        .await
    }

    /// Log an allowed process
    pub async fn log_allowed(
        &self,
        process_name: &str,
        reason: Option<&str>,
    ) -> Result<Log, DbErr> {
        self.create(CreateLogParams {
            process_name: process_name.to_string(),
            action: "allowed".to_string(),
            reason: reason.map(|s| s.to_string()),
            process_path: None,
            score: None,
            device_id: None,
            user_id: None,
        })
        .await
    }

    /// Delete log
    pub async fn delete(&self, id: i32) -> Result<(), DbErr> {
        LogEntity::delete_by_id(id).exec(&self.db).await?;
        Ok(())
    }

    /// Clear all logs
    pub async fn clear_all(&self) -> Result<(), DbErr> {
        LogEntity::delete_many().exec(&self.db).await?;
        Ok(())
    }

    /// Clear logs older than N days
    pub async fn clear_older_than_days(&self, days: i64) -> Result<u64, DbErr> {
        let cutoff = chrono::Local::now() - chrono::Duration::days(days);
        let cutoff_str = cutoff.format("%Y-%m-%d %H:%M:%S").to_string();

        let deleted = LogEntity::delete_many()
            .filter(crate::models::log_entity::Column::Timestamp.lt(cutoff_str))
            .exec(&self.db)
            .await?;

        Ok(deleted.rows_affected)
    }

    // ============ Statistics ============

    /// Get log count by action
    pub async fn get_count_by_action(
        &self,
    ) -> Result<std::collections::HashMap<String, i64>, DbErr> {
        let mut counts = std::collections::HashMap::new();

        for action in &["blocked", "allowed", "warning", "error"] {
            let count = LogEntity::find()
                .filter(crate::models::log_entity::Column::Action.eq(*action))
                .count(&self.db)
                .await?;
            counts.insert(action.to_string(), count as i64);
        }

        Ok(counts)
    }

    /// Get unique process names that have been blocked
    pub async fn get_blocked_processes(&self) -> Result<Vec<String>, DbErr> {
        let logs = self.find_by_action("blocked").await?;
        let mut unique: std::collections::HashSet<String> = std::collections::HashSet::new();
        for log in logs {
            unique.insert(log.process_name);
        }
        Ok(unique.into_iter().collect())
    }

    // ============ Audit Log Operations ============

    /// Create audit log
    pub async fn create_audit(
        &self,
        event_type: &str,
        success: bool,
        user_id: Option<i32>,
        username: Option<&str>,
        ip_address: Option<&str>,
        details: Option<&str>,
    ) -> Result<AuditLog, DbErr> {
        let audit = crate::models::log_entity::audit_log::ActiveModel {
            timestamp: sea_orm::Set(chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string()),
            event_type: sea_orm::Set(event_type.to_string()),
            user_id: sea_orm::Set(user_id),
            username: sea_orm::Set(username.map(|s| s.to_string())),
            ip_address: sea_orm::Set(ip_address.map(|s| s.to_string())),
            details: sea_orm::Set(details.map(|s| s.to_string())),
            success: sea_orm::Set(success),
            ..Default::default()
        };

        let result = AuditLogEntity::insert(audit).exec(&self.db).await?;

        AuditLogEntity::find_by_id(result.last_insert_id)
            .one(&self.db)
            .await?
            .ok_or(DbErr::Custom(
                "Failed to retrieve created audit log".to_string(),
            ))
    }

    /// Get all audit logs
    pub async fn get_audit_logs(&self) -> Result<Vec<AuditLog>, DbErr> {
        AuditLogEntity::find()
            .order_by_desc(crate::models::log_entity::audit_log::Column::Timestamp)
            .all(&self.db)
            .await
    }

    /// Get audit logs by event type
    pub async fn get_audit_by_event(&self, event_type: &str) -> Result<Vec<AuditLog>, DbErr> {
        AuditLogEntity::find()
            .filter(crate::models::log_entity::audit_log::Column::EventType.eq(event_type))
            .order_by_desc(crate::models::log_entity::audit_log::Column::Timestamp)
            .all(&self.db)
            .await
    }

    /// Get blocked count for last N days (simplified - returns total blocked)
    pub async fn get_blocked_count(&self, _days: i64) -> Result<i64, DbErr> {
        let count = LogEntity::find()
            .filter(crate::models::log_entity::Column::Action.eq("blocked"))
            .count(&self.db)
            .await?;
        Ok(count as i64)
    }

    /// Get top blocked processes
    pub async fn get_top_blocked(&self, limit: usize) -> Result<Vec<(String, i64)>, DbErr> {
        let logs = self.find_by_action("blocked").await?;
        let mut counts: std::collections::HashMap<String, i64> = std::collections::HashMap::new();
        for log in logs {
            *counts.entry(log.process_name).or_insert(0) += 1;
        }
        let mut vec: Vec<_> = counts.into_iter().collect();
        vec.sort_by_key(|b| std::cmp::Reverse(b.1));
        vec.truncate(limit);
        Ok(vec)
    }

    /// Get audit logs with filter
    pub async fn get_audit_logs_filtered(
        &self,
        user: Option<&str>,
        limit: usize,
    ) -> Result<Vec<AuditLog>, DbErr> {
        let mut query = AuditLogEntity::find()
            .order_by_desc(crate::models::log_entity::audit_log::Column::Timestamp);

        if let Some(u) = user {
            query = query.filter(crate::models::log_entity::audit_log::Column::Username.eq(u));
        }

        query.limit(limit as u64).all(&self.db).await
    }
}
