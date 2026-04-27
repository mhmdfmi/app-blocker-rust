//! User Model
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "users")]
pub struct Model {
    #[sea_orm(column_name = "id", primary_key)]
    pub id: i32,

    #[sea_orm(column_name = "username")]
    pub username: String,

    #[sea_orm(column_name = "password_hash")]
    pub password_hash: String,

    #[sea_orm(column_name = "role")]
    pub role: String, // 'admin' or 'user'

    #[sea_orm(column_name = "created_at")]
    pub created_at: String,

    #[sea_orm(column_name = "updated_at")]
    pub updated_at: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

/// Helper methods for User model
impl Model {
    pub fn is_admin(&self) -> bool {
        self.role == "admin"
    }
}
