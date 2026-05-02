# CHANGELOG

Format mengikuti [Keep a Changelog](https://keepachangelog.com/en/1.1.0/)
dan [Semantic Versioning](https://semver.org/lang/id/).

---

## [1.2.1] - 2026-04-30

### Ditambahkan (New Features)

- **DB Config Watcher** (`src/config/db_reload_watcher.rs`) — Menambahkan polling-based
  monitoring untuk perubahan konfigurasi di database. Setiap 5 detik memeriksa
  timestamp `updated_at` dari tabel configs, otomatis reload dan validasi config
  baru tanpa restart aplikasi. Fallback ke config lama jika validasi gagal.
- **Logging Path Improvements** — Logging path sekarang lebih fleksibel:
  - Kosongkan path di config (`path = ""`) untuk menggunakan AppData default
  - Override path lama `C:\AppBlocker\logs` ke AppData otomatis
- **Watchdog Heartbeat untuk UI Overlay** — UI overlay sekarang mengirim heartbeat
  ke watchdog setiap 1 detik untuk monitoring kesehatan komponen UI.
- **Lazy Component Registration** — ConfigWatcher dan UiOverlay didaftarkan
  ke watchdog secara lazy saat heartbeat pertama diterima, menghindari
  false positive "thread mati" saat komponen belum berjalan.

### Diperbaiki (Bug Fixes)

- **UI Text Fix** — Menghapus leading spaces dari teks "PERINGATAN KEAMANAN"
  agar rata kiri dengan elemen lainnya di overlay.
- **CLI Version Command** — Memindahkan posisi command `Version` ke grup statistik
  untuk konsistensi pengelompokan perintah.

### Diperbarui

- `.gitignore`: tambahkan `audit.json`
- DbConfigLoader: tambahkan derive `Clone` untuk kompatibilitas dengan Arc
- Main startup: cleanup dan improvement pada async block handling

---

## [1.2.0] - 2026-04-27

### Diperbaiki (Bug Fixes)

- **Watchdog false positive** — Monitor dan Engine tidak mengirim heartbeat ke watchdog
  dengan benar, menyebabkan error "Thread mati terdeteksi". Sekarang menggunakan
  dedicated `HEARTBEAT_TX` channel dengan static OnceLock.
- **Delay start-up** — Monitor dan Engine kini tunggu 2 detik sebelum loop
  heartbeat untuk memberi waktu HEARTBEAT_TX terinisialisasi.

### Ditambahkan (New Features)

- **Global Log Guard** (`src/utils/logger.rs`) — Log sekarang disimpan di
  global static dan bisa di-flush secara eksplisit saat shutdown via
  `flush_logs()`.
- **Path reorganization** — Semua data disimpan di `AppData\Local\AppBlocker\`:
  - `db/core.db` - Database
  - `logs/` - Log files
  - `reports/` - Audit reports
- **Log ke Database** — Logs proses (detected, blocked, allowed) sekarang
  disimpan ke tabel `logs` di database.
- **Runtime Metrics** (`src/metrics.rs`) — Menambahkan sistem metrik runtime
  dengan counter proses yang discan, dikill, overlay triggered, auth attempts,
  dan error count untuk observability internal.

### Diperbarui

- Cargo.toml: update versi ke 1.2.0
- Config seeding: semua config dari `default.toml` di-seed ke database
- Logging path defaults ke AppData
- **Dokumentasi CLI** — Memperbarui README.md dengan daftar lengkap perintah CLI administratif: `stats`, `top-blocked`, `audit-log`, `schedule-list`, `schedule-add`, `schedule-remove`, `schedule-toggle`, `add-blacklist`, `remove-blacklist`, `add-whitelist`, `remove-whitelist`, `upload-config`, `download-config`, `simulation-mode`, `run-simulation`, `run-production`. Semua perintah sekarang tercatat beserta contoh argumen umum.

---

## [1.1.3] - 2026-04-24

### Diketahui (Known Issues)

- **Monitoring berhenti setelah blockir** — Pada beberapa kasus, proses monitoring
  berhenti dan tidak mau melakukan blockir secara otomatis. Kemungkinan disebabkan
  oleh thread yang mati atau crash. Solusi sementara: restart aplikasi secara manual.
- **Bug load environment variable format argon2** — Program tidak dapat mendeteksi
  dengan baik tanda `$` pada password hash yang disimpan di file `.env`, yang
  menyebabkan hash tidak terdeteksi dengan baik.
- **Install script belum sepenuhnya berjalan** — Script instalasi belum sepenuhnya
  berfungsi.
- **Password hash disimpan di dalam program** — Untuk saat ini, password hash disimpan
  di dalam program sehingga perubahan password tidak akan berlaku. Solusi
  sementara: gunakan password default `Admin12345!`.

### Ditambahkan (New Features)

- **Running script** (`scripts/running_service.ps1`) — Script untuk menjalankan
  program tanpa membuka window log.

### Cara Instalasi Sementara

Untuk menjalankan prototype:

1. Salin atau unduh file `app_blocker.exe` ke direktori manapun, contoh:
   `C:\AppBlocker`
2. Salin folder `config` yang berisi `default.toml` dan `production.toml` ke
   direktori yang sama: `C:\AppBlocker\config`
3. Jalankan program via script `running_service.ps1` atau langsung via executable

---

## [1.1.2] - 2026-04-21

### Diperbaiki (Bug Fixes)

- **CRITICAL** Overlay transparan/invisible saat alt+tab — menghapus flag `WS_EX_LAYERED`
  dari `CreateWindowExW` yang menyebabkan window menjadi transparan; overlay sekarang
  terlihat dengan jelas
- **CRITICAL** Password hash corruption saat penyimpanan — hash argon2 sekarang disimpan
  dalam tanda kutip ganda di file `.env` untuk mencegah karakter `$` diinterpretasikan
  sebagai shell variable expansion; saat dibaca, tanda kutip dihapus otomatis
- **CRITICAL** Lock file stale setelah crash — menambahkan проверка apakah proses dengan
  PID tertayang masih berjalan; jika tidak (stale lock), file akan dihapus otomatis

### Diperbarui

- Menambahkan fungsi `is_process_running()` di `src/system/service.rs` untuk memeriksa
  status proses Windows
- Memperbarui `write_password_hash()` dan `read_env_vars()` di `src/config/env_loader.rs`

---

## [1.1.1] - 2026-04-19

### Diperbaiki (Bug Fixes)

- **CRITICAL** Pembersihan merge conflict markers (<<<<<< HEAD, =======, >>>>>>>) di 43 file
- **CRITICAL** Resolve unresolved imports - dependencies tidak ditemukan di dependency chain:
  - Hapus `src/security/memory.rs` (unused, tidak ada di mod.rs)
  - Hapus `src/security/encryption.rs` (unused, tidak ada di mod.rs)
  - Bersihkan semua unused imports di seluruh modul
- Hapus merge conflict residual di `.gitignore`, `CHANGELOG.md`, dan semua `src/utils/*.rs`
- Perbaiki import path di `src/utils/time.rs` - comment out `chrono::Timelike`
- Sederhanakan `RetryConfig` - hapus duplikasi field (`backoff_factor` vs `multiplier`)
- Konsistensi naming: `multiplier` → `backoff_factor` di seluruh retry logic
- Perbaiki typo di comment dan docstring

### Diperbarui

- Refactoring besar-besaran: ~2000 baris kode dihapus (net 2912 deleted, 931 added)
- Penyederhanaan modul: engine.rs (627 baris), monitor.rs (146 baris), process.rs (175 baris)
- Bersihkan semua dead code dan redundant imports

---

## [1.1.0] - 2026-04-18

### Diperbaiki (Bug Fixes)

- **CRITICAL** `Arc::try_unwrap` anti-pattern di `main.rs` — AuthManager kini dibagi via
  `Arc<Mutex<AuthManager>>` yang di-clone ke engine dan overlay callback; tidak ada lagi
  potensi panic saat Arc memiliki lebih dari satu referensi
- **CRITICAL** WM_TIMER ID 99 tidak dihandle di `overlay.rs` — overlay sekarang
  benar-benar menutup 800ms setelah unlock berhasil via `PostQuitMessage(0)`
- **CRITICAL** `ctrlc` handler adalah stub kosong — kini menggunakan crate `ctrlc` sungguhan
  dengan `set_handler` yang meng-set `AtomicBool` shutdown flag
- Config hot reload thread tidak pernah dijalankan — `spawn_config_watcher()` kini
  di-spawn di `main.rs` dan mengirim `AppEvent::ConfigReloaded` saat file berubah
- Win32 imports tidak lengkap di `overlay.rs` — semua symbol (`ES_PASSWORD`,
  `ES_AUTOHSCROLL`, `SetDlgItemTextW`, `BN_CLICKED`, dll.) kini diimport dengan benar
- Unused imports di beberapa modul (`HashMap`, `once_cell`, dll.) — semua dibersihkan
- `--config` flag tidak diteruskan ke `run-production` / `run-simulation` — kini
  `config_path` digunakan konsisten di seluruh startup sequence
- `IntegrityService::new()` menggunakan wrong import path untuk `IsDebuggerPresent`

### Ditambahkan (New Features)

- **Student Mode Policy** (`src/system/student_mode.rs`) — menonaktifkan Task Manager,
  regedit, dan CMD via registry saat overlay aktif; dikembalikan otomatis setelah unlock
- **Atomic JSON Audit Report** (`src/core/audit.rs`) — setiap event kritis ditulis ke
  `reports/audit_YYYY-MM-DD.jsonl` secara atomic, termasuk `session_duration_seconds`,
  `detection_method`, `schedule_rule_triggered`, dan `blocked_game_name`
- **Config File Watcher** (`src/config/hot_reload.rs`) — thread `notify`-based yang
  mendeteksi perubahan file config dan trigger hot reload via event bus
- **Failsafe overlay timeout** — overlay auto-unlock setelah N menit (default 30) via
  `WM_TIMER TIMER_FAILSAFE` agar sistem tidak terkunci permanen
- **Integration tests** (`src/tests/integration_tests.rs`) — 10 test skenario kompleks:
  channel disconnect, mutex contention, concurrent state access, kill loop safety,
  auth rate limiting, event system completeness
- `AuthManager::current_hash()` — method baru untuk mengambil hash password saat ini
- `ConfigManager::config_path()` — expose path config untuk file watcher

### Diperbarui

- Cargo.toml: tambahkan `ctrlc`, `hostname` ke dependencies utama (bukan dev-deps)
- Tambahkan `Win32_UI_Input_KeyboardAndMouse` dan `Win32_System_Diagnostics_Debug`
  ke fitur `windows` crate
- Versi diupdate ke `1.1.0`

---

## [1.0.0] - 2026-04-18

### Ditambahkan

- Arsitektur layered + hexagonal + event-driven dengan Rust 1.70+
- State machine 5 state: Monitoring, Blocking, Locked, Recovering, SafeMode
- Monitor thread (TX), Engine thread (RX), Watchdog thread
- Win32 Overlay fullscreen GDI dengan tema gelap
- Autentikasi Argon2id, password default `Admin12345!`
- Deteksi game: Roblox, Valorant, Steam, Epic Games, ML, Free Fire
- Behavior scoring, bypass detection (USB, rename, portable)
- Jadwal blokir timezone-aware (WIB), Senin-Jumat 07:00-15:00
- Safe kill logic - tidak pernah kill proses sistem
- Single instance lock, disable flag darurat
- Structured logging via tracing, rotasi harian
- CLI: 18 perintah administratif
- Windows Service install/uninstall via PowerShell
- GitHub Actions CI/CD pipeline
- Unit tests: auth, state, engine (18 test total)
