<<<<<<< HEAD
/// Modul keamanan - autentikasi, enkripsi, memori, integritas
pub mod auth;
pub mod encryption;
pub mod integrity;
pub mod memory;

pub use auth::{AuthManager, AuthService, AuthStatus, Argon2AuthService};
pub use encryption::{hash_bytes, hash_file, hash_string};
pub use integrity::IntegrityService;
pub use memory::{SecureBuffer, SecureString};
=======
﻿//! Security Module
//! 
//! Modul keamanan termasuk authentication, encryption, dan memory security.

pub mod auth;
pub mod encryption;
pub mod memory;
pub mod integrity;
>>>>>>> bce0345919f371d153ccb843f2ddbfb5e8695c5f
