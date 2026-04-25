<#
.SYNOPSIS
  Installer script untuk App Blocker sebagai Windows Service + Scheduled Task (SYSTEM).
.DESCRIPTION
  - Membuat folder instalasi, menyalin binary dan konfigurasi.
  - Menyiapkan .env dengan ADMIN_PASSWORD_HASH kosong (aplikasi akan generate default saat pertama run).
  - Mendaftarkan Windows Service (New-Service, fallback sc.exe).
  - Membuat Scheduled Task yang berjalan sebagai SYSTEM pada startup, dengan RunLevel Highest,
    hanya satu instance (IgnoreNew), restart otomatis jika mati, dan menjalankan executable dalam mode hidden.
  - Semua langkah dilengkapi pengecekan, logging, dan penanganan error.
.NOTES
  - Jalankan PowerShell sebagai Administrator.
  - Simpan file ini sebagai UTF-8 tanpa BOM.
#>

#Requires -RunAsAdministrator

param(
    [string]$InstallDir  = "C:\AppBlocker",
    [string]$ExePath     = "$PSScriptRoot\..\target\release\app_blocker.exe",
    [string]$ServiceName = "AppBlockerService",
    [string]$DisplayName = "App Blocker - Lab Computer Guard",
    [string]$Description = "Memblokir aplikasi terlarang di lab komputer selama jam operasional. Dikembangkan oleh Muhamad Fahmi.",
    [string]$TaskName    = "AppBlocker_OnStart_System",
    [switch]$ForceRecreateTask
)

$ErrorActionPreference = "Stop"

function Write-Title {
    param([string]$Text)
    Write-Host ""
    Write-Host "============================================================" -ForegroundColor Cyan
    Write-Host " $Text" -ForegroundColor Cyan
    Write-Host "============================================================" -ForegroundColor Cyan
    Write-Host ""
}

function Exit-WithError {
    param([string]$Message, [int]$Code = 1)
    Write-Host ""
    Write-Host "ERROR: $Message" -ForegroundColor Red
    Write-Host "Instalasi dibatalkan." -ForegroundColor Red
    exit $Code
}

# UI header
Write-Title "App Blocker - Installer (Service + Scheduled Task)"
Write-Host "Pastikan menjalankan PowerShell sebagai Administrator." -ForegroundColor Yellow
Write-Host ""

# 1. Periksa hak admin
try {
    $currentUser = [Security.Principal.WindowsIdentity]::GetCurrent()
    $principal   = New-Object Security.Principal.WindowsPrincipal($currentUser)
    if (-not $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)) {
        Exit-WithError "Script ini harus dijalankan sebagai Administrator!"
    }
} catch {
    Exit-WithError "Gagal memeriksa hak Administrator: $_"
}

# 2. Resolve paths
try {
    $resolvedScriptRoot = $PSScriptRoot
    $exeResolved = Resolve-Path -Path $ExePath -ErrorAction SilentlyContinue
    if ($exeResolved) {
        $ExePath = $exeResolved.Path
    } else {
        $candidate = Join-Path -Path $resolvedScriptRoot -ChildPath "..\target\release\app_blocker.exe"
        $candidateResolved = Resolve-Path -Path $candidate -ErrorAction SilentlyContinue
        if ($candidateResolved) { $ExePath = $candidateResolved.Path }
    }
} catch {
    Exit-WithError "Gagal mem-resolve path executable: $_"
}

# 3. Periksa executable
if (-not (Test-Path $ExePath)) {
    Exit-WithError "Executable tidak ditemukan: $ExePath`nJalankan terlebih dahulu: cargo build --release"
}

# 4. Buat direktori instalasi
try {
    Write-Host "[1/8] Membuat direktori instalasi: $InstallDir" -ForegroundColor Green
    $dirs = @($InstallDir, "$InstallDir\logs", "$InstallDir\reports", "$InstallDir\config")
    foreach ($dir in $dirs) {
        if (-not (Test-Path $dir)) {
            New-Item -ItemType Directory -Path $dir -Force | Out-Null
            Write-Host "      Dibuat: $dir" -ForegroundColor DarkGreen
        } else {
            Write-Host "      Sudah ada: $dir" -ForegroundColor DarkGray
        }
    }
} catch {
    Exit-WithError "Gagal membuat direktori instalasi: $_"
}

# 5. Copy executable
try {
    Write-Host "[2/8] Menyalin executable..." -ForegroundColor Green
    $destExe = Join-Path $InstallDir "app_blocker.exe"
    Copy-Item -Path $ExePath -Destination $destExe -Force -ErrorAction Stop
    Write-Host "      Executable: $destExe" -ForegroundColor DarkGreen
} catch {
    Exit-WithError "Gagal menyalin executable: $_"
}

