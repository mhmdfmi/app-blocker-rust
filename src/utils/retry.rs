//! Retry Module
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
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_delay_ms: 100,
            max_delay_ms: 5000,
            multiplier: 2.0,
        }
    }
}

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
                }
            }
        }
    }
}
