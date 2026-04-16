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
