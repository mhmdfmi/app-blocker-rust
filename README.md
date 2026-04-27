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
app_blocker.exe [GLOBAL_OPTIONS] [PERINTAH]
```

**Global Options:**

| Option                | Deskripsi                                   | Default               |
| --------------------- | ------------------------------------------- | --------------------- |
| `--config <PATH>`     | Path file konfigurasi                       | `config/default.toml` |
| `--log-level <LEVEL>` | Level logging (trace/debug/info/warn/error) | `info`                |

### Monitoring & Operasi

| Perintah           | Deskripsi                                                                           |
| ------------------ | ----------------------------------------------------------------------------------- |
| `status`           | Tampilkan status sistem (mode, simulasi, jumlah blacklist/whitelist, scan interval) |
| `enable`           | Aktifkan pemblokiran                                                                |
| `disable --yes`    | Nonaktifkan pemblokiran darurat (tanpa konfirmasi interaktif)                       |
| `logs --lines <N>` | Tampilkan `N` baris log terakhir                                                    |
| `run-simulation`   | Jalankan dalam mode simulasi (tidak benar-benar kill proses)                        |
| `run-production`   | Jalankan dalam mode produksi (kill proses nyata)                                    |
| `version`          | Tampilkan versi dan informasi build                                                 |

### Manajemen Daftar (Blacklist & Whitelist)

| Perintah                                       | Deskripsi                                     | Contoh                                                 |
| ---------------------------------------------- | --------------------------------------------- | ------------------------------------------------------ |
| `list-blacklist`                               | Daftar semua aplikasi yang diblokir           | —                                                      |
| `add-blacklist --name <EXE> --app-name <NAMA>` | Tambah satu proses ke blacklist               | `add-blacklist --name game.exe --app-name "Game Baru"` |
| `add-blacklist --file <PATH>`                  | Import blacklist dari file JSON (single/bulk) | `add-blacklist --file examples/blacklist_bulk.json`    |
| `remove-blacklist <NAME>`                      | Hapus aplikasi dari blacklist                 | `remove-blacklist game.exe`                            |
| `list-whitelist`                               | Daftar semua proses whitelist                 | —                                                      |
| `add-whitelist --name <EXE>`                   | Tambah proses ke whitelist                    | `add-whitelist --name chrome.exe`                      |
| `add-whitelist --file <PATH>`                  | Import whitelist dari file JSON               | `add-whitelist --file examples/whitelist_bulk.json`    |
| `remove-whitelist <NAME>`                      | Hapus proses dari whitelist                   | `remove-whitelist chrome.exe`                          |

### Konfigurasi & Simulasi

| Perintah                          | Deskripsi                                        | Contoh                                 |
| --------------------------------- | ------------------------------------------------ | -------------------------------------- |
| `simulation-mode <true/false>`    | Ubah mode simulasi di database                   | `simulation-mode true`                 |
| `upload-config <FILE>`            | Upload file konfigurasi baru (.json/.toml/.yaml) | `upload-config config_baru.toml`       |
| `download-config --output <FILE>` | Export konfigurasi saat ini ke file TOML         | `download-config --output backup.toml` |

### Statistik & Audit

| Perintah                              | Deskripsi                                  | Contoh                   |
| ------------------------------------- | ------------------------------------------ | ------------------------ |
| `stats --period <day/week/month>`     | Statistik jumlah pemblokiran per periode   | `stats --period week`    |
| `top-blocked --limit <N>`             | Top `N` proses yang paling sering diblokir | `top-blocked --limit 10` |
| `audit-log --user <NAME> --limit <N>` | Log aktivitas admin (audit trail)          | `audit-log --limit 50`   |

### Jadwal (Schedule)

| Perintah                                                                     | Deskripsi                    | Contoh                                                                              |
| ---------------------------------------------------------------------------- | ---------------------------- | ----------------------------------------------------------------------------------- |
| `schedule-list`                                                              | Tampilkan semua jadwal aktif | —                                                                                   |
| `schedule-add --days <HARI> --start <HH:MM> --end <HH:MM> --action <ACTION>` | Tambah jadwal baru           | `schedule-add --days "Senin,Selasa" --start 07:00 --end 15:00 --action block_games` |
| `schedule-remove --id <ID>`                                                  | Hapus jadwal berdasarkan ID  | `schedule-remove --id 1`                                                            |
| `schedule-toggle --id <ID>`                                                  | Aktifkan/nonaktifkan jadwal  | `schedule-toggle --id 1`                                                            |

### Autentikasi

| Perintah         | Deskripsi                                                  |
| ---------------- | ---------------------------------------------------------- |
| `setup-password` | Setup password admin pertama kali                          |
| `reset-password` | Reset password admin (memerlukan verifikasi password lama) |

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
├── metrics.rs          # Runtime metrics (scanned, killed, overlay, auth, errors)
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

## UI Overlay

Saat aplikasi terlarang terdeteksi, overlay fullscreen akan muncul:

```
┌──────────────────────────────────────────────────────────┐
│                                                          │
│                    ⚠️ AKSES DIBLOKIR ⚠️                  │
│                                                          │
│            Aplikasi berikut diblokir:                    │
│                                                          │
│               [Nama Proses] - Roblox                     │
│                                                          │
│               PC: LAB-KOMPUTER-01                        │
│               Waktu: 07:45:23                            │
│                                                          │
│     _______________________________________________      │
│     |                                             |      │
│     |         MASUKAN PASSWORD ADMIN              |      │
│     |                                             |      │
│     |     [ ********************************* ]   |      │
│     |                                             |      │
│     |______________[ UNLOCK ]_____________________|      │
│                                                          │
│     Percobaan: 1/5  │  Ter kunci: 00:30                  │
│                                                          │
└──────────────────────────────────────────────────────────┘
```

**Fitur UI:**

- Fullscreen, topmost, tidak bisa di-minimize
- Background gelap transparan
- Input password unified (tidak ada karakter terlihat)
- Timeout auto-unlock 30 menit (failsafe)
- Tampilkan: nama proses, PC name, timestamp, attempt counter

---

## Format Log

Log tersimpan di `AppData\Local\AppBlocker\logs\app_blocker.log`:

### Log Baris Contoh

```json
{"timestamp":"2026-04-27T07:45:23.123Z","level":"INFO","message":"App Blocker v1.2.0 dimulai","version":"1.2.0","mode":"production","db_path":"C:\\Users\\fahmi\\AppData\\Local\\AppBlocker\\db\\core.db"}

