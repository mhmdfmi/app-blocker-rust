//! Blacklist Path Model
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "blacklist_paths")]
pub struct Model {
    #[sea_orm(column_name = "id", primary_key)]
    pub id: i32,

    #[sea_orm(column_name = "blacklist_id")]
    pub blacklist_id: i32,

    #[sea_orm(column_name = "path")]
    pub path: String,

    #[sea_orm(column_name = "created_at")]
    pub created_at: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

pub use Entity as BlacklistPathEntity;
