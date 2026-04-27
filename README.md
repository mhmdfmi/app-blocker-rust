# App Blocker — Sistem Pemblokiran Aplikasi Lab Komputer

[![App Blocker CI/CD](https://github.com/mhmdfmi/app-blocker-rust/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/mhmdfmi/app-blocker-rust/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-1.70%2B-orange.svg)](https://www.rust-lang.org)
[![Platform](https://img.shields.io/badge/Platform-Windows%2010%2F11-blue.svg)](https://www.microsoft.com/windows)
[![Version](https://img.shields.io/badge/Version-1.2.0-green.svg)](CHANGELOG.md)

> **Dikembangkan oleh Muhamad Fahmi — Asisten Kepala Lab Komputer**

Sistem produksi berbasis Rust untuk memblokir aplikasi terlarang (game, platform game)
di lab komputer selama jam operasional sekolah, dilengkapi overlay UI fullscreen Win32,
autentikasi Argon2id, dan audit logging terstruktur.

---

## Fitur Utama

| Fitur                    | Detail                                                            |
| ------------------------ | ----------------------------------------------------------------- |
| **Monitoring Real-time** | Scan proses setiap 2 detik via sysinfo                            |
| **Overlay Win32**        | Fullscreen, topmost, tidak bisa ditutup, GDI rendering            |
| **Auth Argon2id**        | Hash argon2id, rate limiting, lockout 5 menit                     |
| **Jadwal Blokir**        | Senin–Jumat 07:00-15:00, Sabtu 07:00-12:00 (WIB)                  |
| **Bypass Detection**     | Rename exe, USB execution, portable app                           |
| **Behavior Scoring**     | CPU spike, rapid spawn, suspicious path, hidden process           |
| **Safe Kill**            | Tidak pernah kill proses sistem (winlogon, csrss, explorer, dll.) |
| **Watchdog**             | Heartbeat monitoring, restart thread otomatis                     |
| **Hot Reload**           | Konfigurasi bisa diperbarui tanpa restart                         |
| **Mode Simulasi**        | Test tanpa benar-benar membunuh proses                            |
| **Windows Service**      | Install/uninstall via PowerShell, auto-restart                    |

---

## Lokasi Data

Semua data disimpan di `C:\Users\<user>\AppData\Local\AppBlocker\`:

| Folder     | Isi                              |
| ---------- | -------------------------------- |
| `db/`      | Database SQLite (`core.db`)      |
| `logs/`    | Log aplikasi (`app_blocker.log`) |
| `reports/` | Audit reports (`audit_*.jsonl`)  |

---

## Persyaratan

- **OS**: Windows 10 / Windows 11 (x86_64)
- **Rust**: 1.70+ (`rustup update stable`)
- **Privilege**: Administrator (untuk kill proses dan install service)

---

##Instalasi

### Build dari Source

```powershell
# Clone project
git clone https://github.com/mhmdfmi/app-blocker-rust.git
cd app_blocker-rust

# Build release
cargo build --release

# Output: target/release/app_blocker.exe
```

###Setup Awal (Pertama Kali)

```powershell
# Jalankan untuk inisialisasi database
.\target\release\app_blocker.exe run-production
```

Akan membuat:

- Database: `AppData\Local\AppBlocker\db\core.db`
- Tabel: configs, blacklists, whitelists, schedules, users, logs, audit_logs
- Config di-seed dari `config/default.toml`

###Install sebagai Service (Opsional)

```powershell
# Jalankan sebagai Administrator
.\scripts\install_service.ps1
```

###Setup Password

```powershell
# WAJIBdiganti setelah instalasi!
.\target\release\app_blocker.exe setup-password
```

Default password: `Admin12345!`

---

## Penggunaan CLI

```powershell
app_blocker.exe [PERINTAH]
```

| Perintah         | Deskripsi              |
| ---------------- | ---------------------- |
| `status`         | Status sistem          |
| `enable`         | Aktifkan pemblokiran   |
| `disable`        | Nonaktifkan darurat    |
| `logs -n 100`    | Tampilkan log terakhir |
| `setup-password` | Setup password         |
| `reset-password` | Reset password         |
| `list-blacklist` | Daftar diblokir        |
| `list-whitelist` | Daftar whitelist       |
| `run-simulation` | Jalankan simulasi      |
| `run-production` | Jalankan produksi      |
| `version`        | Info versi             |

---

## Konfigurasi

Edit `config/default.toml` lalu restart:

```toml
[app]
mode = "production"

[monitoring]
scan_interval_ms = 2000

[schedule]
enabled = true

[[schedule.rules]]
days = ["Monday", "Tuesday", "Wednesday", "Thursday", "Friday"]
start = "07:00"
end = "15:00"
action = "block_games"

[[blocking.blacklist]]
name = "Game Baru"
process_names = ["game.exe"]
paths = ["C:\\Games\\"]
description = "Deskripsi"
```

---

## Arsitektur

```
src/
├── main.rs              # Entry point
├── lib.rs              # Library exports
├── cli.rs             # CLI commands (clap)
├── bootstrap.rs        # Initialization
├── core/
│   ├── engine.rs       # Event handler, state machine
│   ├── monitor.rs     # Process scanner
│   ├── state.rs      # State management
│   ├── events.rs     # Event definitions
│   ├── watchdog.rs   # Thread health
│   └── audit.rs     # JSON audit
├── detection/
│   ├── mod.rs        # DetectionEngine
│   ├── game.rs      # Game matching
│   ├── behavior.rs  # Behavior scoring
│   ├── bypass.rs   # Bypass detection
│   └── schedule.rs # Schedule
├── ui/
│   ├── overlay.rs   # GDI overlay
│   ├── window.rs   # Window
│   └── input.rs   # Input
├── security/
│   ├── auth.rs     # Argon2id
│   └── integrity.rs # Checksums
├── system/
│   ├── process.rs  # Win32 APIs
│   ├── service.rs  # Service
│   └── student_mode.rs
├── config/
│   ├── settings.rs
│   └── hot_reload.rs
├── repository/    # DAL
│   └── *repo.rs
├── models/       # Data models
├── db/
│   ├── connection.rs
│   ├── init.rs    # Schema
└── utils/
    ├── error.rs
    └── logger.rs
```

### Struktur Database

```sql
-- core.db (SQLite)
configs      -- key, value, description
blacklists    -- name, process_names (JSON), paths, description
whitelists   -- name, process_names (JSON)
schedules    -- days (JSON), start_time, end_time, action
users       -- username, password_hash, role

logs         -- timestamp, process_name, action, reason, score
audit_logs  -- timestamp, event_type, details (JSON), success
```

### Alur Data

```
MONITOR THREAD
  │ scan setiap 2 detik
  │ cek schedule aktif
  │ │ProcessDetected
  ▼
ENGINE THREAD
  │ handle event
  │ kill process (Win32)
  │ trigger overlay
  │ audit (JSON + DB)
  │ │OverlayCallback
  ▼
UI THREAD
  │ display overlay
  │ verify password
  │ │UnlockSuccess
  ▼
(kembali ke monitoring)

WATCHDOG (parallel)
  │ receive heartbeat
  │ restart if dead
  ▼
```

### State Machine

```
Monitoring → Blocking → Locked → Recovering → Monitoring
                ↓
              SafeMode (on error)
```

---

## Disable Darurat

Buat file kosong:

```
AppData\Local\AppBlocker\disable
```

App masuk SafeMode dalam 2 detik.

---

## Keamanan

- **Password**: Argon2id hash, tidak plaintext
- **Memory**: Zero on drop
- **Safe Kill**: Protected processes tidak dihentikan
- **Single Instance**: Mencegah duplikasi

---

## Testing

```powershell
cargo test --all
```

---

## Lisensi

MIT - Lihat [LICENSE](LICENSE)

---

_Created by Muhamad Fahmi, Assistant Head of Computer Lab_
