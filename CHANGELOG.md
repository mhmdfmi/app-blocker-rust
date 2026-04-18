# CHANGELOG

Format mengikuti [Keep a Changelog](https://keepachangelog.com/id/1.0.0/)
dan [Semantic Versioning](https://semver.org/lang/id/).

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
