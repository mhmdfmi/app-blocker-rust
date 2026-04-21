// src/security/auth.rs
/// Layanan autentikasi dengan Argon2id.
/// Password TIDAK PERNAH disimpan dalam plaintext.
use crate::security::memory::SecureString;
use crate::utils::error::{AppError, AppResult};
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use std::time::{Duration, Instant};
use tracing::{info, warn};

/// Kata sandi default (hanya digunakan untuk generate hash pertama kali)
pub const DEFAULT_PASSWORD: &str = "Admin12345!";

/// Status autentikasi
#[derive(Debug, Clone, PartialEq)]
pub enum AuthStatus {
    /// Autentikasi berhasil
    Success,
    /// Kata sandi salah
    Failed,
    /// Terkunci karena terlalu banyak percobaan
    LockedOut { remaining_seconds: u64 },
}

/// State percobaan autentikasi
#[derive(Debug, Default)]
pub struct AuthAttemptState {
    /// Jumlah percobaan gagal berturut-turut
    pub failed_attempts: u32,
    /// Waktu percobaan pertama gagal
    pub first_failure: Option<Instant>,
    /// Waktu lockout dimulai
    pub lockout_start: Option<Instant>,
}

impl AuthAttemptState {
    /// Reset state percobaan setelah sukses
    pub fn reset(&mut self) {
        self.failed_attempts = 0;
        self.first_failure = None;
        self.lockout_start = None;
    }

    /// Catat percobaan gagal
    pub fn record_failure(&mut self) {
        if self.first_failure.is_none() {
            self.first_failure = Some(Instant::now());
        }
        self.failed_attempts += 1;
    }

    /// Mulai lockout
    pub fn start_lockout(&mut self) {
        self.lockout_start = Some(Instant::now());
    }

    /// Periksa sisa waktu lockout (detik)
    pub fn lockout_remaining_seconds(&self, lockout_duration: Duration) -> Option<u64> {
        self.lockout_start.and_then(|start| {
            let elapsed = start.elapsed();
            if elapsed < lockout_duration {
                Some((lockout_duration - elapsed).as_secs())
            } else {
                None
            }
        })
    }

    /// Periksa apakah lockout sudah selesai
    pub fn is_lockout_expired(&self, lockout_duration: Duration) -> bool {
        self.lockout_start
            .map(|start| start.elapsed() >= lockout_duration)
            .unwrap_or(true)
    }
}

/// Trait kontrak untuk layanan autentikasi
pub trait AuthService: Send + Sync {
    /// Verifikasi kata sandi terhadap hash tersimpan
    fn verify_password(&self, password: &SecureString) -> AppResult<bool>;
    /// Hash kata sandi baru dengan argon2
    fn hash_password(&self, password: &SecureString) -> AppResult<String>;
    /// Dapatkan hash saat ini
    fn current_hash(&self) -> &str;
}

/// Implementasi AuthService menggunakan Argon2id
pub struct Argon2AuthService {
    /// Hash argon2 dari kata sandi admin
    password_hash: String,
}

impl Argon2AuthService {
    /// Buat service auth dengan hash yang ada
    pub fn new(password_hash: String) -> AppResult<Self> {
        // Trim whitespace/newline dari input
        let cleaned = password_hash.trim();

        info!("Membuat Argon2AuthService dengan hash: {}", cleaned);
        // Validasi hash kosong
        if cleaned.is_empty() {
            return Err(AppError::Auth("Hash password kosong".to_string()));
        }

        // Validasi format hash jika tidak kosong
        if !cleaned.is_empty() && !cleaned.starts_with("$argon2") {
            return Err(AppError::Auth(
                "Format hash tidak valid, harus argon2".to_string(),
            ));
        }
        Ok(Self {
            password_hash: cleaned.to_string(),
        })
    }

    /// Buat service dengan generate hash default "Admin12345!"
    pub fn with_default_password() -> AppResult<(Self, String)> {
        // Buat sementara SecureString dari DEFAULT_PASSWORD untuk hashing
        let tmp = SecureString::try_from_str(DEFAULT_PASSWORD)?;
        let mut service = Self {
            password_hash: String::new(),
        };
        let hash = service.hash_password(&tmp)?;
        // explicit zeroing not necessary here because tmp will be dropped and zeroized
        service.password_hash = hash.clone();
        info!(
            "Hash password default berhasil di-generate {}",
            service.password_hash
        );
        Ok((service, hash))
    }

    /// Update hash password
    pub fn update_hash(&mut self, new_hash: String) -> AppResult<()> {
        let cleaned = new_hash.trim();
        if !cleaned.starts_with("$argon2") {
            return Err(AppError::Auth("Format hash baru tidak valid".to_string()));
        }
        self.password_hash = cleaned.to_string();
        info!("Hash password berhasil diperbarui {}", self.password_hash);
        Ok(())
    }
}

