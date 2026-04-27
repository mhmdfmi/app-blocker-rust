//! Log Models - Event logs and audit logs
use sea_orm::entity::prelude::*;

pub mod audit_log {
    use sea_orm::entity::prelude::*;

    #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
    #[sea_orm(table_name = "audit_logs")]
    pub struct Model {
        #[sea_orm(column_name = "id", primary_key)]
        pub id: i32,

        #[sea_orm(column_name = "timestamp")]
        pub timestamp: String,

        #[sea_orm(column_name = "event_type")]
        pub event_type: String,

        #[sea_orm(column_name = "user_id")]
        pub user_id: Option<i32>,

        #[sea_orm(column_name = "username")]
        pub username: Option<String>,

        #[sea_orm(column_name = "ip_address")]
        pub ip_address: Option<String>,

        #[sea_orm(column_name = "details")]
        pub details: Option<String>, // JSON

        #[sea_orm(column_name = "success")]
        pub success: bool,

        #[sea_orm(column_name = "created_at")]
        pub created_at: String,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}
}

/// Main event log table
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "logs")]
pub struct Model {
    #[sea_orm(column_name = "id", primary_key)]
    pub id: i32,

    #[sea_orm(column_name = "timestamp")]
    pub timestamp: String,

    #[sea_orm(column_name = "process_name")]
    pub process_name: String,

    #[sea_orm(column_name = "process_path")]
    pub process_path: Option<String>,

    #[sea_orm(column_name = "action")]
    pub action: String, // "blocked", "allowed", "warning", "error"

    #[sea_orm(column_name = "reason")]
    pub reason: Option<String>,

    #[sea_orm(column_name = "score")]
    pub score: Option<i32>,

    #[sea_orm(column_name = "device_id")]
    pub device_id: Option<String>,

    #[sea_orm(column_name = "user_id")]
    pub user_id: Option<i32>,

    #[sea_orm(column_name = "created_at")]
    pub created_at: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

pub use audit_log::Entity as AuditLogEntity;
pub use Entity as LogEntity;
