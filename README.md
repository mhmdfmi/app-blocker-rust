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
| **Monitoring Real-time** | Scan proses setiap 1.5–2 detik via sysinfo                        |
| **Overlay Win32**        | Fullscreen, topmost, tidak bisa ditutup, GDI rendering            |
| **Auth Argon2id**        | Hash bcrypt-grade, rate limiting, lockout 5 menit                 |
| **Jadwal Blokir**        | Senin–Jumat 07:00–15:00, Sabtu 07:00–12:00 (WIB)                  |
| **Bypass Detection**     | Rename exe, USB execution, portable app                           |
| **Behavior Scoring**     | CPU spike, rapid spawn, suspicious path, hidden process           |
| **Safe Kill**            | Tidak pernah kill proses sistem (winlogon, csrss, explorer, dll.) |
| **Watchdog**             | Heartbeat monitoring, restart thread otomatis                     |
| **Hot Reload**           | Konfigurasi bisa diperbarui tanpa restart                         |
| **Mode Simulasi**        | Test tanpa benar-benar membunuh proses                            |
| **Windows Service**      | Install/uninstall via PowerShell, auto-restart                    |

---

## Lokasi Data

Semua data disimpan di `C:\Users\<username>\AppData\Local\AppBlocker\`:

| Folder      | Isi                            |
|------------|--------------------------------|
| `db/`      | Database SQLite (`core.db`)        |
| `logs/`    | Log aplikasi (`app_blocker.log`)  |
| `reports/` | Audit reports (`audit_*.jsonl`) |

---

## Persyaratan

- **OS**: Windows 10 / Windows 11 (x86_64)
- **Rust**: 1.70+ (`rustup update stable`)
- **Privilege**: Administrator (untuk kill proses dan install service)
- **Vc Redist x86_64**: Dibutuhkan untuk execute file

---

## Instalasi Cepat

### Opsi 1: Prototype (Tanpa Build)

Untuk menjalankan prototype tanpa build dari source:

1. Unduh atau salin file `app_blocker.exe` ke direktori pilihan, contoh:
   `C:\AppBlocker`
2. Salin folder `config` yang berisi `default.toml` dan `production.toml` ke
   direktori yang sama: `C:\AppBlocker\config`
3. Jalankan program:

```powershell
# Opsi 1
# Lewat script (tanpa window log)
.\scripts\running_service.ps1

# Opsi 2
# Buat sebagai task schedule
.\script\task_scheduler.bat

# Opsi 3
# Atau langsung via executable
.\app_blocker.exe run-production
```

### Opsi 2: Build dari Source

```powershell
# Clone atau ekstrak proyek
cd app_blocker

# Build release
cargo build --release

# Binary ada di: target\release\app_blocker.exe
```

### 2. Instalasi sebagai Windows Service

```powershell
# Jalankan sebagai Administrator
.\scripts\install_service.ps1
```

Script ini akan:

- Menyalin binary ke `C:\AppBlocker\`
- Mendaftarkan Windows Service dengan auto-restart
- Generate hash password default `Admin12345!`

### 3. Setup Password Admin

```powershell
# Ganti password default (WAJIB!)
.\target\release\app_blocker.exe reset-password
```

---

## Penggunaan CLI

```powershell
app_blocker.exe [PERINTAH] [OPSI]
```

| Perintah                                               | Deskripsi                        |
| ------------------------------------------------------ | -------------------------------- |
| `enable`                                               | Aktifkan pemblokiran             |
| `disable`                                              | Nonaktifkan darurat              |
| `status`                                               | Status sistem saat ini           |
| `logs -n 100`                                          | Tampilkan 100 baris log terakhir |
| `setup-password`                                       | Setup password pertama kali      |
| `reset-password`                                       | Reset password admin             |
| `list-blacklist`                                       | Daftar aplikasi yang diblokir    |
| `add-blacklist --name game.exe --app-name "Nama Game"` | Tambah ke blacklist              |
| `remove-blacklist game.exe`                            | Hapus dari blacklist             |
| `list-whitelist`                                       | Daftar whitelist                 |
| `add-whitelist chrome.exe`                             | Tambah ke whitelist              |
| `simulation-mode true`                                 | Aktifkan mode simulasi           |
| `run-simulation`                                       | Jalankan dalam mode simulasi     |
| `run-production`                                       | Jalankan dalam mode produksi     |
| `version`                                              | Info versi dan build             |

---

## Konfigurasi

Edit `config/default.toml`:

```toml
[app]
mode = "production"
startup_delay_seconds = 15

