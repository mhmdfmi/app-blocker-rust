//! Schedule Rule Model
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "schedule_rules")]
pub struct Model {
    #[sea_orm(column_name = "id", primary_key)]
    pub id: i32,

    #[sea_orm(column_name = "schedule_id")]
    pub schedule_id: i32,

    #[sea_orm(column_name = "days")]
    pub days: String, // JSON array: ["Monday", "Tuesday"]

    #[sea_orm(column_name = "start_time")]
    pub start_time: String, // HH:MM format

    #[sea_orm(column_name = "end_time")]
    pub end_time: String, // HH:MM format

    #[sea_orm(column_name = "action")]
    pub action: String, // "block_games", "block_all", etc.

    #[sea_orm(column_name = "enabled")]
    pub enabled: bool,

    #[sea_orm(column_name = "created_at")]
    pub created_at: String,

    #[sea_orm(column_name = "updated_at")]
    pub updated_at: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

pub use Entity as ScheduleRuleEntity;
