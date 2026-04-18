<<<<<<< HEAD
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
        Stop-Service -Name $ServiceName -Force -ErrorAction SilentlyContinue
        Start-Sleep -Seconds 3
    }
    sc.exe delete $ServiceName | Out-Null
    Write-Host "      Service dihapus."
} else {
    Write-Host "      Service tidak ditemukan (sudah dihapus)."
}

# ── 2. Hentikan proses yang masih berjalan ────────────────────────────────────
Write-Host "[2/4] Menghentikan proses app_blocker.exe..." -ForegroundColor Green
$procs = Get-Process -Name "app_blocker" -ErrorAction SilentlyContinue
if ($procs) {
    $procs | Stop-Process -Force
    Write-Host "      Proses dihentikan."
} else {
    Write-Host "      Tidak ada proses yang berjalan."
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
=======
# App Blocker Service Uninstallation Script
# Requires Administrator privileges

$ErrorActionPreference = "Stop"

$SERVICE_NAME = "AppBlocker"

Write-Host "App Blocker Service Uninstaller" -ForegroundColor Cyan
Write-Host "==============================" -ForegroundColor Cyan

# Check admin
$isAdmin = ([Security.Principal.WindowsPrincipal] [Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
if (-not $isAdmin) {
    Write-Error "This script requires Administrator privileges."
    exit 1
}

# Check if service exists
$service = Get-Service -Name $SERVICE_NAME -ErrorAction SilentlyContinue
if (-not $service) {
    Write-Host "Service does not exist. Nothing to do." -ForegroundColor Yellow
    exit 0
}

# Stop service
Write-Host "Stopping service..." -ForegroundColor Yellow
Stop-Service -Name $SERVICE_NAME -Force -ErrorAction SilentlyContinue
Start-Sleep -Seconds 2

# Delete service
Write-Host "Deleting service..." -ForegroundColor Yellow
sc.exe delete $SERVICE_NAME

Write-Host ""
Write-Host "App Blocker service uninstalled successfully!" -ForegroundColor Green
>>>>>>> bce0345919f371d153ccb843f2ddbfb5e8695c5f
