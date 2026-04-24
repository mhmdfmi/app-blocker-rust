# Force strict mode
Set-StrictMode -Version Latest

# Stop on error
$ErrorActionPreference = "Stop"

# Log file (optional tapi sangat disarankan)
$logFile = "C:\AppBlocker\logs\service.log"

function Write-Log {
    param ($msg)
    $time = Get-Date -Format "yyyy-MM-dd HH:mm:ss"
    Add-Content -Path $logFile -Value "[$time] $msg"
}

try {
    Write-Log "Service started"

    # Path ke exe
    $exePath = "C:\AppBlocker\app_blocker.exe"

    if (!(Test-Path $exePath)) {
        Write-Log "ERROR: EXE tidak ditemukan di $exePath"
        exit 1
    }

    # Jalankan exe hidden
    Start-Process -FilePath $exePath -WindowStyle Hidden

    Write-Log "App Blocker dijalankan"

} catch {
    Write-Log "FATAL ERROR: $_"
}