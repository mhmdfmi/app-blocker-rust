# ============================================================
# App Blocker - Script Uninstalasi Windows Service
# Dikembangkan oleh Muhamad Fahmi, Asisten Kepala Lab Komputer
# Jalankan sebagai Administrator!
# ============================================================

#Requires -RunAsAdministrator

param(
    [string]$InstallDir  = "C:\AppBlocker",
    [string]$ServiceName = "AppBlockerService",
    [switch]$KeepLogs    = $false,
    [switch]$Force       = $false
)

$ErrorActionPreference = "Stop"

Write-Host "============================================================" -ForegroundColor Yellow
Write-Host " App Blocker - Uninstalasi Windows Service" -ForegroundColor Yellow
Write-Host "============================================================" -ForegroundColor Yellow
Write-Host ""

if (-not $Force) {
    $confirm = Read-Host "Yakin ingin menghapus App Blocker? (y/N)"
    if ($confirm -notmatch '^[Yy]$') {
        Write-Host "Uninstalasi dibatalkan." -ForegroundColor Green
        exit 0
    }
}

# ── 1. Hentikan dan hapus service ─────────────────────────────────────────────
Write-Host "[1/4] Menghentikan service '$ServiceName'..." -ForegroundColor Green
$svc = Get-Service -Name $ServiceName -ErrorAction SilentlyContinue
if ($svc) {
    if ($svc.Status -ne "Stopped") {
        try {
            Stop-Service -Name $ServiceName -Force -ErrorAction Stop
            Start-Sleep -Seconds 3
        } catch {
            Write-Host "      Warning: Gagal hentikan service, mungkin sudah berhenti." -ForegroundColor Yellow
        }
    }
    # Hapus service dengan sc.exe
    $deleteResult = sc.exe delete $ServiceName 2>&1
    if ($LASTEXITCODE -eq 0) {
        Write-Host "      Service dihapus."
    } else {
        Write-Host "      Service mungkin sudah dihapus atau tidak ada."
    }
} else {
    Write-Host "      Service tidak ditemukan (sudah dihapus)."
}

# ── 2. Hentikan proses yang masih berjalan ────────────────────────────────────
Write-Host "[2/4] Menghentikan proses app_blocker.exe..." -ForegroundColor Green
$procs = Get-Process -Name "app_blocker" -ErrorAction SilentlyContinue
if ($procs) {
    $procs | Stop-Process -Force -ErrorAction SilentlyContinue
    Start-Sleep -Seconds 2
    # Cek lagi jika ada yang masih berjalan
    $procs = Get-Process -Name "app_blocker" -ErrorAction SilentlyContinue
    if ($procs) {
        Write-Host "      Warning: Proses masih berjalan, forced kill..." -ForegroundColor Yellow
        $procs | Stop-Process -Force -ErrorAction SilentlyContinue
    }
    Write-Host "      Proses dihentikan."
} else {
    Write-Host "      Tidak ada proses yang berjalan."
}

# Hapus juga proses lain yang mungkin terkait
$otherProcs = Get-Process | Where-Object {
    $_.Name -like "*app_blocker*" -or $_.Name -like "*AppBlocker*"
} -ErrorAction SilentlyContinue
if ($otherProcs) {
    $otherProcs | Stop-Process -Force -ErrorAction SilentlyContinue
}

# ── 3. Bersihkan file ─────────────────────────────────────────────────────────
Write-Host "[3/4] Membersihkan file..." -ForegroundColor Green

# Hapus flag disable jika ada
$disableFlag = "$InstallDir\disable"
if (Test-Path $disableFlag) {
    Remove-Item $disableFlag -Force
}

# Hapus lock file
$lockFile = "$InstallDir\app.lock"
if (Test-Path $lockFile) {
    Remove-Item $lockFile -Force
}

if (Test-Path $InstallDir) {
    if ($KeepLogs) {
        # Hapus semua kecuali logs dan reports
        Get-ChildItem -Path $InstallDir -Exclude "logs","reports" |
            Remove-Item -Recurse -Force -ErrorAction SilentlyContinue
        Write-Host "      File dihapus (log dipertahankan di $InstallDir\logs)"
    } else {
        Remove-Item -Path $InstallDir -Recurse -Force -ErrorAction SilentlyContinue
        Write-Host "      Direktori $InstallDir dihapus."
    }
} else {
    Write-Host "      Direktori tidak ditemukan."
}

# ── 4. Bersihkan registry (jika ada) ─────────────────────────────────────────
Write-Host "[4/4] Membersihkan registry..." -ForegroundColor Green
$regPath = "HKLM:\SYSTEM\CurrentControlSet\Services\$ServiceName"
if (Test-Path $regPath) {
    Remove-Item -Path $regPath -Recurse -Force -ErrorAction SilentlyContinue
    Write-Host "      Registry dibersihkan."
} else {
    Write-Host "      Tidak ada entri registry."
}

Write-Host ""
Write-Host "============================================================" -ForegroundColor Green
Write-Host " Uninstalasi selesai! App Blocker telah dihapus." -ForegroundColor Green
if ($KeepLogs) {
    Write-Host " Log tersimpan di: $InstallDir\logs\" -ForegroundColor White
}
Write-Host "============================================================" -ForegroundColor Green