impl AuthService for Argon2AuthService {
    /// Verifikasi kata sandi menggunakan Argon2id
    fn verify_password(&self, password: &SecureString) -> AppResult<bool> {
        if self.password_hash.is_empty() {
            return Err(AppError::Auth(
                "Hash password belum dikonfigurasi".to_string(),
            ));
        }

        let parsed_hash = PasswordHash::new(&self.password_hash)
            .map_err(|e| AppError::Auth(format!("Parse hash gagal: {e}")))?;

        let argon2 = Argon2::default();

        // Akses bytes sementara tanpa membuat salinan plaintext
        let res = argon2.verify_password(password.as_bytes(), &parsed_hash);

        Ok(res.is_ok())
    }

    /// Hash kata sandi baru dengan argon2id + salt random
    fn hash_password(&self, password: &SecureString) -> AppResult<String> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();

        let hash = argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| AppError::Auth(format!("Hashing gagal: {e}")))?;

        Ok(hash.to_string())
    }

    fn current_hash(&self) -> &str {
        &self.password_hash
    }
}

/// Manager autentikasi lengkap dengan rate limiting dan lockout
pub struct AuthManager {
    service: Box<dyn AuthService>,
    state: AuthAttemptState,
    max_attempts: u32,
    lockout_duration: Duration,
}

impl AuthManager {
    /// Buat AuthManager baru
    pub fn new(
        service: Box<dyn AuthService>,
        max_attempts: u32,
        lockout_duration_seconds: u64,
    ) -> Self {
        Self {
            service,
            state: AuthAttemptState::default(),
            max_attempts,
            lockout_duration: Duration::from_secs(lockout_duration_seconds),
        }
    }

    /// Validasi kebijakan password (panjang minimal, dll.)
    fn validate_policy(&self, password: &SecureString) -> AppResult<()> {
        let len = password.len();
        if len < 8 {
            return Err(AppError::Validation(
                "Password minimal 8 karakter".to_string(),
            ));
        }
        if len > 4096 {
            return Err(AppError::Validation("Password terlalu panjang".to_string()));
        }
        Ok(())
    }

    /// Coba autentikasi dengan rate limiting
    pub fn authenticate(&mut self, password: &SecureString) -> AppResult<AuthStatus> {
        // Validasi kebijakan sebelum verifikasi
        self.validate_policy(password)?;

        // Periksa apakah dalam lockout
        if self.state.lockout_start.is_some() {
            if let Some(remaining) = self.state.lockout_remaining_seconds(self.lockout_duration) {
                warn!(
                    remaining_seconds = remaining,
                    failed_attempts = self.state.failed_attempts,
                    "Autentikasi ditolak - dalam lockout"
                );
                return Ok(AuthStatus::LockedOut {
                    remaining_seconds: remaining,
                });
            } else {
                // Lockout sudah berakhir, reset
                self.state.reset();
            }
        }

        // Verifikasi password melalui service (tidak menyimpan plaintext)
        match self.service.verify_password(password)? {
            true => {
                info!("Autentikasi berhasil");
                self.state.reset();
                Ok(AuthStatus::Success)
            }
            false => {
                self.state.record_failure();
                warn!(
                    failed_attempts = self.state.failed_attempts,
                    max_attempts = self.max_attempts,
                    "Autentikasi gagal"
                );

                // Periksa apakah perlu lockout
                if self.state.failed_attempts >= self.max_attempts {
                    self.state.start_lockout();
                    warn!(
                        lockout_seconds = self.lockout_duration.as_secs(),
                        "Akun dikunci karena terlalu banyak percobaan gagal"
                    );
                    return Ok(AuthStatus::LockedOut {
                        remaining_seconds: self.lockout_duration.as_secs(),
                    });
                }

                Ok(AuthStatus::Failed)
            }
        }
    }

    /// Dapatkan jumlah percobaan gagal saat ini
    pub fn failed_attempts(&self) -> u32 {
        self.state.failed_attempts
    }

    /// Hash kata sandi baru melalui service
    pub fn hash_new_password(&self, password: &SecureString) -> AppResult<String> {
        // Validasi kebijakan sebelum hashing
        if password.len() < 8 {
            return Err(AppError::Validation("Password minimal 8 karakter".into()));
        }
        self.service.hash_password(password)
    }

    /// Reset state percobaan (digunakan setelah unlock berhasil)
    pub fn reset_attempts(&mut self) {
        self.state.reset();
    }
}

// ── Tambahan untuk akses hash saat ini ──────────────────────────────────────
impl AuthManager {
    /// Dapatkan hash password saat ini dari service
    pub fn current_hash(&self) -> &str {
        self.service.current_hash()
    }
}
