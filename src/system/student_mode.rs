/// Student Mode Policy - menonaktifkan alat sistem saat overlay aktif.
/// Mencegah siswa bypass menggunakan Task Manager, regedit, atau cmd.
/// Semua perubahan dikembalikan saat unlock berhasil.
use crate::utils::error::{AppError, AppResult};
use tracing::{info, warn};
// use tracing::debug;  // Uncomment jika ingin log debug untuk operasi pembatasan (opsional)

/// Konfigurasi student mode
#[derive(Debug, Clone)]
pub struct StudentModeConfig {
    pub enabled: bool,
    pub disable_task_manager: bool,
    pub disable_registry_tools: bool,
    pub disable_cmd: bool,
    /// Hanya berlaku saat sistem dalam keadaan locked
    pub apply_only_when_locked: bool,
}

impl Default for StudentModeConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            disable_task_manager: true,
            disable_registry_tools: true,
            disable_cmd: true,
            apply_only_when_locked: true,
        }
    }
}

/// Menerapkan pembatasan student mode
pub fn apply_restrictions(config: &StudentModeConfig) -> AppResult<()> {
    if !config.enabled {
        return Ok(());
    }

    info!("Student mode: menerapkan pembatasan sistem");

    #[cfg(target_os = "windows")]
    {
        if config.disable_task_manager {
            set_task_manager_disabled(true)?;
        }
        if config.disable_registry_tools {
            set_registry_tools_disabled(true)?;
        }
        if config.disable_cmd {
            set_cmd_disabled(true)?;
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        debug!("Student mode: non-Windows, skip pembatasan registry");
    }

    Ok(())
}

/// Mengembalikan semua pembatasan (dipanggil setelah unlock)
pub fn restore_restrictions(config: &StudentModeConfig) -> AppResult<()> {
    if !config.enabled {
        return Ok(());
    }

    info!("Student mode: mengembalikan semua pembatasan sistem");

    #[cfg(target_os = "windows")]
    {
        if config.disable_task_manager {
            let _ = set_task_manager_disabled(false);
        }
        if config.disable_registry_tools {
            let _ = set_registry_tools_disabled(false);
        }
        if config.disable_cmd {
            let _ = set_cmd_disabled(false);
        }
    }

    Ok(())
}

/// Nonaktifkan / aktifkan Task Manager via registry
#[cfg(target_os = "windows")]
fn set_task_manager_disabled(disable: bool) -> AppResult<()> {
    // use windows::Win32::System::Registry::{
    //     RegCloseKey, RegCreateKeyExW, RegDeleteValueW, RegOpenKeyExW, RegSetValueExW,
    //     HKEY_CURRENT_USER, KEY_SET_VALUE, REG_DWORD,
    // };

    const KEY_PATH: &str = r"Software\Microsoft\Windows\CurrentVersion\Policies\System";
    const VALUE_NAME: &str = "DisableTaskMgr";

    set_dword_registry_value(KEY_PATH, VALUE_NAME, if disable { Some(1) } else { None })?;

    if disable {
        warn!("Task Manager dinonaktifkan (student mode)");
    } else {
        info!("Task Manager dikembalikan");
    }
    Ok(())
}

/// Nonaktifkan / aktifkan Registry Tools via registry
#[cfg(target_os = "windows")]
fn set_registry_tools_disabled(disable: bool) -> AppResult<()> {
    const KEY_PATH: &str = r"Software\Microsoft\Windows\CurrentVersion\Policies\System";
    const VALUE_NAME: &str = "DisableRegistryTools";

    set_dword_registry_value(KEY_PATH, VALUE_NAME, if disable { Some(1) } else { None })?;

    if disable {
        warn!("Registry tools dinonaktifkan (student mode)");
    } else {
        info!("Registry tools dikembalikan");
    }
    Ok(())
}

/// Nonaktifkan / aktifkan CMD via registry
#[cfg(target_os = "windows")]
fn set_cmd_disabled(disable: bool) -> AppResult<()> {
    const KEY_PATH: &str = r"Software\Policies\Microsoft\Windows\System";
    const VALUE_NAME: &str = "DisableCMD";

    set_dword_registry_value(
        KEY_PATH,
        VALUE_NAME,
        if disable { Some(2) } else { None }, // 2 = nonaktifkan cmd tapi bukan batch
    )?;

    if disable {
        warn!("CMD dinonaktifkan (student mode)");
    } else {
        info!("CMD dikembalikan");
    }
    Ok(())
}

/// Helper: set atau hapus DWORD registry value di HKCU
#[cfg(target_os = "windows")]
fn set_dword_registry_value(key_path: &str, value_name: &str, value: Option<u32>) -> AppResult<()> {
    use crate::ui::window::to_wide;
    use windows::Win32::System::Registry::{
        RegCloseKey, RegCreateKeyExW, RegDeleteValueW, RegSetValueExW, HKEY_CURRENT_USER,
        KEY_SET_VALUE, REG_DWORD, REG_OPTION_NON_VOLATILE,
    }; // add RegOpenKeyExW if needed for update instead of create

    let key_wide = to_wide(key_path);
    let val_wide = to_wide(value_name);
    let mut hkey = windows::Win32::System::Registry::HKEY::default();

    let result = unsafe {
        RegCreateKeyExW(
            HKEY_CURRENT_USER,
            windows::core::PCWSTR(key_wide.as_ptr()),
            0,
            None,
            REG_OPTION_NON_VOLATILE,
            KEY_SET_VALUE,
            None,
            &mut hkey,
            None,
        )
    };

    if result.is_err() {
        return Err(AppError::Win32(format!(
            "RegCreateKeyEx gagal untuk '{key_path}'"
        )));
    }

    // RAII guard untuk menutup handle registry
    struct RegKeyGuard(windows::Win32::System::Registry::HKEY);
    impl Drop for RegKeyGuard {
        fn drop(&mut self) {
            unsafe {
                let _ = RegCloseKey(self.0);
            }
        }
    }
    let _guard = RegKeyGuard(hkey);

    match value {
        Some(v) => {
            let bytes = v.to_le_bytes();
            unsafe {
                RegSetValueExW(
                    hkey,
                    windows::core::PCWSTR(val_wide.as_ptr()),
                    0,
                    REG_DWORD,
                    Some(&bytes),
                )
            }
            .map_err(|e| AppError::Win32(format!("RegSetValueEx gagal: {e}")))?;
        }
        None => {
            // Hapus value untuk restore
            unsafe {
                let _ = RegDeleteValueW(hkey, windows::core::PCWSTR(val_wide.as_ptr()));
            }
        }
    }

    Ok(())
}

#[cfg(not(target_os = "windows"))]
fn set_task_manager_disabled(_disable: bool) -> AppResult<()> {
    Ok(())
}
#[cfg(not(target_os = "windows"))]
fn set_registry_tools_disabled(_disable: bool) -> AppResult<()> {
    Ok(())
}
#[cfg(not(target_os = "windows"))]
fn set_cmd_disabled(_disable: bool) -> AppResult<()> {
    Ok(())
}
#[cfg(not(target_os = "windows"))]
fn set_dword_registry_value(_: &str, _: &str, _: Option<u32>) -> AppResult<()> {
    Ok(())
}
