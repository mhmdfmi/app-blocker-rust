/// Konstanta pesan UI dalam Bahasa Indonesia.

// === Judul & Header ===
pub const APP_NAME: &str = "App Blocker";
pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const APP_AUTHOR: &str = "Muhamad Fahmi";
pub const APP_ROLE: &str = "Asisten Kepala Lab Komputer";

// === Overlay UI ===
pub const OVERLAY_TITLE: &str = "⚠️  PERINGATAN KEAMANAN";
pub const OVERLAY_MESSAGE: &str = "Aplikasi terlarang telah terdeteksi dan dihentikan oleh sistem.";
pub const OVERLAY_SUBMESSAGE: &str = "Silakan masukkan kata sandi administrator untuk melanjutkan.";
pub const OVERLAY_FOOTER: &str =
    "This program was developed by Muhamad Fahmi, Assistant Head of the Computer Lab.";
pub const OVERLAY_PASSWORD_HINT: &str = "Kata Sandi Admin";
pub const OVERLAY_SUBMIT_BTN: &str = "Buka Kunci";
pub const OVERLAY_ATTEMPTS_FORMAT: &str = "Percobaan: {}/{} ";
pub const OVERLAY_TIMESTAMP_FORMAT: &str = "Waktu: {}";
pub const OVERLAY_PC_NAME_FORMAT: &str = "Komputer: {}";
pub const OVERLAY_BLOCKED_PROCESS: &str = "Proses: {}  (PID: {})";
pub const OVERLAY_USER_FORMAT: &str = "Pengguna: {}";

// === Status Unlock ===
pub const MSG_UNLOCK_SUCCESS: &str = "✓ Kata sandi benar. Akses diberikan.";
pub const MSG_UNLOCK_FAILED: &str = "✗ Kata sandi salah. Coba lagi.";
pub const MSG_UNLOCK_LOCKED_OUT: &str =
    "✗ Terlalu banyak percobaan gagal. Tunggu {} detik.";
pub const MSG_UNLOCK_COOLDOWN: &str = "Harap tunggu {} detik sebelum mencoba lagi.";

// === CLI Messages ===
pub const CLI_BLOCKER_ENABLED: &str = "✓ App Blocker diaktifkan.";
pub const CLI_BLOCKER_DISABLED: &str = "✓ App Blocker dinonaktifkan.";
pub const CLI_STATUS_FORMAT: &str = "Status: {}\nMode: {}\nUptime: {}";
pub const CLI_PASSWORD_CHANGED: &str = "✓ Kata sandi berhasil diperbarui.";
pub const CLI_CONFIG_RELOADED: &str = "✓ Konfigurasi berhasil dimuat ulang.";
pub const CLI_CONFIRM_PROMPT: &str = "Konfirmasi (y/N): ";
pub const CLI_PASSWORD_PROMPT: &str = "Kata sandi baru: ";
pub const CLI_PASSWORD_CONFIRM: &str = "Konfirmasi kata sandi: ";
pub const CLI_PASSWORD_MISMATCH: &str = "✗ Kata sandi tidak cocok.";
pub const CLI_SETUP_COMPLETE: &str = "✓ Setup selesai. Password default: Admin12345!";

// === Log Messages ===
pub const LOG_STARTUP: &str = "Sistem App Blocker dimulai";
pub const LOG_SHUTDOWN: &str = "Sistem App Blocker dihentikan";
pub const LOG_PROCESS_DETECTED: &str = "Proses terlarang terdeteksi";
pub const LOG_PROCESS_KILLED: &str = "Proses berhasil dihentikan";
pub const LOG_PROCESS_KILL_FAILED: &str = "Gagal menghentikan proses";
pub const LOG_PROTECTED_PROCESS: &str = "Proses dilindungi, tidak dihentikan";
pub const LOG_OVERLAY_SHOWN: &str = "Overlay ditampilkan";
pub const LOG_OVERLAY_CLOSED: &str = "Overlay ditutup";
pub const LOG_AUTH_SUCCESS: &str = "Autentikasi berhasil";
pub const LOG_AUTH_FAILED: &str = "Autentikasi gagal";
pub const LOG_STATE_TRANSITION: &str = "Transisi state";
pub const LOG_SAFE_MODE_ENTERED: &str = "Masuk Safe Mode";
pub const LOG_MONITORING_STARTED: &str = "Monitoring dimulai";
pub const LOG_WATCHDOG_RESTART: &str = "Watchdog: memulai ulang komponen";
pub const LOG_SINGLE_INSTANCE: &str = "Instance lain sudah berjalan";
pub const LOG_CONFIG_HOT_RELOAD: &str = "Konfigurasi dimuat ulang secara otomatis";

// === Error Messages ===
pub const ERR_PROTECTED_PROCESS: &str = "Tidak dapat menghentikan proses sistem yang dilindungi";
pub const ERR_CONFIG_INVALID: &str = "Konfigurasi tidak valid";
pub const ERR_ENV_MISSING: &str = "File .env tidak ditemukan";
pub const ERR_HASH_INVALID: &str = "Hash kata sandi tidak valid";
pub const ERR_CHANNEL_DISCONNECT: &str = "Koneksi channel terputus";
pub const ERR_THREAD_PANIC: &str = "Thread mengalami panic";
pub const ERR_DEADLOCK_DETECTED: &str = "Potensi deadlock terdeteksi";
pub const ERR_SINGLE_INSTANCE: &str = "Aplikasi sudah berjalan";