{"timestamp":"2026-04-27T07:45:25.456Z","level":"INFO","message":"Monitor thread dimulai"}

{"timestamp":"2026-04-27T07:45:30.789Z","level":"INFO","message":"Total proses yang terdeteksi: 128"}

{"timestamp":"2026-04-27T07:45:32.012Z","level":"WARN","message":"Komponen tidak responsif","component":"ConfigWatcher","missed":3,"restarts":0}

{"timestamp":"2026-04-27T07:45:33.345Z","level":"ERROR","message":"Thread mati terdeteksi","component":"UiOverlay","reason":"missed_heartbeat"}
```

### Cara Baca Log

| Level | Arti                |
| ----- | ------------------- |
| INFO  | Event normal        |
| WARN  | Peringatan          |
| ERROR | Error kritis        |
| DEBUG | Debug (jika detail) |

### Contoh Pencarian

```powershell
# Cari semua blocked
Select-String -Path "app_blocker.log" -Pattern "blocked"

# Cari error hari ini
Select-String -Path "app_blocker.log" -Pattern "ERROR"

# Proses tertentu
Select-String -Path "app_blocker.log" -Pattern "Roblox"
```

---

## Backup & Restore

### Backup Database

```powershell
# Copy database
Copy-Item "$env:APPDATA\AppBlocker\db\core.db" "backup_core.db"

# Copy config
Copy-Item "config\default.toml" "backup_config.toml"
```

### Backup Lengkap

```powershell
# Backup semua data
$backupDir = "AppBlocker_Backup_$(Get-Date -Format 'yyyyMMdd_HHmmss')"
New-Item -ItemType Directory -Path $backupDir

Copy-Item "$env:APPDATA\AppBlocker\db" "$backupDir\db"
Copy-Item "$env:APPDATA\AppBlocker\logs" "$backupDir\logs"
Copy-Item "$env:APPDATA\AppBlocker\reports" "$backupDir\reports"
Copy-Item "config\default.toml" "$backupDir\"

