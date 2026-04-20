/// Primitif manajemen window Win32.
// use crate::utils::error::{AppError, AppResult};  //  Jika Anda ingin menggunakan error handling khusus

/// Encode string Rust ke wide string (null-terminated) untuk Win32
pub fn to_wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

/// Ambil dimensi layar primer
#[cfg(target_os = "windows")]
pub fn get_screen_dimensions() -> (i32, i32) {
    use windows::Win32::UI::WindowsAndMessaging::{GetSystemMetrics, SM_CXSCREEN, SM_CYSCREEN};
    let w = unsafe { GetSystemMetrics(SM_CXSCREEN) };
    let h = unsafe { GetSystemMetrics(SM_CYSCREEN) };
    (w, h)
}

#[cfg(not(target_os = "windows"))]
pub fn get_screen_dimensions() -> (i32, i32) {
    (1920, 1080)
}

/// Pesan custom untuk komunikasi antar thread ke window
pub mod msg {
    pub const WM_APP_UNLOCK_SUCCESS: u32 = 0x8001;
    pub const WM_APP_CLOSE_OVERLAY: u32 = 0x8003;
    pub const WM_APP_SHAKE: u32 = 0x8004;
}
