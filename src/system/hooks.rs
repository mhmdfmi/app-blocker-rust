//! Keyboard hook manager (Windows)
//! - Dedicated thread for WH_KEYBOARD_LL
//! - Blocks Alt+F4 and Escape while active
//! - Panic-safe callback (catch_unwind)
//! - Graceful uninstall: stop channel + PostThreadMessage(WM_QUIT) + join
//! - No-op on non-Windows

use crate::utils::error::{AppError, AppResult};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Mutex};
use std::thread::JoinHandle;
use tracing::{debug, error, info, warn};

/// Derived flag whether hook is active
static HOOK_ACTIVE: AtomicBool = AtomicBool::new(false);

/// Install keyboard hook; returns a guard that will uninstall on Drop.
pub fn install_keyboard_hook() -> AppResult<HookGuard> {
    if HOOK_ACTIVE.load(Ordering::SeqCst) {
        warn!("Keyboard hook already active");
        return Ok(HookGuard { installed: false });
    }

    #[cfg(target_os = "windows")]
    {
        windows_impl::install_hook_windows()?;
    }

    HOOK_ACTIVE.store(true, Ordering::SeqCst);
    info!("Keyboard hook installed (Alt+F4, Escape blocked)");
    Ok(HookGuard { installed: true })
}

/// Guard that uninstalls hook on drop
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

/// Uninstall keyboard hook (idempotent)
pub fn uninstall_keyboard_hook() {
    if !HOOK_ACTIVE.load(Ordering::SeqCst) {
        debug!("Keyboard hook not active; nothing to uninstall");
        return;
    }

    #[cfg(target_os = "windows")]
    {
        if let Err(e) = windows_impl::uninstall_hook_windows() {
            error!("Failed to uninstall keyboard hook cleanly: {}", e);
        }
    }

    HOOK_ACTIVE.store(false, Ordering::SeqCst);
    info!("Keyboard hook uninstalled");
}

/// Is hook active
pub fn is_hook_active() -> bool {
    HOOK_ACTIVE.load(Ordering::SeqCst)
}

#[cfg(not(target_os = "windows"))]
mod windows_impl {
    use super::*;
    pub fn install_hook_windows() -> AppResult<()> {
        // No-op on non-Windows
        Ok(())
    }
    pub fn uninstall_hook_windows() -> Result<(), AppError> {
        Ok(())
    }
}

#[cfg(target_os = "windows")]
mod windows_impl {
    use super::*;
    use once_cell::sync::OnceCell;
    use std::panic;
    use std::time::{Duration, Instant};
    use windows::Win32::Foundation::{LPARAM, LRESULT, WPARAM};
    use windows::Win32::System::LibraryLoader::GetModuleHandleW;
    use windows::Win32::System::Threading::GetCurrentThreadId;
    use windows::Win32::UI::Input::KeyboardAndMouse::{VK_ESCAPE, VK_F4};
    use windows::Win32::UI::WindowsAndMessaging::{
        CallNextHookEx, DispatchMessageW, GetMessageW, PostThreadMessageW, SetWindowsHookExW,
        TranslateMessage, UnhookWindowsHookEx, HHOOK, KBDLLHOOKSTRUCT, MSG, WH_KEYBOARD_LL,
        WM_KEYDOWN, WM_QUIT, WM_SYSKEYDOWN,
    };

    static HOOK_THREAD_CELL: OnceCell<Mutex<Option<HookThread>>> = OnceCell::new();
    static HOOK_HANDLE_CELL: OnceCell<Mutex<Option<HHOOK>>> = OnceCell::new();
    // HookThread stores stop channel and join handle and thread id
    struct HookThread {
        stop_tx: mpsc::Sender<()>,
        join_handle: JoinHandle<()>,
        thread_id: u32,
        exit_rx: mpsc::Receiver<()>, // receiver for thread-exit acknowledgement
    }

    // Global storage protected by Mutex via OnceCell

