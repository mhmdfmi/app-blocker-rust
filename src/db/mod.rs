//! Database module for App Blocker
//! Version: 1.2.0

pub mod connection;
pub mod init;

pub use connection::{create_db_pool, get_db_connection, DbPool};
pub use init::{init_database, run_migrations, seed_default_data};
