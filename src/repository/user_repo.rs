//! User Repository
//! CRUD operations untuk user

use crate::models::user::{Entity as UserEntity, Model as User};
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter};
/// User Repository - menangani semua operasi CRUD untuk user
pub struct UserRepository {
    db: DatabaseConnection,
}

impl UserRepository {
    /// Create new user repository
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    /// Find user by ID
    pub async fn find_by_id(&self, id: i32) -> Result<Option<User>, DbErr> {
        UserEntity::find_by_id(id).one(&self.db).await
    }

    /// Find user by username
    pub async fn find_by_username(&self, username: &str) -> Result<Option<User>, DbErr> {
        UserEntity::find()
            .filter(crate::models::user::Column::Username.eq(username))
            .one(&self.db)
            .await
    }

    /// Get all users
    pub async fn find_all(&self) -> Result<Vec<User>, DbErr> {
        UserEntity::find().all(&self.db).await
    }

    /// Create new user
    pub async fn create(
        &self,
        username: &str,
        password_hash: &str,
        role: &str,
    ) -> Result<User, DbErr> {
        let user = crate::models::user::ActiveModel {
            username: sea_orm::Set(username.to_string()),
            password_hash: sea_orm::Set(password_hash.to_string()),
            role: sea_orm::Set(role.to_string()),
            ..Default::default()
        };

        UserEntity::insert(user).exec(&self.db).await?;

        // Return the created user
        self.find_by_username(username)
            .await?
            .ok_or(DbErr::Custom("Failed to retrieve created user".to_string()))
    }

    /// Update user
    pub async fn update(
        &self,
        id: i32,
        username: Option<&str>,
        password_hash: Option<&str>,
        role: Option<&str>,
    ) -> Result<User, DbErr> {
        let mut user: crate::models::user::ActiveModel = self
            .find_by_id(id)
            .await?
            .ok_or(DbErr::Custom("User not found".to_string()))?
            .into();

        if let Some(u) = username {
            user.username = sea_orm::Set(u.to_string());
        }
        if let Some(p) = password_hash {
            user.password_hash = sea_orm::Set(p.to_string());
        }
        if let Some(r) = role {
            user.role = sea_orm::Set(r.to_string());
        }

        user.update(&self.db).await?;

        self.find_by_id(id)
            .await?
            .ok_or(DbErr::Custom("User not found after update".to_string()))
    }

    /// Delete user
    pub async fn delete(&self, id: i32) -> Result<(), DbErr> {
        UserEntity::delete_by_id(id).exec(&self.db).await?;
        Ok(())
    }

    /// Check if username exists
    pub async fn username_exists(&self, username: &str) -> Result<bool, DbErr> {
        let user = self.find_by_username(username).await?;
        Ok(user.is_some())
    }

    /// Verify password (simple comparison - in production use argon2 verify)
    pub async fn verify_password(
        &self,
        username: &str,
        password: &str,
    ) -> Result<Option<User>, DbErr> {
        let user = self.find_by_username(username).await?;

        if let Some(user) = user {
            // Use argon2 to verify password
            use argon2::{Argon2, PasswordVerifier};
            use zeroize::Zeroizing;

            let password_bytes = Zeroizing::new(password.as_bytes().to_vec());

            // Check if hash starts with $argon2
            if user.password_hash.starts_with("$argon2") {
                if let Ok(parsed_hash) = argon2::PasswordHash::new(&user.password_hash) {
                    if Argon2::default()
                        .verify_password(&password_bytes, &parsed_hash)
                        .is_ok()
                    {
                        return Ok(Some(user));
                    }
                }
            }

            return Ok(None);
        }

        Ok(None)
    }
}
