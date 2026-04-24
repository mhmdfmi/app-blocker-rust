@REM This script creates a scheduled task to run the AppBlockerService at system startup with the highest privileges.

schtasks /create ^
/tn "AppBlockerService" ^
/tr "powershell -ExecutionPolicy Bypass -WindowStyle Hidden -File C:\AppBlocker\app_blocker.exe" ^
/sc onstart ^
/ru SYSTEM ^
/rl HIGHEST ^
/f