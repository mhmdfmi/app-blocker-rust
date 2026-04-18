<<<<<<< HEAD
/// Mekanisme retry dengan exponential backoff untuk operasi yang bisa gagal sementara.
use crate::utils::error::{AppError, AppResult};
use std::time::Duration;
use tracing::{debug, warn};

/// Konfigurasi strategi retry
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// Jumlah maksimal percobaan
    pub max_attempts: u32,
    /// Delay awal sebelum retry pertama (ms)
    pub initial_delay_ms: u64,
    /// Faktor pengali delay (exponential backoff)
    pub backoff_factor: f64,
    /// Delay maksimal (ms)
    pub max_delay_ms: u64,
=======
﻿//! Retry Module
//! 
//! Modul untuk retry logic dengan exponential backoff.

use std::time::Duration;
use std::thread;

/// Retry configuration
#[derive(Debug, Clone)]
pub struct RetryConfig {
    pub max_attempts: u32,
    pub initial_delay_ms: u64,
    pub max_delay_ms: u64,
    pub multiplier: f64,
>>>>>>> bce0345919f371d153ccb843f2ddbfb5e8695c5f
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_delay_ms: 100,
<<<<<<< HEAD
            backoff_factor: 2.0,
            max_delay_ms: 5000,
=======
            max_delay_ms: 5000,
            multiplier: 2.0,
>>>>>>> bce0345919f371d153ccb843f2ddbfb5e8695c5f
        }
    }
}

<<<<<<< HEAD
impl RetryConfig {
    /// Konfigurasi untuk operasi kilat process
    pub fn for_process_kill() -> Self {
        Self {
            max_attempts: 3,
            initial_delay_ms: 500,
            backoff_factor: 2.0,
            max_delay_ms: 3000,
        }
    }

    /// Konfigurasi untuk operasi channel
    pub fn for_channel() -> Self {
        Self {
            max_attempts: 5,
            initial_delay_ms: 50,
            backoff_factor: 1.5,
            max_delay_ms: 1000,
        }
    }
}

/// Jalankan fungsi dengan retry otomatis.
///
/// Mengembalikan error terakhir jika semua percobaan gagal.
pub fn with_retry<F, T>(config: &RetryConfig, operation_name: &str, mut f: F) -> AppResult<T>
where
    F: FnMut() -> AppResult<T>,
{
    let mut last_error = AppError::Unknown(format!("Operasi '{operation_name}' belum dijalankan"));
    let mut delay_ms = config.initial_delay_ms;

    for attempt in 1..=config.max_attempts {
        match f() {
            Ok(result) => {
                if attempt > 1 {
                    debug!(
                        operation = operation_name,
                        attempt,
                        "Operasi berhasil setelah retry"
                    );
                }
                return Ok(result);
            }
            Err(e) => {
                last_error = e;
                if attempt < config.max_attempts {
                    if last_error.is_retryable() {
                        warn!(
                            operation = operation_name,
                            attempt,
                            max_attempts = config.max_attempts,
                            delay_ms,
                            error = %last_error,
                            "Operasi gagal, mencoba kembali"
                        );
                        std::thread::sleep(Duration::from_millis(delay_ms));
                        // Hitung delay berikutnya (exponential backoff)
                        delay_ms = ((delay_ms as f64 * config.backoff_factor) as u64)
                            .min(config.max_delay_ms);
                    } else {
                        // Error tidak bisa di-retry, langsung gagal
                        warn!(
                            operation = operation_name,
                            error = %last_error,
                            "Error tidak bisa di-retry, menghentikan"
                        );
                        break;
                    }
=======
/// Retry helper
pub struct Retry {
    config: RetryConfig,
}

impl Retry {
    pub fn new(config: RetryConfig) -> Self {
        Self { config }
    }
    
    pub fn default_config() -> Self {
        Self::new(RetryConfig::default())
    }
    
    /// Execute dengan retry
    pub fn execute<F, T, E>(&self, mut f: F) -> Result<T, E>
    where
        F: FnMut() -> Result<T, E>,
        E: std::fmt::Debug,
    {
        let mut attempt = 0;
        let mut delay = self.config.initial_delay_ms;
        
        loop {
            match f() {
                Ok(result) => return Ok(result),
                Err(e) => {
                    attempt += 1;
                    
                    if attempt >= self.config.max_attempts {
                        tracing::error!("Max retry attempts ({}) reached", self.config.max_attempts);
                        return Err(e);
                    }
                    
                    tracing::warn!("Attempt {} failed: {:?}, retrying in {}ms", 
                        attempt, e, delay);
                    
                    thread::sleep(Duration::from_millis(delay));
                    delay = ((delay as f64) * self.config.multiplier) as u64;
                    delay = delay.min(self.config.max_delay_ms);
>>>>>>> bce0345919f371d153ccb843f2ddbfb5e8695c5f
                }
            }
        }
    }
<<<<<<< HEAD

    warn!(
        operation = operation_name,
        attempts = config.max_attempts,
        error = %last_error,
        "Semua percobaan retry gagal"
    );

    Err(last_error)
}

/// Hitung delay untuk attempt ke-N dengan exponential backoff
pub fn calculate_backoff_ms(attempt: u32, initial_ms: u64, factor: f64, max_ms: u64) -> u64 {
    if attempt == 0 {
        return 0;
    }
    let multiplier = factor.powi((attempt - 1) as i32);
    ((initial_ms as f64 * multiplier) as u64).min(max_ms)
=======
>>>>>>> bce0345919f371d153ccb843f2ddbfb5e8695c5f
}
