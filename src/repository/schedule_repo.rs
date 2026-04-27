//! Schedule Repository
//! CRUD operations untuk schedule dan schedule rules

use crate::models::schedule::{Entity as ScheduleEntity, Model as Schedule};
use crate::models::schedule_rule::{Entity as ScheduleRuleEntity, Model as ScheduleRule};
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter};

/// Schedule with all rules
#[derive(Debug, Clone)]
pub struct ScheduleWithRules {
    pub schedule: Schedule,
    pub rules: Vec<ScheduleRule>,
}

/// Schedule Repository - menangani semua operasi CRUD untuk schedule
pub struct ScheduleRepository {
    db: DatabaseConnection,
}

impl ScheduleRepository {
    /// Create new schedule repository
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    // ============ Schedule CRUD ============

    /// Find schedule by ID
    pub async fn find_by_id(&self, id: i32) -> Result<Option<Schedule>, DbErr> {
        ScheduleEntity::find_by_id(id).one(&self.db).await
    }

    /// Get schedule (usually just ID 1 for global schedule)
    pub async fn find_global(&self) -> Result<Option<Schedule>, DbErr> {
        ScheduleEntity::find_by_id(1).one(&self.db).await
    }

    /// Get all schedules
    pub async fn find_all(&self) -> Result<Vec<Schedule>, DbErr> {
        ScheduleEntity::find().all(&self.db).await
    }

    /// Create new schedule
    pub async fn create(&self, enabled: bool, timezone: &str) -> Result<Schedule, DbErr> {
        let schedule = crate::models::schedule::ActiveModel {
            enabled: sea_orm::Set(enabled),
            timezone: sea_orm::Set(timezone.to_string()),
            ..Default::default()
        };

        let result = ScheduleEntity::insert(schedule).exec(&self.db).await?;

        self.find_by_id(result.last_insert_id)
            .await?
            .ok_or(DbErr::Custom(
                "Failed to retrieve created schedule".to_string(),
            ))
    }

    /// Update schedule
    pub async fn update(
        &self,
        id: i32,
        enabled: Option<bool>,
        timezone: Option<&str>,
    ) -> Result<Schedule, DbErr> {
        let mut schedule: crate::models::schedule::ActiveModel = self
            .find_by_id(id)
            .await?
            .ok_or(DbErr::Custom("Schedule not found".to_string()))?
            .into();

        if let Some(e) = enabled {
            schedule.enabled = sea_orm::Set(e);
        }
        if let Some(tz) = timezone {
            schedule.timezone = sea_orm::Set(tz.to_string());
        }

        schedule.update(&self.db).await?;

        self.find_by_id(id)
            .await?
            .ok_or(DbErr::Custom("Schedule not found after update".to_string()))
    }

    /// Delete schedule
    pub async fn delete(&self, id: i32) -> Result<(), DbErr> {
        ScheduleEntity::delete_by_id(id).exec(&self.db).await?;
        Ok(())
    }

    // ============ Schedule Rule CRUD ============

    /// Find rule by ID
    pub async fn find_rule_by_id(&self, id: i32) -> Result<Option<ScheduleRule>, DbErr> {
        ScheduleRuleEntity::find_by_id(id).one(&self.db).await
    }

    /// Get all rules for a schedule
    pub async fn get_rules(&self, schedule_id: i32) -> Result<Vec<ScheduleRule>, DbErr> {
        ScheduleRuleEntity::find()
            .filter(crate::models::schedule_rule::Column::ScheduleId.eq(schedule_id))
            .all(&self.db)
            .await
    }

    /// Get all enabled rules for a schedule
    pub async fn get_enabled_rules(&self, schedule_id: i32) -> Result<Vec<ScheduleRule>, DbErr> {
        ScheduleRuleEntity::find()
            .filter(
                crate::models::schedule_rule::Column::ScheduleId
                    .eq(schedule_id)
                    .and(crate::models::schedule_rule::Column::Enabled.eq(true)),
            )
            .all(&self.db)
            .await
    }

    /// Create new schedule rule
    pub async fn create_rule(
        &self,
        schedule_id: i32,
        days: &str, // JSON array string
        start_time: &str,
        end_time: &str,
        action: &str,
        enabled: bool,
    ) -> Result<ScheduleRule, DbErr> {
        let rule = crate::models::schedule_rule::ActiveModel {
            schedule_id: sea_orm::Set(schedule_id),
            days: sea_orm::Set(days.to_string()),
            start_time: sea_orm::Set(start_time.to_string()),
            end_time: sea_orm::Set(end_time.to_string()),
            action: sea_orm::Set(action.to_string()),
            enabled: sea_orm::Set(enabled),
            ..Default::default()
        };

        let result = ScheduleRuleEntity::insert(rule).exec(&self.db).await?;

        self.find_rule_by_id(result.last_insert_id)
            .await?
            .ok_or(DbErr::Custom("Failed to retrieve created rule".to_string()))
    }

