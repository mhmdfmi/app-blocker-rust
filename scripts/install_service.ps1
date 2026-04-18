<<<<<<< HEAD
# ============================================================
# App Blocker - Script Instalasi Windows Service
# Dikembangkan oleh Muhamad Fahmi, Asisten Kepala Lab Komputer
# Jalankan sebagai Administrator!
# ============================================================

#Requires -RunAsAdministrator

param(
    [string]$InstallDir  = "C:\AppBlocker",
    [string]$ExePath     = "$PSScriptRoot\..\target\release\app_blocker.exe",
    [string]$ServiceName = "AppBlockerService",
    [string]$DisplayName = "App Blocker - Lab Computer Guard",
    [string]$Description = "Memblokir aplikasi terlarang di lab komputer selama jam operasional. Dikembangkan oleh Muhamad Fahmi."
)

$ErrorActionPreference = "Stop"

Write-Host "============================================================" -ForegroundColor Cyan
Write-Host " App Blocker - Instalasi Windows Service" -ForegroundColor Cyan
Write-Host " Dikembangkan oleh Muhamad Fahmi, Asisten Kepala Lab" -ForegroundColor Cyan
Write-Host "============================================================" -ForegroundColor Cyan
Write-Host ""

# ── 1. Periksa hak admin ─────────────────────────────────────────────────────
$currentUser = [Security.Principal.WindowsIdentity]::GetCurrent()
$principal   = New-Object Security.Principal.WindowsPrincipal($currentUser)
if (-not $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)) {
    Write-Error "Script ini harus dijalankan sebagai Administrator!"
    exit 1
}

# ── 2. Periksa executable ─────────────────────────────────────────────────────
if (-not (Test-Path $ExePath)) {
    Write-Error "Executable tidak ditemukan: $ExePath"
    Write-Host "Jalankan terlebih dahulu: cargo build --release" -ForegroundColor Yellow
    exit 1
}

# ── 3. Buat direktori instalasi ───────────────────────────────────────────────
Write-Host "[1/6] Membuat direktori instalasi: $InstallDir" -ForegroundColor Green
$dirs = @($InstallDir, "$InstallDir\logs", "$InstallDir\reports", "$InstallDir\config")
foreach ($dir in $dirs) {
    if (-not (Test-Path $dir)) {
        New-Item -ItemType Directory -Path $dir -Force | Out-Null
        Write-Host "      Dibuat: $dir"
    }
}

# ── 4. Copy file ───────────────────────────────────────────────────────────────
Write-Host "[2/6] Menyalin file..." -ForegroundColor Green
Copy-Item -Path $ExePath -Destination "$InstallDir\app_blocker.exe" -Force
Write-Host "      Executable: $InstallDir\app_blocker.exe"

# Copy konfigurasi jika belum ada
$configSrc = "$PSScriptRoot\..\config\production.toml"
$configDst = "$InstallDir\config\default.toml"
if ((Test-Path $configSrc) -and (-not (Test-Path $configDst))) {
    Copy-Item -Path $configSrc -Destination $configDst -Force
    Write-Host "      Konfigurasi: $configDst"
}

# Copy .env template jika belum ada
$envSrc = "$PSScriptRoot\..\.env"
$envDst = "$InstallDir\.env"
if ((Test-Path $envSrc) -and (-not (Test-Path $envDst))) {
    Copy-Item -Path $envSrc -Destination $envDst -Force
    Write-Host "      Kredensial : $envDst"
}

# ── 5. Hapus service lama jika ada ────────────────────────────────────────────
Write-Host "[3/6] Memeriksa service lama..." -ForegroundColor Green
$existingService = Get-Service -Name $ServiceName -ErrorAction SilentlyContinue
if ($existingService) {
    Write-Host "      Menghentikan service lama..." -ForegroundColor Yellow
    Stop-Service -Name $ServiceName -Force -ErrorAction SilentlyContinue
    Start-Sleep -Seconds 2
    sc.exe delete $ServiceName | Out-Null
    Write-Host "      Service lama dihapus."
}

# ── 6. Daftarkan Windows Service menggunakan sc.exe ───────────────────────────
Write-Host "[4/6] Mendaftarkan Windows Service..." -ForegroundColor Green
$binPath = "`"$InstallDir\app_blocker.exe`" run-production --config `"$InstallDir\config\default.toml`""

sc.exe create $ServiceName `
    binPath= $binPath `
    DisplayName= $DisplayName `
    start= auto `
    obj= "LocalSystem" | Out-Null

