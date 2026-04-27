//! Blacklist Models - Simplified version
use sea_orm::entity::prelude::*;

// Main Blacklist entity
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "blacklist")]
pub struct Model {
    #[sea_orm(column_name = "id", primary_key)]
    pub id: i32,

    #[sea_orm(column_name = "name")]
    pub name: String,

    #[sea_orm(column_name = "description")]
    pub description: Option<String>,

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

// Re-export entity
pub use Entity as BlacklistEntity;
