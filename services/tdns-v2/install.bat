@echo off
title PlenumNET TDNS Extension Installer v2.3.3
echo.
echo   PlenumNET TDNS - Browser Extension Installer
echo.

set "INSTALL_DIR=%LOCALAPPDATA%\PlenumNET\tdns-extension"
set "GH_RAW=https://raw.githubusercontent.com/SigmaWolf-8/Ternary/main/extension-chromium"

if exist "%INSTALL_DIR%" rmdir /s /q "%INSTALL_DIR%"
mkdir "%INSTALL_DIR%" 2>nul
mkdir "%INSTALL_DIR%\icons" 2>nul

echo   Downloading extension files...

powershell -NoProfile -Command "(New-Object System.Net.WebClient).DownloadFile('%GH_RAW%/manifest.json', '%INSTALL_DIR%\manifest.json')"
powershell -NoProfile -Command "(New-Object System.Net.WebClient).DownloadFile('%GH_RAW%/background.js', '%INSTALL_DIR%\background.js')"
powershell -NoProfile -Command "(New-Object System.Net.WebClient).DownloadFile('%GH_RAW%/content.js', '%INSTALL_DIR%\content.js')"
powershell -NoProfile -Command "(New-Object System.Net.WebClient).DownloadFile('%GH_RAW%/popup.html', '%INSTALL_DIR%\popup.html')"
powershell -NoProfile -Command "(New-Object System.Net.WebClient).DownloadFile('%GH_RAW%/popup.js', '%INSTALL_DIR%\popup.js')"
powershell -NoProfile -Command "(New-Object System.Net.WebClient).DownloadFile('%GH_RAW%/dimensions.json', '%INSTALL_DIR%\dimensions.json')"
powershell -NoProfile -Command "(New-Object System.Net.WebClient).DownloadFile('%GH_RAW%/icons/icon16.png', '%INSTALL_DIR%\icons\icon16.png')"
powershell -NoProfile -Command "(New-Object System.Net.WebClient).DownloadFile('%GH_RAW%/icons/icon48.png', '%INSTALL_DIR%\icons\icon48.png')"
powershell -NoProfile -Command "(New-Object System.Net.WebClient).DownloadFile('%GH_RAW%/icons/icon128.png', '%INSTALL_DIR%\icons\icon128.png')"

set /a count=0
for /r "%INSTALL_DIR%" %%f in (*) do set /a count+=1

echo.
echo   [OK] Downloaded %count% files to:
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
