<#
.SYNOPSIS
  Uninstaller script untuk App Blocker (service + scheduled task + files + registry).
.DESCRIPTION
  - Menghentikan dan menghapus Windows Service.
  - Menghentikan proses yang berjalan.
  - Menghapus Scheduled Task (Register-ScheduledTask / schtasks fallback).
  - Menghapus launcher (run_hidden.ps1), flag, lock, dan direktori instalasi (opsional).
  - Membersihkan registry service entry.
  - Menyediakan opsi untuk mempertahankan log dan memaksa tanpa konfirmasi.
.NOTES
  - Jalankan PowerShell sebagai Administrator.
  - Simpan file ini sebagai UTF-8 tanpa BOM.
#>

#Requires -RunAsAdministrator

param(
    [string]$InstallDir  = "C:\AppBlocker",
    [string]$ServiceName = "AppBlockerService",
    [string]$TaskName    = "AppBlocker_OnStart_System",
    [switch]$KeepLogs    = $false,
    [switch]$Force       = $false
)

$ErrorActionPreference = "Stop"

function Write-Title {
    param([string]$Text)
    Write-Host ""
    Write-Host "============================================================" -ForegroundColor Yellow
    Write-Host " $Text" -ForegroundColor Yellow
    Write-Host "============================================================" -ForegroundColor Yellow
    Write-Host ""
}

function Exit-WithError {
    param([string]$Message, [int]$Code = 1)
    Write-Host ""
    Write-Host "ERROR: $Message" -ForegroundColor Red
    Write-Host "Uninstall dibatalkan." -ForegroundColor Red
    exit $Code
}

Write-Title "App Blocker - Uninstalasi (Service + Scheduled Task)"

# Konfirmasi kecuali Force
if (-not $Force) {
    try {
        $confirm = Read-Host "Yakin ingin menghapus App Blocker sepenuhnya? (y/N)"
    } catch {
        Exit-WithError "Input gagal: $_"
    }
    if ($confirm -notmatch '^[Yy]$') {
        Write-Host "Uninstalasi dibatalkan oleh pengguna." -ForegroundColor Green
        exit 0
    }
}

# 1. Hentikan dan hapus Scheduled Task (jika ada)
Write-Host "[1/6] Menghapus Scheduled Task (jika ada): $TaskName" -ForegroundColor Green
try {
    $task = Get-ScheduledTask -TaskName $TaskName -ErrorAction SilentlyContinue
    if ($task) {
        try {
            Unregister-ScheduledTask -TaskName $TaskName -Confirm:$false -ErrorAction Stop
            Write-Host "      Scheduled Task '$TaskName' dihapus (Register-ScheduledTask)." -ForegroundColor DarkGreen
        } catch {
            Write-Host "      Gagal menghapus via Unregister-ScheduledTask: $_" -ForegroundColor Yellow
            Write-Host "      Mencoba fallback dengan schtasks.exe..." -ForegroundColor Yellow
            try {
                Start-Process -FilePath schtasks.exe -ArgumentList "/Delete /TN `"$TaskName`" /F" -NoNewWindow -Wait -ErrorAction Stop
                Write-Host "      Scheduled Task '$TaskName' dihapus via schtasks.exe." -ForegroundColor DarkGreen
            } catch {
                Write-Host "      Gagal menghapus Scheduled Task via schtasks.exe: $_" -ForegroundColor Red
                Write-Host "      Anda dapat menghapus manual dengan:" -ForegroundColor Yellow
                Write-Host "        schtasks /Delete /TN `"$TaskName`" /F" -ForegroundColor White
            }
        }
    } else {
        Write-Host "      Scheduled Task '$TaskName' tidak ditemukan." -ForegroundColor DarkGray
    }
} catch {
    Write-Host "      Peringatan saat memeriksa Scheduled Task: $_" -ForegroundColor Yellow
}