sc.exe description $ServiceName $Description | Out-Null
sc.exe failure $ServiceName reset= 3600 actions= restart/5000/restart/10000/restart/30000 | Out-Null

Write-Host "      Service '$ServiceName' terdaftar."

# ── 7. Set izin direktori ─────────────────────────────────────────────────────
Write-Host "[5/6] Mengatur izin direktori..." -ForegroundColor Green
$acl = Get-Acl $InstallDir
$rule = New-Object System.Security.AccessControl.FileSystemAccessRule(
    "SYSTEM", "FullControl", "ContainerInherit,ObjectInherit", "None", "Allow"
)
$acl.SetAccessRule($rule)
Set-Acl -Path $InstallDir -AclObject $acl
Write-Host "      Izin SYSTEM: FullControl pada $InstallDir"

# ── 8. Setup password default ─────────────────────────────────────────────────
Write-Host "[6/6] Konfigurasi awal..." -ForegroundColor Green
Write-Host "      Kata sandi default: Admin12345!" -ForegroundColor Yellow
Write-Host "      SEGERA ganti dengan: app_blocker.exe reset-password" -ForegroundColor Red

# ── 9. Mulai service ──────────────────────────────────────────────────────────
Write-Host ""
Write-Host "Memulai service '$ServiceName'..." -ForegroundColor Green
try {
    Start-Service -Name $ServiceName
    Start-Sleep -Seconds 2
    $svc = Get-Service -Name $ServiceName
    if ($svc.Status -eq "Running") {
        Write-Host "✓ Service berhasil berjalan!" -ForegroundColor Green
    } else {
        Write-Warning "Service status: $($svc.Status) - periksa Event Viewer"
    }
} catch {
    Write-Warning "Gagal memulai service: $_"
    Write-Host "Coba jalankan manual: sc.exe start $ServiceName"
}

Write-Host ""
Write-Host "============================================================" -ForegroundColor Cyan
Write-Host " Instalasi selesai!" -ForegroundColor Green
Write-Host " Direktori    : $InstallDir" -ForegroundColor White
Write-Host " Service      : $ServiceName" -ForegroundColor White
Write-Host " Log          : $InstallDir\logs\" -ForegroundColor White
Write-Host " Reset pass   : app_blocker.exe reset-password" -ForegroundColor White
Write-Host "============================================================" -ForegroundColor Cyan
=======
# App Blocker Service Installation Script
# Requires Administrator privileges

$ErrorActionPreference = "Stop"

$SERVICE_NAME = "AppBlocker"
$DISPLAY_NAME = "App Blocker Service"
$DESCRIPTION = "Windows Application Blocker Service"
$EXE_PATH = Join-Path $PSScriptRoot "target\release\app_blocker.exe"

Write-Host "App Blocker Service Installer" -ForegroundColor Cyan
Write-Host "===========================" -ForegroundColor Cyan

# Check admin
$isAdmin = ([Security.Principal.WindowsPrincipal] [Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
if (-not $isAdmin) {
    Write-Error "This script requires Administrator privileges."
    exit 1
}

# Check if binary exists
if (-not (Test-Path $EXE_PATH)) {
    Write-Error "Binary not found at: $EXE_PATH"
    Write-Host "Please build the project first: cargo build --release"
    exit 1
}

# Check if service exists
$existingService = Get-Service -Name $SERVICE_NAME -ErrorAction SilentlyContinue
if ($existingService) {
    Write-Host "Service already exists. Stopping and removing..." -ForegroundColor Yellow
    Stop-Service -Name $SERVICE_NAME -Force -ErrorAction SilentlyContinue
    sc.exe delete $SERVICE_NAME
    Start-Sleep -Seconds 2
}

# Create service
Write-Host "Creating service..." -ForegroundColor Green
sc.exe create $SERVICE_NAME binPath= "$EXE_PATH" start= auto
sc.exe description $SERVICE_NAME $DESCRIPTION

# Set recovery options
sc.exe failure $SERVICE_NAME reset= 86400 actions= restart/5000/restart/5000/restart/5000

# Start service
Write-Host "Starting service..." -ForegroundColor Green
Start-Service -Name $SERVICE_NAME

Write-Host ""
Write-Host "App Blocker service installed successfully!" -ForegroundColor Green
Write-Host "Service Status: $((Get-Service -Name $SERVICE_NAME).Status)" -ForegroundColor Cyan
>>>>>>> bce0345919f371d153ccb843f2ddbfb5e8695c5f
