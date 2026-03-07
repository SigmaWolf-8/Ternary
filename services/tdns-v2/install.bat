@echo off
title PlenumNET TDNS Extension Installer v1.0.4
echo.
echo   PlenumNET TDNS - Browser Extension Installer v1.0.4
echo.

set "INSTALL_DIR=%LOCALAPPDATA%\PlenumNET\tdns-extension"
set "ZIP_URL=https://plenumnet.replit.app/api/extension-zip"
set "ZIP_FILE=%TEMP%\plenumnet-tdns-extension.zip"

if exist "%INSTALL_DIR%" rmdir /s /q "%INSTALL_DIR%"
mkdir "%INSTALL_DIR%" 2>nul

echo   Downloading extension package...

powershell -NoProfile -Command "try { [Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12; (New-Object System.Net.WebClient).DownloadFile('%ZIP_URL%', '%ZIP_FILE%'); Write-Host '  [OK] Downloaded'; } catch { Write-Host '  [FAIL] Download error:' $_.Exception.Message; exit 1 }"

if not exist "%ZIP_FILE%" (
    echo   ERROR: Download failed. Check your internet connection.
    pause
    exit /b 1
)

echo   Extracting...
powershell -NoProfile -Command "Expand-Archive -Path '%ZIP_FILE%' -DestinationPath '%INSTALL_DIR%' -Force"

del "%ZIP_FILE%" 2>nul

set /a count=0
for /r "%INSTALL_DIR%" %%f in (*) do set /a count+=1

echo.
echo   [OK] Installed %count% files to:
echo   %INSTALL_DIR%
echo.

echo|set /p="%INSTALL_DIR%"| clip

echo   Folder path copied to clipboard.
echo.
echo   Opening Chrome extensions page...
start chrome://extensions

echo.
echo   TO FINISH:
echo     1. Enable "Developer mode" (top-right toggle)
echo     2. Click "Load unpacked"
echo     3. Paste the folder path (Ctrl+V) and press Enter
echo.
pause