# 6. Copy konfigurasi jika belum ada
try {
    Write-Host "[3/8] Menyalin konfigurasi default jika perlu..." -ForegroundColor Green
    $configSrc = Join-Path $resolvedScriptRoot "..\config\production.toml"
    $configSrcResolved = Resolve-Path -Path $configSrc -ErrorAction SilentlyContinue
    $configDst = Join-Path $InstallDir "config\default.toml"
    if ($configSrcResolved -and -not (Test-Path $configDst)) {
        Copy-Item -Path $configSrcResolved.Path -Destination $configDst -Force -ErrorAction Stop
        Write-Host "      Konfigurasi: $configDst" -ForegroundColor DarkGreen
    } elseif (-not $configSrcResolved) {
        Write-Host "      Sumber konfigurasi tidak ditemukan: $configSrc (abaikan jika tidak ada)" -ForegroundColor Yellow
    } else {
        Write-Host "      Konfigurasi sudah ada: $configDst" -ForegroundColor DarkGray
    }
} catch {
    Exit-WithError "Gagal menyalin konfigurasi: $_"
}

# 7. Buat .env default dengan ADMIN_PASSWORD_HASH kosong (app akan generate default saat pertama run)
try {
    Write-Host "[4/8] Menyiapkan file .env (ADMIN_PASSWORD_HASH dikosongkan)..." -ForegroundColor Green
    $envDst = Join-Path $InstallDir ".env"
    if (-not (Test-Path $envDst)) {
        $envContent = @"
# App Blocker - Konfigurasi Kredensial
# JANGAN bagikan file ini!

# ADMIN_PASSWORD_HASH dikosongkan; aplikasi akan generate default saat pertama kali dijalankan
ADMIN_PASSWORD_HASH=

APP_MODE=production
LOG_LEVEL=info
"@
        Set-Content -Path $envDst -Value $envContent -Encoding UTF8 -Force
        Write-Host "      .env dibuat: $envDst" -ForegroundColor DarkGreen
        Write-Host "      ADMIN_PASSWORD_HASH dikosongkan. Aplikasi akan meng-generate password default saat pertama run." -ForegroundColor Yellow
    } else {
        Write-Host "      .env sudah ada: $envDst" -ForegroundColor DarkGray
    }
} catch {
    Exit-WithError "Gagal membuat .env: $_"
}

