/// Loader untuk file .env - memuat kredensial dan pengaturan sensitif.
/// Tidak menyimpan plaintext, hanya hash.
use crate::utils::error::{AppError, AppResult};
use dotenvy::from_path;
use std::path::Path;
use tracing::{info, warn};

/// Variabel lingkungan yang diperlukan
pub struct EnvVars {
    /// Hash argon2 dari kata sandi admin
    pub admin_password_hash: String,
    /// Level log dari environment
    pub log_level: Option<String>,
    /// Mode aplikasi dari environment
    pub app_mode: Option<String>,
}

/// Muat variabel dari file .env
///
/// Jika file tidak ada, dibuat dengan nilai default (hash "Admin12345!")
pub fn load_env(env_path: &Path) -> AppResult<EnvVars> {
    // Buat file .env default jika belum ada
    if !env_path.exists() {
        info!(
            path = %env_path.display(),
            "File .env tidak ditemukan, membuat default"
        );
        create_default_env(env_path)?;
    }

    // Muat .env ke environment
    from_path(env_path).map_err(|e| AppError::Config(format!("Gagal memuat .env: {e}")))?;

    read_env_vars()
}

/// Baca variabel yang sudah dimuat ke environment
pub fn read_env_vars() -> AppResult<EnvVars> {
    // Priority 1: Environment variable sistem (untuk override)
    // Priority 2: Fallback ke kosong jika tidak ada
    let mut admin_password_hash = std::env::var("ADMIN_PASSWORD_HASH")
        .unwrap_or_default()
        .trim()
        .to_string();

    // FIX: Hapus tanda kutip ganda di awal dan akhir jika ada
    // Ini menangani kasus di mana hash disimpan dengan tanda kutip di .env
    if admin_password_hash.starts_with('"') && admin_password_hash.ends_with('"') {
        admin_password_hash = admin_password_hash[1..admin_password_hash.len()-1].to_string();
    }

    Ok(EnvVars {
        admin_password_hash,
        log_level: std::env::var("LOG_LEVEL").ok(),
        app_mode: std::env::var("APP_MODE").ok(),
    })
}

/// Sama seperti read_env_vars tapi paksa baca dari sistem env var
/// Digunakan setelah dotenvy::from_path dipanggil
pub fn read_env_vars_for_overwrite() -> AppResult<EnvVars> {
    read_env_vars()
}

/// Tulis hash password baru ke file .env
pub fn write_password_hash(env_path: &Path, hash: &str) -> AppResult<()> {
    // FIX: Selalu bungkus hash dalam tanda kutip ganda untuk mencegah
    // interpretasi $ sebagai variable expansion di shell
    let quoted_hash = format!("\"{hash}\"");

    let content = if env_path.exists() {
        let existing =
            std::fs::read_to_string(env_path).map_err(|e| AppError::io("Baca .env", e))?;
        update_env_value(&existing, "ADMIN_PASSWORD_HASH", &quoted_hash)
    } else {
        format!(
            "# App Blocker - Konfigurasi Kredensial\n\
             # JANGAN bagikan file ini!\n\
             ADMIN_PASSWORD_HASH={}\n\
             APP_MODE=production\n\
             LOG_LEVEL=info\n",
            quoted_hash
        )
    };

    std::fs::write(env_path, content).map_err(|e| AppError::io("Tulis .env", e))?;

    info!("Hash password berhasil diperbarui di .env");
    Ok(())
}

/// Update nilai dalam string .env
fn update_env_value(content: &str, key: &str, value: &str) -> String {
    let mut updated = false;
    let lines: Vec<String> = content
        .lines()
        .map(|line| {
            if line.starts_with(&format!("{key}=")) || line.starts_with(&format!("{key} =")) {
                updated = true;
                format!("{key}={value}")
            } else {
                line.to_string()
            }
        })
        .collect();

    if updated {
        lines.join("\n")
    } else {
        format!("{content}\n{key}={value}")
    }
}

/// Buat file .env default dengan hash password "Admin12345!"
fn create_default_env(env_path: &Path) -> AppResult<()> {
    // Hash akan di-generate oleh security::auth pada startup
    let content = "# App Blocker - Konfigurasi Kredensial\n\
                   # JANGAN bagikan file ini!\n\
                   # Hash diisi otomatis saat startup pertama\n\
                   ADMIN_PASSWORD_HASH=\n\
                   APP_MODE=production\n\
                   LOG_LEVEL=info\n";

    if let Some(parent) = env_path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| AppError::io("Buat direktori .env", e))?;
    }

    std::fs::write(env_path, content).map_err(|e| AppError::io("Buat .env default", e))?;

    warn!(
        "File .env dibuat dengan nilai default. \
         Kata sandi default: Admin12345! - SEGERA GANTI!"
    );

    Ok(())
}

/// Validasi variabel environment
pub fn validate_env(vars: &EnvVars) -> AppResult<()> {
    // Hash boleh kosong pada startup pertama (akan di-generate)
    if !vars.admin_password_hash.is_empty() {
        // Validasi format hash argon2
        if !vars.admin_password_hash.starts_with("$argon2") {
            return Err(AppError::Config(
                "ADMIN_PASSWORD_HASH bukan format argon2 yang valid".to_string(),
            ));
        }
    }

    Ok(())
}