Write-Host "Backup ke: $backupDir"
```

### Restore

```powershell
# Stop service
Stop-Service AppBlocker -ErrorAction SilentlyContinue

# Restore database
Copy-Item "backup_core.db" "$env:APPDATA\AppBlocker\db\core.db"

# Start service
Start-Service AppBlocker
```

### Reset ke Default

```powershell
# Hapus database (akan dibuat ulang saat start)
Remove-Item "$env:APPDATA\AppBlocker\db\core.db"

# Restart
.\app_blocker.exe run-production
```

---

## Performa & Batas Sistem

### Batas yang Direkomendasikan

| Komponen      | Batas             |
| ------------- | ----------------- |
| CPU usage     | < 20%             |
| Memory        | < 200 MB          |
| Proses discan | ~100-200 per scan |
| Scan interval | 2000ms (default)  |

### Thread Usage

| Thread   | Fungsi                   |
| -------- | ------------------------ |
| Main     | Event loop, CLI          |
| Monitor  | Scan proses (1 thread)   |
| Engine   | Event handler (1 thread) |
| Watchdog | Health check (1 thread)  |
| UI       | Overlay (1 thread)       |

**Total: ~5 threads**

### Kapasitas Database

- **logs**: ~10.000 baris per hari (estimasi)
- **audit_logs**: ~1.000 baris per hari
- **Monitoring**: Tidak ada batasan proses

---

## Troubleshooting

### Error: "Database locked"

```powershell
# Hapus lock file
Remove-Item "$env:APPDATA\AppBlocker\db\core.db-wal" -ErrorAction SilentlyContinue
Remove-Item "$env:APPDATA\AppBlocker\db\core.db-shm" -ErrorAction SilentlyContinue

# Restart
.\app_blocker.exe run-production
```

### Error: "Single instance already running"

```powershell
# Cek proses yang berjalan
Get-Process | Where-Object {$_.Name -like "*app_blocker*"}

# Atau cek lock file
Test-Path "$env:APPDATA\AppBlocker\app.lock"
```

### Error: "Access denied" saat kill proses

```powershell
# Jalankan sebagai Administrator
Start-Process cmd -ArgumentList "/c app_blocker.exe" -Verb RunAs
```

### Overlay tidak muncul

1. **Cek schedule aktif**: Pastikan jam sekarang dalam range jadwal blokir
2. **Cek state**: Jalankan `app_blocker.exe status`
3. **Cek log**: Lihat error di `logs/app_blocker.log`

### Proses tidak diblokir

1. **Cek blacklist**: `app_blocker.exe list-blacklist`
2. **Cek schedule**: Pastikan `schedule.enabled = true`
3. **Cek log**: Cari "process detected" di log
4. **Cek mode**: Pastikan `mode = "production"` bukan `"simulation"`

### Watchdog error "Thread mati"

Biasanya false positive, sudah diperbaiki di v1.2.0. Jika masih terjadi:

- Update ke versi terbaru
- Cek log untuk error lain

---

## FAQ

**Q: Apa bedanya Simulation vs Production?**
A: Simulation hanya logging, tidak kill proses. Gunakan untuk testing.

**Q: Kenapa overlay tidak muncul saat proses diblokir?**
A: Cek schedule aktif (harus dalam jam blokir), atau cek state dengan `status`.

**Q: Bagaimana reset ke config default?**
A: Hapus database, jalankan ulang aplikasi.

**Q: Apakah bisa running tanpa service?**
A: Ya, langsung jalankan `.\app_blocker.exe run-production`

**Q: Berapa lama untuk auto-unlock jika lupa password?**
A: Failsafe timeout 30 menit (konfigurasi di config).

**Q: Bisakah dimintai dari luar lab?**
A: Tidak, aplikasi berjalan lokal di setiap PC.

**Q: Bagaimana cara menambah game baru?**
A: Edit `config/default.toml`, tambah ke section `[[blocking.blacklist]]`, lalu restart.

**Q: Apakah log bisa dihapus otomatis?**
A: Ya,rotasi harian. Semua log > 7 hari bisa dihapus manual.

**Q: Password default tidak bisa?**
A: Pastikan dijalankan sebagai Administrator, atau reset dengan `setup-password`.

**Q: App startup lambat?**
A: Normal, ada delay 5 detik untuk whitelist process scan.

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