[monitoring]
scan_interval_ms = 2000

[schedule]
enabled = true
timezone = "Asia/Jakarta"

[[schedule.rules]]
days = ["Monday", "Tuesday", "Wednesday", "Thursday", "Friday"]
start = "07:00"
end = "15:00"
action = "block_games"
```

### Tambah Game ke Blacklist

```toml
[[blocking.blacklist]]
name = "Nama Game"
process_names = ["game.exe"]
paths = ["C:\\Games\\NamaGame\\"]
description = "Deskripsi game"
```

---

## Arsitektur

```
app_blocker/
├── src/
│   ├── core/           # Engine, monitor, state machine, events, watchdog
│   ├── detection/      # Game detector, behavior scorer, bypass, schedule
│   ├── ui/             # Win32 overlay, GDI rendering, input handler
│   ├── security/       # Argon2 auth, SHA-256 integrity, memory zeroing
│   ├── system/         # Process Win32, user info, keyboard hooks, service
│   ├── config/         # Settings, env loader, hot reload, validator
│   ├── utils/          # Error (thiserror), logger (tracing), time, retry
│   └── constants/      # Pesan UI Bahasa Indonesia, path sistem
├── config/
│   ├── default.toml    # Konfigurasi default
│   └── production.toml # Konfigurasi produksi
├── examples/ # Contoh File Konfigurasi
└── scripts/
    ├── install_service.ps1
    ├── uninstall_service.ps1
    └── running_service.ps1
```

### Alur Data

```
Monitor Thread (TX)
    │ scan proses setiap N ms
    │ kirim ProcessDetected event
    ▼
Engine Thread (RX)
    │ handle event, transisi state
    │ kill proses via Win32 API
    │ trigger overlay callback
    ▼
UI Thread
    │ tampilkan Win32 overlay fullscreen
    │ tunggu input password
    │ verifikasi via Argon2
    │ kirim UnlockSuccess/Failed event
    ▼
Engine Thread
    │ terima UnlockSuccess
    │ transisi ke Recovering → Monitoring
    ▼
(kembali ke monitoring)

Watchdog Thread (parallel)
    │ pantau heartbeat semua thread
    │ restart thread yang mati
    └ force SafeMode jika gagal restart
```

### State Machine

```
Monitoring ──(proses terdeteksi)──► Blocking
Blocking   ──(kill berhasil)──────► Locked
Locked     ──(unlock berhasil)────► Recovering
Recovering ──(cleanup selesai)────► Monitoring
any        ──(error kritis)───────► SafeMode
SafeMode   ──(manual recovery)────► Monitoring
```

---

## Keamanan

- **Password**: Hash Argon2id, tidak pernah disimpan plaintext
- **Memory**: Data sensitif di-zero saat drop (zeroize crate)
- **Anti-bypass**: Deteksi rename, USB, portable app, debugger
- **Safe kill**: Daftar protected processes tidak bisa dihentikan
- **Integrity**: SHA-256 self-hash dan config hash
- **Single instance**: Lock file mencegah duplikasi
- **Disable flag**: File `C:\AppBlocker\disable` untuk emergency stop

---

## Default Password

**`Admin12345!`**

> ⚠️ **WAJIB ganti segera setelah instalasi!**
>
> ```powershell
> app_blocker.exe reset-password
> ```

---

## Disable Darurat

Jika sistem perlu dihentikan segera tanpa akses CLI:

1. Buat file kosong: `C:\AppBlocker\disable`
2. App Blocker otomatis masuk SafeMode dalam 2 detik
3. Hapus file untuk mengaktifkan kembali

---

## Testing

```powershell
# Jalankan semua unit test
cargo test --all

# Test dengan output verbose
cargo test --all -- --nocapture

# Test spesifik
cargo test auth_tests
cargo test state_tests
cargo test engine_tests
```

---

## Uninstalasi

```powershell
# Uninstalasi lengkap
.\scripts\uninstall_service.ps1

# Pertahankan log
.\scripts\uninstall_service.ps1 -KeepLogs

# Tanpa konfirmasi
.\scripts\uninstall_service.ps1 -Force
```

---

## Lisensi

MIT License — Lihat [LICENSE](LICENSE)

---

## Credit

_Dikembangkan oleh **Muhamad Fahmi**, Asisten Kepala Lab Komputer_
_This program was developed by Muhamad Fahmi, Assistant Head of the Computer Lab._