# 8. Hapus service lama jika ada
try {
    Write-Host "[5/8] Memeriksa service lama..." -ForegroundColor Green
    $existingService = Get-Service -Name $ServiceName -ErrorAction SilentlyContinue
    if ($existingService) {
        Write-Host "      Service lama ditemukan: $ServiceName - mencoba menghentikan dan menghapus..." -ForegroundColor Yellow
        try { Stop-Service -Name $ServiceName -Force -ErrorAction Stop } catch { Write-Host "      Gagal menghentikan service (abaikan): $_" -ForegroundColor DarkYellow }
        Start-Sleep -Seconds 2
        Start-Process -FilePath sc.exe -ArgumentList "delete `"$ServiceName`"" -NoNewWindow -Wait -ErrorAction SilentlyContinue
        Start-Sleep -Seconds 2
        Write-Host "      Proses penghapusan service selesai (jika ada)." -ForegroundColor DarkGreen
    } else {
        Write-Host "      Tidak ada service lama." -ForegroundColor DarkGray
    }
} catch {
    Exit-WithError "Gagal memeriksa atau menghapus service lama: $_"
}

# 9. Daftarkan Windows Service (New-Service dengan fallback sc.exe)
try {
    Write-Host "[6/8] Mendaftarkan Windows Service..." -ForegroundColor Green

    $binary = $destExe
    $arguments = "run-production --config `"$InstallDir\config\default.toml`""
    $binPathName = "$binary $arguments"

    if (Get-Service -Name $ServiceName -ErrorAction SilentlyContinue) {
        Start-Process -FilePath sc.exe -ArgumentList "delete `"$ServiceName`"" -NoNewWindow -Wait -ErrorAction SilentlyContinue
        Start-Sleep -Seconds 2
    }

    try {
        New-Service -Name $ServiceName -BinaryPathName $binPathName -DisplayName $DisplayName -Description $Description -StartupType Automatic -ErrorAction Stop
        Write-Host "      Service '$ServiceName' terdaftar via New-Service." -ForegroundColor DarkGreen
    } catch {
        Write-Host "      New-Service gagal, mencoba fallback ke sc.exe..." -ForegroundColor Yellow
        $scBinPath = "`"$binary`" $arguments"
        $scArgs = "create `"$ServiceName`" binPath= $scBinPath DisplayName= `"$DisplayName`" start= auto obj= LocalSystem"
        Start-Process -FilePath sc.exe -ArgumentList $scArgs -NoNewWindow -Wait -ErrorAction Stop
        Start-Process -FilePath sc.exe -ArgumentList "description `"$ServiceName`" `"$Description`"" -NoNewWindow -Wait -ErrorAction SilentlyContinue
        Write-Host "      Service '$ServiceName' terdaftar via sc.exe." -ForegroundColor DarkGreen
    }

    # Set failure actions (sc.exe)
    try {
        Start-Process -FilePath sc.exe -ArgumentList "failure `"$ServiceName`" reset= 3600 actions= restart/5000/restart/10000/restart/30000" -NoNewWindow -Wait -ErrorAction SilentlyContinue
    } catch {
        Write-Host "      Peringatan: gagal mengatur failure actions (abaikan jika tidak didukung)." -ForegroundColor Yellow
    }

    Start-Sleep -Seconds 1
    $svc = Get-Service -Name $ServiceName -ErrorAction SilentlyContinue
    if (-not $svc) { Exit-WithError "Pendaftaran service gagal: $ServiceName" }
} catch {
    Exit-WithError "Gagal mendaftarkan service: $_"
}

# 10. Buat helper script run_hidden.ps1 yang memastikan single instance dan menjalankan exe hidden
try {
    Write-Host "[7/8] Menyiapkan Scheduled Task helper (run_hidden.ps1)..." -ForegroundColor Green
    $launcherPath = Join-Path $InstallDir "run_hidden.ps1"
    $launcherContent = @"
# Helper script untuk menjalankan AppBlocker sebagai single instance dan hidden window.
param()
try {
    \$exe = "`"$InstallDir\app_blocker.exe`""
    \$args = 'run-production --config `"$InstallDir\config\default.toml`"'

    # Pastikan tidak ada instance lain berjalan (cek nama executable)
    \$procName = [System.IO.Path]::GetFileNameWithoutExtension(\$exe)
    \$existing = Get-Process -ErrorAction SilentlyContinue | Where-Object { \$_.ProcessName -ieq \$procName }
    if (\$existing) {
        # Jika ada proses yang sama, keluar (jaga single instance)
        exit 0
    }

    # Start hidden
    Start-Process -FilePath \$exe -ArgumentList \$args -WindowStyle Hidden -WorkingDirectory "$InstallDir" -ErrorAction Stop
} catch {
    # Log ke file log agar mudah debugging
    \$logDir = "$InstallDir\logs"
    if (-not (Test-Path \$logDir)) { New-Item -ItemType Directory -Path \$logDir -Force | Out-Null }
    \$msg = \"[run_hidden] `$(Get-Date -Format o)` - ERROR: \$_`n\"
    Add-Content -Path (Join-Path \$logDir 'run_hidden.log') -Value \$msg
    exit 1
}
"@

    # Tulis file launcher
    Set-Content -Path $launcherPath -Value $launcherContent -Encoding UTF8 -Force
    Write-Host "      Launcher dibuat: $launcherPath" -ForegroundColor DarkGreen
} catch {
    Exit-WithError "Gagal membuat launcher run_hidden.ps1: $_"
}

# 11. Buat Scheduled Task (AtStartup) berjalan sebagai SYSTEM, RunLevel Highest, single instance, restart jika mati
try {
    Write-Host "[8/8] Mendaftarkan Scheduled Task (OnStart) sebagai SYSTEM..." -ForegroundColor Green

    # Jika task sudah ada dan user meminta recreate, hapus dulu
    $taskExists = Get-ScheduledTask -TaskName $TaskName -ErrorAction SilentlyContinue
    if ($taskExists -and $ForceRecreateTask) {
        Unregister-ScheduledTask -TaskName $TaskName -Confirm:$false -ErrorAction SilentlyContinue
        Start-Sleep -Seconds 1
        $taskExists = $null
    }

    if (-not $taskExists) {
        # Action: jalankan PowerShell untuk mengeksekusi launcher; launcher akan start exe hidden
        $action = New-ScheduledTaskAction -Execute "PowerShell.exe" -Argument "-NoProfile -WindowStyle Hidden -ExecutionPolicy Bypass -File `"$launcherPath`""

        # Trigger: At startup
        $trigger = New-ScheduledTaskTrigger -AtStartup

        # Settings: hanya 1 instance (IgnoreNew), restart 3 kali dengan interval 5s, run with highest privileges, hidden
        $settings = New-ScheduledTaskSettingsSet `
            -AllowStartIfOnBatteries:$true `
            -DontStopIfGoingOnBatteries:$true `
            -MultipleInstances IgnoreNew `
            -RestartCount 3 `
            -RestartInterval (New-TimeSpan -Seconds 5) `
            -ExecutionTimeLimit (New-TimeSpan -Hours 0) `
            -Hidden:$true

        # Principal: run as SYSTEM with highest privileges
        $principal = New-ScheduledTaskPrincipal -UserId "SYSTEM" -LogonType ServiceAccount -RunLevel Highest

        Register-ScheduledTask -TaskName $TaskName -Action $action -Trigger $trigger -Settings $settings -Principal $principal -Description "Start AppBlocker on system startup as SYSTEM (hidden, single instance)" -ErrorAction Stop

        Write-Host "      Scheduled Task '$TaskName' terdaftar (OnStart, SYSTEM, Highest)." -ForegroundColor DarkGreen
    } else {
        Write-Host "      Scheduled Task '$TaskName' sudah ada. Gunakan -ForceRecreateTask untuk membuat ulang." -ForegroundColor DarkGray
    }
} catch {
    Write-Host "      Peringatan: gagal mendaftarkan Scheduled Task via Register-ScheduledTask: $_" -ForegroundColor Yellow
    Write-Host "      Mencoba fallback menggunakan schtasks.exe..." -ForegroundColor Yellow
    try {
        # Fallback: gunakan schtasks (note: schtasks tidak mendukung MultipleInstances policy; best-effort)
        $psExec = "PowerShell.exe -NoProfile -WindowStyle Hidden -ExecutionPolicy Bypass -File `"$launcherPath`""
        $schtArgs = "/Create /TN `"$TaskName`" /TR `"$psExec`" /SC ONSTART /RL HIGHEST /F /RU SYSTEM"
        Start-Process -FilePath schtasks.exe -ArgumentList $schtArgs -NoNewWindow -Wait -ErrorAction Stop
        Write-Host "      Scheduled Task dibuat via schtasks.exe (fallback)." -ForegroundColor DarkGreen
    } catch {
        Write-Host "      Gagal membuat Scheduled Task via fallback schtasks.exe: $_" -ForegroundColor Red
        Write-Host "      Anda dapat membuat task manual dengan perintah berikut (jalankan sebagai Administrator):" -ForegroundColor Yellow
        Write-Host "      schtasks /Create /TN `"$TaskName`" /TR `"$psExec`" /SC ONSTART /RL HIGHEST /F /RU SYSTEM" -ForegroundColor White
    }
}

# 12. Pastikan service berjalan (jika service dibuat)
try {
    Write-Host ""
    Write-Host "Memulai service '$ServiceName' (jika ada)..." -ForegroundColor Green
    try {
        Start-Service -Name $ServiceName -ErrorAction Stop
        Start-Sleep -Seconds 2
    } catch {
        Write-Host "      Gagal memulai service via Start-Service (abaikan jika tidak diperlukan): $_" -ForegroundColor DarkYellow
    }

    $svc = Get-Service -Name $ServiceName -ErrorAction SilentlyContinue
    if ($svc -and $svc.Status -eq "Running") {
        Write-Host "✓ Service berhasil berjalan!" -ForegroundColor Green
    } else {
        Write-Host "Service status: $($svc.Status) - jika service tidak diperlukan, Scheduled Task akan menjalankan executable pada startup." -ForegroundColor Yellow
    }
} catch {
    Write-Host "Peringatan saat verifikasi service: $_" -ForegroundColor Yellow
}

# Final summary UI
Write-Host ""
Write-Host "============================================================" -ForegroundColor Cyan
Write-Host " Instalasi selesai!" -ForegroundColor Green
Write-Host " Direktori    : $InstallDir" -ForegroundColor White
Write-Host " Service      : $ServiceName" -ForegroundColor White
Write-Host " ScheduledTask: $TaskName (OnStart, SYSTEM, Highest)" -ForegroundColor White
Write-Host " Executable   : $destExe" -ForegroundColor White
Write-Host " Launcher     : $launcherPath" -ForegroundColor White
Write-Host " Log          : $InstallDir\logs\" -ForegroundColor White
Write-Host ""
Write-Host "Catatan:" -ForegroundColor Yellow
Write-Host " - ADMIN_PASSWORD_HASH dikosongkan di .env; aplikasi akan generate password default saat pertama run." -ForegroundColor Yellow
Write-Host " - Scheduled Task menjalankan launcher PowerShell yang memastikan hanya 1 instance dan menjalankan exe hidden." -ForegroundColor Yellow
Write-Host " - Jika ingin recreate Scheduled Task, jalankan script ini dengan parameter -ForceRecreateTask." -ForegroundColor Yellow
Write-Host "============================================================" -ForegroundColor Cyan
Write-Host ""
