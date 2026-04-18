/// Layanan informasi pengguna dan privilege Windows.
use crate::utils::error::{AppError, AppResult};
use tracing::warn;

/// Informasi sesi pengguna saat ini
#[derive(Debug, Clone)]
pub struct UserSession {
    pub username: String,
    pub computer_name: String,
    pub is_admin: bool,
    pub session_id: u32,
}

/// Dapatkan informasi sesi pengguna saat ini
pub fn get_current_session() -> UserSession {
    UserSession {
        username: get_username(),
        computer_name: get_computer_name(),
        is_admin: check_admin_privilege(),
        session_id: get_session_id(),
    }
}

/// Ambil nama pengguna aktif
pub fn get_username() -> String {
    std::env::var("USERNAME")
        .or_else(|_| std::env::var("USER"))
        .unwrap_or_else(|_| "UNKNOWN".to_string())
}

/// Ambil nama komputer
pub fn get_computer_name() -> String {
    std::env::var("COMPUTERNAME")
        .or_else(|_| {
            hostname::get()
                .map(|h| h.to_string_lossy().to_string())
                .map_err(|e| std::env::VarError::NotPresent)
        })
        .unwrap_or_else(|_| "UNKNOWN-PC".to_string())
}

/// Periksa apakah proses berjalan dengan hak admin
#[cfg(target_os = "windows")]
pub fn check_admin_privilege() -> bool {
    use windows::Win32::Foundation::HANDLE;
    use windows::Win32::Security::{GetTokenInformation, TokenElevation, TOKEN_ELEVATION, TOKEN_QUERY};
    use windows::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};

    unsafe {
        let mut token = HANDLE::default();
        if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token).is_err() {
            return false;
        }

        let mut elevation = TOKEN_ELEVATION::default();
        let mut size = std::mem::size_of::<TOKEN_ELEVATION>() as u32;

        let result = GetTokenInformation(
            token,
            TokenElevation,
            Some(&mut elevation as *mut _ as *mut _),
            size,
            &mut size,
        );

        let _ = windows::Win32::Foundation::CloseHandle(token);
        result.is_ok() && elevation.TokenIsElevated != 0
    }
}

#[cfg(not(target_os = "windows"))]
pub fn check_admin_privilege() -> bool {
    false
}

/// Ambil ID sesi Windows
#[cfg(target_os = "windows")]
pub fn get_session_id() -> u32 {
    use windows::Win32::System::Threading::{GetCurrentProcessId, ProcessIdToSessionId};
    let pid = unsafe { GetCurrentProcessId() };
    let mut session_id = 0u32;
    unsafe {
        let _ = ProcessIdToSessionId(pid, &mut session_id);
    }
    session_id
}

#[cfg(not(target_os = "windows"))]
pub fn get_session_id() -> u32 {
    0
}
