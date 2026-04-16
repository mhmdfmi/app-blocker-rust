//! Authentication Module
//! 
//! Modul untuk autentikasi dengan Argon2 dan PBKDF2.

use crate::utils::error::{AppResult, AppError};
use argon2::{Argon2, PasswordHasher, password_hash::PasswordHash, password_hash::PasswordVerifier};
use parking_lot::Mutex;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

/// Authenticator untuk validasi password
pub struct Authenticator {
    max_attempts: u32,
    lockout_duration_secs: u64,
    failed_attempts: Arc<Mutex<u32>>,
    lockout_until: Arc<Mutex<Option<u64>>>,
}

impl Authenticator {
    /// Buat authenticator baru
    pub fn new(max_attempts: u32, lockout_duration_secs: u64) -> Self {
        Self {
            max_attempts,
            lockout_duration_secs,
            failed_attempts: Arc::new(Mutex::new(0)),
            lockout_until: Arc::new(Mutex::new(None)),
        }
    }
    
    /// Validasi password
    pub fn verify(&self, password: &str, hash: &str) -> AppResult<bool> {
        // Check lockout
        if let Some(lockout_time) = *self.lockout_until.lock() {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_err(|e| AppError::AuthError(e.to_string()))?
                .as_secs();
            
            if now < lockout_time {
                return Err(AppError::AuthError(
                    "Account is locked. Please wait.".to_string()
                ));
            }
            
            // Lockout expired
            *self.lockout_until.lock() = None;
            *self.failed_attempts.lock() = 0;
        }
        
        // Parse hash
        let parsed_hash = PasswordHash::new(hash)
            .map_err(|e| AppError::AuthError(format!("Invalid hash: {}", e)))?;
        
        // Verify password
        let argon2 = Argon2::default();
        let result = argon2.verify_password(password.as_bytes(), &parsed_hash);
        
        match result {
            Ok(_) => {
                *self.failed_attempts.lock() = 0;
                Ok(true)
            }
            Err(_) => {
                let mut attempts = self.failed_attempts.lock();
                *attempts += 1;
                
                if *attempts >= self.max_attempts {
                    // Set lockout
                    let lockout_time = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .map_err(|e| AppError::AuthError(e.to_string()))?
                        .as_secs() + self.lockout_duration_secs;
                    *self.lockout_until.lock() = Some(lockout_time);
                    
                    tracing::error!("Max attempts reached, account locked for {} seconds", 
                        self.lockout_duration_secs);
                }
                
                Ok(false)
            }
        }
    }
    
    /// Get failed attempts
    pub fn get_failed_attempts(&self) -> u32 {
        *self.failed_attempts.lock()
    }
    
    /// Reset attempts
    pub fn reset(&self) {
        *self.failed_attempts.lock() = 0;
        *self.lockout_until.lock() = None;
    }
    
    /// Hash password (untuk setup awal)
    pub fn hash_password(password: &str) -> AppResult<String> {
        let argon2 = Argon2::default();
        let salt = argon2::password_hash::SaltString::generate(&mut rand::thread_rng());
        
        let hash = argon2.hash_password(password.as_bytes(), &salt)
            .map_err(|e| AppError::AuthError(format!("Hash error: {}", e)))?;
        
        Ok(hash.to_string())
    }
}