    fn hook_thread_cell() -> &'static Mutex<Option<HookThread>> {
        HOOK_THREAD_CELL.get_or_init(|| Mutex::new(None))
    }
    fn hook_handle_cell() -> &'static Mutex<Option<HHOOK>> {
        HOOK_HANDLE_CELL.get_or_init(|| Mutex::new(None))
    }

    /// Install hook by spawning a dedicated thread
    pub fn install_hook_windows() -> AppResult<()> {
        // Prevent double install
        {
            let guard = hook_thread_cell()
                .lock()
                .map_err(|e| AppError::Win32(format!("HOOK_THREAD lock failed: {}", e)))?;
            if guard.is_some() {
                return Ok(());
            }
        }

        // Channel to request thread stop
        let (stop_tx, stop_rx) = mpsc::channel::<()>();
        // Channel for thread to notify when it is exiting
        let (exit_tx, exit_rx) = mpsc::channel::<()>();

        // Spawn thread, move exit_tx into the thread so it can send ack on exit
        let join_handle = std::thread::Builder::new()
            .name("app_blocker_keyboard_hook".into())
            .spawn(move || {
                let res = panic::catch_unwind(|| {
                    if let Err(e) = hook_thread_main(stop_rx, exit_tx) {
                        error!("Keyboard hook thread error: {}", e);
                    }
                });
                if res.is_err() {
                    error!("Keyboard hook thread panicked");
                }
            })
            .map_err(|e| AppError::Win32(format!("Failed to spawn hook thread: {}", e)))?;

        // Store HookThread with thread_id = 0 for now; thread will update its id
        {
            let mut guard = hook_thread_cell()
                .lock()
                .map_err(|e| AppError::Win32(format!("HOOK_THREAD lock failed: {}", e)))?;
            *guard = Some(HookThread {
                stop_tx,
                join_handle,
                thread_id: 0,
                exit_rx, // keep receiver so uninstall can wait for ack
            });
        }

        // Wait briefly for thread to register its thread id (optional)
        let start = Instant::now();
        let timeout = Duration::from_millis(500);
        while start.elapsed() < timeout {
            {
                let guard = hook_thread_cell()
                    .lock()
                    .map_err(|e| AppError::Win32(format!("HOOK_THREAD lock failed: {}", e)))?;
                if let Some(ref ht) = *guard {
                    if ht.thread_id != 0 {
                        return Ok(());
                    }
                }
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }

        Ok(())
    }

    /// Uninstall hook: send stop signal, post WM_QUIT, join, then unhook
    pub fn uninstall_hook_windows() -> Result<(), AppError> {
        // Take HookThread
        let maybe_thread = {
            let mut guard = hook_thread_cell()
                .lock()
                .map_err(|e| AppError::Win32(format!("HOOK_THREAD lock failed: {}", e)))?;
            guard.take()
        };

        if let Some(ht) = maybe_thread {
            // Send stop signal
            let _ = ht.stop_tx.send(());

            // Post WM_QUIT to thread if we have thread id
            if ht.thread_id != 0 {
                unsafe {
                    let _ = PostThreadMessageW(ht.thread_id, WM_QUIT, WPARAM(0), LPARAM(0));
                }
            }

            // First, wait for explicit exit acknowledgement from thread (via exit_rx)
            let join_timeout = std::time::Duration::from_secs(2);
            match ht.exit_rx.recv_timeout(join_timeout) {
                Ok(_) => {
                    debug!("Received exit acknowledgement from hook thread");
                    // Now join the thread (should be immediate)
                    match ht.join_handle.join() {
                        Ok(_) => debug!("Hook thread joined after ack"),
                        Err(_) => warn!("Hook thread panicked during join after ack"),
                    }
                }
                Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                    warn!("Timeout waiting for hook thread exit acknowledgement; falling back to join helper");
                    // Fallback: spawn helper to join and wait with timeout
                    let (done_tx, done_rx) = std::sync::mpsc::channel::<()>();
                    let join_handle = ht.join_handle;
                    std::thread::spawn(move || {
                        let _ = join_handle.join();
                        let _ = done_tx.send(());
                    });

                    // Wait for helper to signal join completion
                    match done_rx.recv_timeout(join_timeout) {
                        Ok(_) => debug!("Hook thread joined (fallback)"),
                        Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                            warn!("Timeout waiting for hook thread to join (fallback)");
                        }
                        Err(e) => warn!("Error waiting for hook thread join notification: {}", e),
                    }
                }
                Err(e) => {
                    warn!("Error waiting for hook thread exit acknowledgement: {}", e);
                    // Attempt to join anyway (best-effort)
                    let _ = ht.join_handle.join();
                }
            }
        }

        // Ensure HHOOK is unhooked
        if let Ok(mut hguard) = hook_handle_cell().lock() {
            if let Some(hhook) = hguard.take() {
                match unsafe { UnhookWindowsHookEx(hhook) } {
                    Ok(_) => debug!("UnhookWindowsHookEx succeeded"),
                    Err(e) => warn!("UnhookWindowsHookEx failed: {}", e),
                }
            }
        }

        Ok(())
    }

    /// Thread main: install hook, store handle, run message loop until WM_QUIT or stop signal
    fn hook_thread_main(
        stop_rx: mpsc::Receiver<()>,
        exit_tx: mpsc::Sender<()>,
    ) -> Result<(), AppError> {
        // Get module handle
        let module = unsafe {
            GetModuleHandleW(None)
                .map_err(|e| AppError::Win32(format!("GetModuleHandleW failed: {}", e)))?
        };

        // Install low-level keyboard hook
        let hook = unsafe {
            SetWindowsHookExW(WH_KEYBOARD_LL, Some(keyboard_proc), module, 0)
                .map_err(|e| AppError::Win32(format!("SetWindowsHookExW failed: {}", e)))?
        };

        // Store hook handle
        {
            let mut guard = hook_handle_cell()
                .lock()
                .map_err(|e| AppError::Win32(format!("HOOK_HANDLE lock failed: {}", e)))?;
            *guard = Some(hook);
        }

        // Save thread id into HookThread entry
        let thread_id = unsafe { GetCurrentThreadId() };
        {
            let mut guard = hook_thread_cell()
                .lock()
                .map_err(|e| AppError::Win32(format!("HOOK_THREAD lock failed: {}", e)))?;
            if let Some(ref mut ht) = *guard {
                ht.thread_id = thread_id;
            }
        }

        info!("Keyboard hook installed on thread id {}", thread_id);

        // Message loop
        let mut msg = MSG::default();
        loop {
            // If stop signal received, break
            if stop_rx.try_recv().is_ok() {
                debug!("Stop signal received for keyboard hook thread");
                break;
            }

            // Block on GetMessageW; will return 0 on WM_QUIT
            let r = unsafe { GetMessageW(&mut msg, None, 0, 0) };
            if r.0 == 0 {
                debug!("GetMessageW returned 0 (WM_QUIT), exiting hook thread loop");
                break;
            } else if r.0 == -1 {
                warn!("GetMessageW returned -1 (error)");
                break;
            } else {
                unsafe {
                    TranslateMessage(&msg);
                    DispatchMessageW(&msg);
                }
            }
        }

        // Cleanup: unhook if still present
        {
            let mut guard = hook_handle_cell()
                .lock()
                .map_err(|e| AppError::Win32(format!("HOOK_HANDLE lock failed: {}", e)))?;
            if let Some(hhook) = guard.take() {
                match unsafe { UnhookWindowsHookEx(hhook) } {
                    Ok(_) => debug!("UnhookWindowsHookEx succeeded in thread cleanup"),
                    Err(e) => warn!("UnhookWindowsHookEx failed in thread cleanup: {}", e),
                }
            }
        }

        // Send exit acknowledgement to uninstaller (best-effort)
        let _ = exit_tx.send(());

        info!("Keyboard hook thread exiting cleanly");
        Ok(())
    }
    // Low-level keyboard proc. Must be extern "system" and panic-safe.
    unsafe extern "system" fn keyboard_proc(
        n_code: i32,
        w_param: WPARAM,
        l_param: LPARAM,
    ) -> LRESULT {
        // Wrap in catch_unwind to avoid unwinding into OS
        let res = panic::catch_unwind(|| {
            if n_code >= 0 {
                let msg = w_param.0 as u32;
                if (msg == WM_KEYDOWN || msg == WM_SYSKEYDOWN) && l_param.0 != 0 {
                    let kbd = &*(l_param.0 as *const KBDLLHOOKSTRUCT);
                    let vk = kbd.vkCode;

                    // flags is KBDLLHOOKSTRUCT::flags; access numeric value via .0
                    let flags_val = kbd.flags.0;
                    const LLKHF_ALTDOWN: u32 = 0x20;

                    // Block Alt+F4
                    if vk == VK_F4.0 as u32 {
                        let alt_pressed = (flags_val & LLKHF_ALTDOWN) != 0;
                        if alt_pressed {
                            debug!("Blocked Alt+F4 via keyboard hook");
                            return LRESULT(1);
                        }
                    }

                    // Block Escape
                    if vk == VK_ESCAPE.0 as u32 {
                        debug!("Blocked Escape via keyboard hook");
                        return LRESULT(1);
                    }
                }
            }
            // Not handled: call next hook
            CallNextHookEx(None, n_code, w_param, l_param)
        });

        match res {
            Ok(ret) => ret,
            Err(_) => {
                error!("Panic inside keyboard_proc; calling next hook to avoid blocking");
                CallNextHookEx(None, n_code, w_param, l_param)
            }
        }
    }
}
