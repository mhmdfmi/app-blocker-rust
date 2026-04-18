<<<<<<< HEAD
/// Komponen UI kustom - ctrl IDs, tema warna, layout.
use serde::Serialize;

/// ID kontrol Win32
pub mod ctrl_id {
    pub const ID_INPUT_PASSWORD: i32 = 101;
    pub const ID_BTN_SUBMIT: i32     = 102;
    pub const ID_LABEL_TITLE: i32    = 103;
    pub const ID_LABEL_INFO: i32     = 108;
    pub const ID_LABEL_COOLDOWN: i32 = 109;
}

/// Tema warna overlay (RGB tuples)
pub mod theme {
    pub const BG_MAIN:      (u8,u8,u8) = (15,  23,  42);
    pub const RED_DANGER:   (u8,u8,u8) = (239, 68,  68);
    pub const GREEN_SUCCESS:(u8,u8,u8) = (34,  197, 94);
    pub const TEXT_WHITE:   (u8,u8,u8) = (248, 250, 252);
    pub const TEXT_MUTED:   (u8,u8,u8) = (148, 163, 184);
    pub const BG_CARD:      (u8,u8,u8) = (30,  41,  59);
    pub const BORDER_CARD:  (u8,u8,u8) = (51,  65,  85);
    pub const YELLOW_WARN:  (u8,u8,u8) = (245, 158, 11);

    /// Konversi RGB ke COLORREF Win32 (0x00BBGGRR)
    pub fn to_colorref(r: u8, g: u8, b: u8) -> u32 {
        (b as u32) << 16 | (g as u32) << 8 | (r as u32)
    }
}

/// Data tampilan overlay
#[derive(Debug, Clone, Default)]
pub struct DisplayData {
    pub process_name:  String,
    pub pid:           u32,
    pub username:      String,
    pub computer_name: String,
    pub timestamp:     String,
    pub attempts:      u32,
    pub max_attempts:  u32,
}

/// Layout kartu tengah overlay
#[derive(Debug, Clone, Copy)]
pub struct CardLayout {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}

impl CardLayout {
    pub fn centered(screen_w: i32, screen_h: i32) -> Self {
        let w = (screen_w as f32 * 0.40) as i32;
        let h = 530;
        Self {
            x: (screen_w - w) / 2,
            y: (screen_h - h) / 2,
            w,
            h,
        }
    }
}
=======
﻿//! UI Components Module

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
>>>>>>> bce0345919f371d153ccb843f2ddbfb5e8695c5f
