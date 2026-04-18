<<<<<<< HEAD
/// Modul utilitas umum - logger, error, time, retry
pub mod error;
pub mod logger;
pub mod retry;
pub mod time;

// Re-export tipe yang sering digunakan
pub use error::{AppError, AppResult};
pub use retry::{with_retry, RetryConfig};
pub use time::{format_datetime, now_utc};
=======
﻿//! Utils Module
//! 
//! Modul utility termasuk logging, error handling, time, dan retry logic.

pub mod logger;
pub mod error;
pub mod time;
pub mod retry;
>>>>>>> bce0345919f371d153ccb843f2ddbfb5e8695c5f