    /// Update schedule rule
    pub async fn update_rule(
        &self,
        id: i32,
        days: Option<&str>,
        start_time: Option<&str>,
        end_time: Option<&str>,
        action: Option<&str>,
        enabled: Option<bool>,
    ) -> Result<ScheduleRule, DbErr> {
        let mut rule: crate::models::schedule_rule::ActiveModel = self
            .find_rule_by_id(id)
            .await?
            .ok_or(DbErr::Custom("Rule not found".to_string()))?
            .into();

        if let Some(d) = days {
            rule.days = sea_orm::Set(d.to_string());
        }
        if let Some(st) = start_time {
            rule.start_time = sea_orm::Set(st.to_string());
        }
        if let Some(et) = end_time {
            rule.end_time = sea_orm::Set(et.to_string());
        }
        if let Some(a) = action {
            rule.action = sea_orm::Set(a.to_string());
        }
        if let Some(e) = enabled {
            rule.enabled = sea_orm::Set(e);
        }

        rule.update(&self.db).await?;

        self.find_rule_by_id(id)
            .await?
            .ok_or(DbErr::Custom("Rule not found after update".to_string()))
    }

    /// Delete schedule rule
    pub async fn delete_rule(&self, id: i32) -> Result<(), DbErr> {
        ScheduleRuleEntity::delete_by_id(id).exec(&self.db).await?;
        Ok(())
    }

    /// Clear all rules for a schedule
    pub async fn clear_rules(&self, schedule_id: i32) -> Result<(), DbErr> {
        ScheduleRuleEntity::delete_many()
            .filter(crate::models::schedule_rule::Column::ScheduleId.eq(schedule_id))
            .exec(&self.db)
            .await?;
        Ok(())
    }

    // ============ Combined Operations ============

    /// Get schedule with all rules
    pub async fn find_with_rules(&self, id: i32) -> Result<Option<ScheduleWithRules>, DbErr> {
        let schedule = self.find_by_id(id).await?;

        if let Some(s) = schedule {
            let rules = self.get_rules(s.id).await?;

            Ok(Some(ScheduleWithRules { schedule: s, rules }))
        } else {
            Ok(None)
        }
    }

    /// Get global schedule with all rules
    pub async fn get_global_with_rules(&self) -> Result<Option<ScheduleWithRules>, DbErr> {
        self.find_with_rules(1).await
    }

    /// Check if current time matches any enabled schedule rule
    pub async fn should_block_now(&self, schedule_id: i32) -> Result<bool, DbErr> {
        let rules = self.get_enabled_rules(schedule_id).await?;

        if rules.is_empty() {
            return Ok(false);
        }

        // Get current time in the schedule's timezone
        let schedule = self.find_by_id(schedule_id).await?;
        let _timezone = schedule
            .map(|s| s.timezone)
            .unwrap_or_else(|| "UTC".to_string());

        // Parse current time (simplified - in production use chrono-tz)
        let now = chrono::Local::now();
        let current_day = now.format("%A").to_string();
        let current_time = now.format("%H:%M").to_string();

        for rule in rules {
            // Parse days JSON array
            let days: Vec<String> = serde_json::from_str(&rule.days).unwrap_or_default();

            // Check if current day matches
            if days.contains(&current_day) {
                // Check if current time is within range
                if current_time >= rule.start_time && current_time <= rule.end_time {
                    return Ok(true);
                }
            }
        }

        Ok(false)
    }

    /// Check if global schedule should block now
    pub async fn global_should_block_now(&self) -> Result<bool, DbErr> {
        // Check if schedule is enabled
        let schedule = self.find_global().await?;

        if let Some(s) = schedule {
            if !s.enabled {
                return Ok(false);
            }
            return self.should_block_now(s.id).await;
        }

        Ok(false)
    }

    /// Create schedule with rules
    pub async fn create_with_rules(
        &self,
        enabled: bool,
        timezone: &str,
        rules: Vec<(&str, &str, &str, &str)>, // (days_json, start, end, action)
    ) -> Result<ScheduleWithRules, DbErr> {
        // Create schedule
        let schedule = self.create(enabled, timezone).await?;

        // Add rules
        for (days, start, end, action) in rules {
            self.create_rule(schedule.id, days, start, end, action, true)
                .await?;
        }

        // Return with rules
        self.find_with_rules(schedule.id)
            .await?
            .ok_or(DbErr::Custom(
                "Failed to retrieve schedule with rules".to_string(),
            ))
    }
}
