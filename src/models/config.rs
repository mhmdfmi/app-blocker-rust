//! Config Model (Key-Value Store)
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "configs")]
pub struct Model {
    #[sea_orm(column_name = "key", primary_key)]
    pub key: String,

    #[sea_orm(column_name = "value")]
    pub value: String,

    #[sea_orm(column_name = "description")]
    pub description: Option<String>,

    #[sea_orm(column_name = "updated_at")]
    pub updated_at: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

/// Helper methods for Config model
impl Model {
    /// Parse value as string
    pub fn as_str(&self) -> &str {
        &self.value
    }

    /// Parse value as i32
    pub fn as_i32(&self) -> Option<i32> {
        self.value.parse().ok()
    }

    /// Parse value as bool
    pub fn as_bool(&self) -> Option<bool> {
        match self.value.to_lowercase().as_str() {
            "true" | "1" | "yes" | "on" => Some(true),
            "false" | "0" | "no" | "off" => Some(false),
            _ => None,
        }
    }
}