# 2. Hentikan dan hapus Windows Service
Write-Host "[2/6] Menghentikan dan menghapus Service: $ServiceName" -ForegroundColor Green
try {
    $svc = Get-Service -Name $ServiceName -ErrorAction SilentlyContinue
    if ($svc) {
        if ($svc.Status -ne 'Stopped') {
            try {
                Stop-Service -Name $ServiceName -Force -ErrorAction Stop
                Start-Sleep -Seconds 2
                Write-Host "      Service dihentikan." -ForegroundColor DarkGreen
            } catch {
                Write-Host "      Gagal menghentikan service via Stop-Service: $_" -ForegroundColor Yellow
            }
        } else {
            Write-Host "      Service sudah berhenti." -ForegroundColor DarkGray
        }

        # Hapus service via sc.exe (lebih andal untuk penghapusan)
        try {
            Start-Process -FilePath sc.exe -ArgumentList "delete `"$ServiceName`"" -NoNewWindow -Wait -ErrorAction Stop
            Start-Sleep -Seconds 1
            Write-Host "      Perintah hapus service (sc.exe) telah dijalankan." -ForegroundColor DarkGreen
        } catch {
            Write-Host "      Gagal menghapus service via sc.exe: $_" -ForegroundColor Yellow
            Write-Host "      Coba hapus registry service di langkah berikutnya." -ForegroundColor Yellow
        }
    } else {
        Write-Host "      Service '$ServiceName' tidak ditemukan." -ForegroundColor DarkGray
    }
} catch {
    Write-Host "      Peringatan saat memeriksa service: $_" -ForegroundColor Yellow
}

# 3. Hentikan proses yang masih berjalan (app_blocker)
Write-Host "[3/6] Menghentikan proses app_blocker.exe jika masih berjalan" -ForegroundColor Green
try {
    $procCandidates = Get-Process -ErrorAction SilentlyContinue | Where-Object {
        $_.Name -like "app_blocker" -or $_.Name -like "*AppBlocker*"
    }
    if ($procCandidates) {
        foreach ($p in $procCandidates) {
            try {
                Stop-Process -Id $p.Id -Force -ErrorAction Stop
                Write-Host "      Proses dihentikan: $($p.ProcessName) (PID $($p.Id))" -ForegroundColor DarkGreen
            } catch {
                Write-Host "      Gagal menghentikan PID $($p.Id): $_" -ForegroundColor Yellow
            }
        }
        Start-Sleep -Seconds 2
    } else {
        Write-Host "      Tidak ada proses app_blocker yang berjalan." -ForegroundColor DarkGray
    }
} catch {
    Write-Host "      Peringatan saat menghentikan proses: $_" -ForegroundColor Yellow
}

# 4. Hapus launcher, flag, lock, dan file lain (tetap simpan logs jika diminta)
Write-Host "[4/6] Membersihkan file dan launcher" -ForegroundColor Green
try {
    $launcherPath = Join-Path $InstallDir "run_hidden.ps1"
    $disableFlag = Join-Path $InstallDir "disable"
    $lockFile = Join-Path $InstallDir "app.lock"
    $launcherLog = Join-Path $InstallDir "logs\run_hidden.log"

    # Hapus launcher
    if (Test-Path $launcherPath) {
        try { Remove-Item -Path $launcherPath -Force -ErrorAction Stop; Write-Host "      Launcher dihapus: $launcherPath" -ForegroundColor DarkGreen } catch { Write-Host "      Gagal hapus launcher: $_" -ForegroundColor Yellow }
    } else { Write-Host "      Launcher tidak ditemukan." -ForegroundColor DarkGray }

    # Hapus flag dan lock
    if (Test-Path $disableFlag) { Remove-Item -Path $disableFlag -Force -ErrorAction SilentlyContinue; Write-Host "      Flag disable dihapus." -ForegroundColor DarkGreen }
    if (Test-Path $lockFile)    { Remove-Item -Path $lockFile -Force -ErrorAction SilentlyContinue; Write-Host "      Lock file dihapus." -ForegroundColor DarkGreen }

    # Hapus logs/reports kecuali jika KeepLogs
    if (Test-Path $InstallDir) {
        if ($KeepLogs) {
            # Hapus semua kecuali logs dan reports
            try {
                Get-ChildItem -Path $InstallDir -Force | Where-Object { $_.Name -notin @("logs","reports") } | ForEach-Object {
                    Remove-Item -Path $_.FullName -Recurse -Force -ErrorAction SilentlyContinue
                }
                Write-Host "      File dihapus; logs dan reports dipertahankan." -ForegroundColor DarkGreen
            } catch {
                Write-Host "      Peringatan saat membersihkan direktori (KeepLogs): $_" -ForegroundColor Yellow
            }
        } else {
            try {
                Remove-Item -Path $InstallDir -Recurse -Force -ErrorAction Stop
                Write-Host "      Direktori $InstallDir dihapus sepenuhnya." -ForegroundColor DarkGreen
            } catch {
                Write-Host "      Gagal menghapus direktori $InstallDir: $_" -ForegroundColor Yellow
                Write-Host "      Coba hapus manual atau periksa permission." -ForegroundColor Yellow
            }
        }
    } else {
        Write-Host "      Direktori instalasi tidak ditemukan: $InstallDir" -ForegroundColor DarkGray
    }
} catch {
    Write-Host "      Peringatan saat membersihkan file: $_" -ForegroundColor Yellow
}

# 5. Bersihkan registry service entry jika masih ada
Write-Host "[5/6] Membersihkan registry service entry (jika ada)" -ForegroundColor Green
try {
    $regPath = "HKLM:\SYSTEM\CurrentControlSet\Services\$ServiceName"
    if (Test-Path $regPath) {
        try {
            Remove-Item -Path $regPath -Recurse -Force -ErrorAction Stop
            Write-Host "      Registry service entry dihapus: $regPath" -ForegroundColor DarkGreen
        } catch {
            Write-Host "      Gagal menghapus registry service entry: $_" -ForegroundColor Yellow
            Write-Host "      Anda dapat menghapus manual via regedit jika perlu." -ForegroundColor Yellow
        }
    } else {
        Write-Host "      Tidak ada entri registry untuk service $ServiceName." -ForegroundColor DarkGray
    }
} catch {
    Write-Host "      Peringatan saat membersihkan registry: $_" -ForegroundColor Yellow
}

# 6. Final verification: pastikan service dan task tidak ada
Write-Host "[6/6] Verifikasi akhir" -ForegroundColor Green
$stillService = Get-Service -Name $ServiceName -ErrorAction SilentlyContinue
if ($stillService) {
    Write-Host "      Peringatan: service masih terdaftar dengan status: $($stillService.Status)" -ForegroundColor Yellow
    Write-Host "      Coba hapus manual: sc.exe delete `"$ServiceName`"" -ForegroundColor Yellow
} else {
    Write-Host "      Service tidak ditemukan. OK." -ForegroundColor DarkGreen
}

$stillTask = Get-ScheduledTask -TaskName $TaskName -ErrorAction SilentlyContinue
if ($stillTask) {
    Write-Host "      Peringatan: Scheduled Task '$TaskName' masih ada." -ForegroundColor Yellow
    Write-Host "      Coba hapus manual: schtasks /Delete /TN `"$TaskName`" /F" -ForegroundColor Yellow
} else {
    Write-Host "      Scheduled Task tidak ditemukan. OK." -ForegroundColor DarkGreen
}

Write-Host ""
Write-Host "============================================================" -ForegroundColor Green
Write-Host " Uninstalasi selesai! App Blocker telah dihapus (sebagian atau seluruhnya)." -ForegroundColor Green
if ($KeepLogs) {
    Write-Host " Log tersimpan di: $InstallDir\logs\" -ForegroundColor White
}
Write-Host " Jika ada item yang masih tersisa, jalankan perintah manual yang ditampilkan di atas." -ForegroundColor Yellow
Write-Host "============================================================" -ForegroundColor Green
