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
