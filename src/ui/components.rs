//! UI Components Module

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageBoxConfig {
    pub title: String,
    pub message: String,
    pub button_text: String,
}

impl Default for MessageBoxConfig {
    fn default() -> Self {
        Self {
            title: "Peringatan Keamanan".to_string(),
            message: "Aplikasi terlarang terdeteksi dan telah ditutup.".to_string(),
            button_text: "Buka Kunci".to_string(),
        }
    }
}

pub struct PasswordField;

impl PasswordField {
    pub fn new() -> Self {
        Self
    }
    
    pub fn render(&self) -> String {
        "[Password Input Field]".to_string()
    }
}

pub struct InfoPanel;

impl InfoPanel {
    pub fn render(process_name: &str, pid: u32) -> String {
        format!(
            "Process: {} | PID: {} | Time: {}",
            process_name,
            pid,
            chrono::Local::now().format("%Y-%m-%d %H:%M:%S")
        )
    }
}
