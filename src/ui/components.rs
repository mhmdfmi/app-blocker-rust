/// Komponen UI kustom - ctrl IDs, tema warna, layout.
// use serde::Serialize; // jika ingin serialisasi tema dari config

/// ID kontrol Win32
pub mod ctrl_id {
    pub const ID_INPUT_PASSWORD: i32 = 101;
    pub const ID_BTN_SUBMIT: i32 = 102;
    pub const ID_LABEL_TITLE: i32 = 103;
    pub const ID_LABEL_INFO: i32 = 108;
    pub const ID_LABEL_COOLDOWN: i32 = 109;
}

/// Tema warna overlay (RGB tuples)
pub mod theme {
    pub const BG_MAIN: (u8, u8, u8) = (15, 23, 42);
    pub const RED_DANGER: (u8, u8, u8) = (239, 68, 68);
    pub const GREEN_SUCCESS: (u8, u8, u8) = (34, 197, 94);
    pub const TEXT_WHITE: (u8, u8, u8) = (248, 250, 252);
    pub const TEXT_MUTED: (u8, u8, u8) = (148, 163, 184);
    pub const BG_CARD: (u8, u8, u8) = (30, 41, 59);
    pub const BORDER_CARD: (u8, u8, u8) = (51, 65, 85);
    pub const YELLOW_WARN: (u8, u8, u8) = (245, 158, 11);

    /// Konversi RGB ke COLORREF Win32 (0x00BBGGRR)
    pub fn to_colorref(r: u8, g: u8, b: u8) -> u32 {
        (b as u32) << 16 | (g as u32) << 8 | (r as u32)
    }
}

/// Data tampilan overlay
#[derive(Debug, Clone, Default)]
pub struct DisplayData {
    pub process_name: String,
    pub pid: u32,
    pub username: String,
    pub computer_name: String,
    pub timestamp: String,
    pub attempts: u32,
    pub max_attempts: u32,
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

impl DisplayData {
    /// Ambil PID proses yang memicu overlay
    pub fn get_pid(&self) -> u32 {
        self.pid
    }
}
