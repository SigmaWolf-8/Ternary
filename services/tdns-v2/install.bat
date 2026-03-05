@echo off
title PlenumNET TDNS Extension Installer
echo.
echo   PlenumNET TDNS - Browser Extension Installer
echo.

set "INSTALL_DIR=%LOCALAPPDATA%\PlenumNET\tdns-extension"
set "GH_RAW=https://raw.githubusercontent.com/SigmaWolf-8/Ternary/main/services/tdns-v2/extension/chromium"

if exist "%INSTALL_DIR%" rmdir /s /q "%INSTALL_DIR%"
mkdir "%INSTALL_DIR%" 2>nul

echo   Downloading extension files...

powershell -NoProfile -Command "(New-Object System.Net.WebClient).DownloadFile('%GH_RAW%/manifest.json', '%INSTALL_DIR%\manifest.json')"
powershell -NoProfile -Command "(New-Object System.Net.WebClient).DownloadFile('%GH_RAW%/background.js', '%INSTALL_DIR%\background.js')"
powershell -NoProfile -Command "(New-Object System.Net.WebClient).DownloadFile('%GH_RAW%/content.js', '%INSTALL_DIR%\content.js')"
powershell -NoProfile -Command "(New-Object System.Net.WebClient).DownloadFile('%GH_RAW%/popup.html', '%INSTALL_DIR%\popup.html')"
powershell -NoProfile -Command "(New-Object System.Net.WebClient).DownloadFile('%GH_RAW%/popup.js', '%INSTALL_DIR%\popup.js')"
powershell -NoProfile -Command "(New-Object System.Net.WebClient).DownloadFile('%GH_RAW%/rules.json', '%INSTALL_DIR%\rules.json')"
powershell -NoProfile -Command "(New-Object System.Net.WebClient).DownloadFile('%GH_RAW%/icon16.png', '%INSTALL_DIR%\icon16.png')"
powershell -NoProfile -Command "(New-Object System.Net.WebClient).DownloadFile('%GH_RAW%/icon48.png', '%INSTALL_DIR%\icon48.png')"
powershell -NoProfile -Command "(New-Object System.Net.WebClient).DownloadFile('%GH_RAW%/icon128.png', '%INSTALL_DIR%\icon128.png')"

set /a count=0
for %%f in ("%INSTALL_DIR%\*") do set /a count+=1

echo.
echo   [OK] Downloaded %count% files to:
echo   %INSTALL_DIR%
echo.

echo|set /p="%INSTALL_DIR%"| clip

echo   Folder path copied to clipboard.
echo.
echo   Opening Edge extensions page...
start edge://extensions

echo.
echo   TO FINISH:
echo     1. Enable "Developer mode" (top-right toggle)
echo     2. Click "Load unpacked"
echo     3. Paste the folder path (Ctrl+V) and press Enter
echo.
pause
