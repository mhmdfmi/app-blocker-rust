/// Windows keyboard hook untuk memblokir key kombinasi tertentu saat overlay aktif.
/// Hanya aktif ketika overlay sedang ditampilkan.
use crate::utils::error::{AppError, AppResult};
use std::sync::atomic::{AtomicBool, Ordering};
use tracing::{debug, info, warn};

/// Flag global apakah hook sedang aktif
static HOOK_ACTIVE: AtomicBool = AtomicBool::new(false);

/// Aktifkan keyboard hook (blokir Alt+F4, Escape saat overlay aktif)
pub fn install_keyboard_hook() -> AppResult<HookGuard> {
    if HOOK_ACTIVE.load(Ordering::SeqCst) {
        warn!("Keyboard hook sudah aktif");
        return Ok(HookGuard { installed: false });
    }

    #[cfg(target_os = "windows")]
    {
        install_hook_windows()?;
    }

    HOOK_ACTIVE.store(true, Ordering::SeqCst);
    info!("Keyboard hook diinstal (Alt+F4, Escape diblokir)");
    Ok(HookGuard { installed: true })
}

/// Guard yang otomatis melepas hook saat drop
pub struct HookGuard {
    installed: bool,
}

impl Drop for HookGuard {
    fn drop(&mut self) {
        if self.installed {
            uninstall_keyboard_hook();
        }
    }
}

/// Lepas keyboard hook
pub fn uninstall_keyboard_hook() {
    if HOOK_ACTIVE.load(Ordering::SeqCst) {
        #[cfg(target_os = "windows")]
        {
            uninstall_hook_windows();
        }
        HOOK_ACTIVE.store(false, Ordering::SeqCst);
        info!("Keyboard hook dilepas");
    }
}

/// Apakah hook sedang aktif
pub fn is_hook_active() -> bool {
    HOOK_ACTIVE.load(Ordering::SeqCst)
}

#[cfg(target_os = "windows")]
mod windows_impl {
    use super::*;
    use std::sync::Mutex;
    use windows::Win32::Foundation::{HINSTANCE, LPARAM, LRESULT, WPARAM};
    use windows::Win32::System::LibraryLoader::GetModuleHandleW;
    use windows::Win32::UI::WindowsAndMessaging::{
        CallNextHookEx, SetWindowsHookExW, UnhookWindowsHookEx, HHOOK, KBDLLHOOKSTRUCT,
        WH_KEYBOARD_LL, WM_KEYDOWN, WM_SYSKEYDOWN,
    };

    static HOOK_HANDLE: Mutex<Option<HHOOK>> = Mutex::new(None);

    /// Callback low-level keyboard hook
    unsafe extern "system" fn keyboard_proc(
        n_code: i32,
        w_param: WPARAM,
        l_param: LPARAM,
    ) -> LRESULT {
        use windows::Win32::UI::Input::KeyboardAndMouse::{VK_ESCAPE, VK_F4};

        if n_code >= 0 {
            let msg = w_param.0 as u32;
            if msg == WM_KEYDOWN || msg == WM_SYSKEYDOWN {
                let kbd = &*(l_param.0 as *const KBDLLHOOKSTRUCT);
                let vk = kbd.vkCode;

                // Blokir Alt+F4
                if vk == VK_F4.0 as u32 {
                    let alt_pressed = (kbd.flags.0 & 0x20) != 0; // LLKHF_ALTDOWN
                    if alt_pressed {
                        debug!("Alt+F4 diblokir oleh hook");
                        return LRESULT(1);
                    }
                }

                // Blokir Escape
                if vk == VK_ESCAPE.0 as u32 {
                    debug!("Escape diblokir oleh hook");
                    return LRESULT(1);
                }
            }
        }

        CallNextHookEx(None, n_code, w_param, l_param)
    }

    pub fn install_hook_windows() -> AppResult<()> {
        let module = unsafe {
            GetModuleHandleW(None)
                .map_err(|e| AppError::Win32(format!("GetModuleHandle gagal: {e}")))?
        };

        let hook = unsafe {
            SetWindowsHookExW(WH_KEYBOARD_LL, Some(keyboard_proc), module, 0)
                .map_err(|e| AppError::Win32(format!("SetWindowsHookEx gagal: {e}")))?
        };

        let mut guard = HOOK_HANDLE
            .lock()
            .map_err(|e| AppError::Win32(format!("Lock hook handle: {e}")))?;
        *guard = Some(hook);
        Ok(())
    }

    pub fn uninstall_hook_windows() {
        if let Ok(mut guard) = HOOK_HANDLE.lock() {
            if let Some(hook) = guard.take() {
                unsafe {
                    let _ = UnhookWindowsHookEx(hook);
                }
            }
        }
    }
}

#[cfg(target_os = "windows")]
use windows_impl::{install_hook_windows, uninstall_hook_windows};

#[cfg(not(target_os = "windows"))]
fn install_hook_windows() -> AppResult<()> {
    Ok(()) // No-op di non-Windows
}

#[cfg(not(target_os = "windows"))]
fn uninstall_hook_windows() {}
