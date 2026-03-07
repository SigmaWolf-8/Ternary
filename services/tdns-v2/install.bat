@echo off
title PlenumNET TDNS Extension Installer v1.0.4
echo.
echo   PlenumNET TDNS - Browser Extension Installer v1.0.4
echo   Capomastro Holdings Ltd. - Applied Physics Division
echo.

set "GH_RAW=https://raw.githubusercontent.com/SigmaWolf-8/Ternary/main/services/tdns-v2/extension-chromium"
set "BASE_DIR=%LOCALAPPDATA%\PlenumNET\tdns-extensions"

set /a installed=0

echo   Detecting browsers...
echo.

if exist "%LOCALAPPDATA%\Google\Chrome\User Data" (
    call :install_for "Chrome" "%BASE_DIR%\chrome" "chrome://extensions"
)
if exist "%LOCALAPPDATA%\Microsoft\Edge\User Data" (
    call :install_for "Edge" "%BASE_DIR%\edge" "edge://extensions"
)
if exist "%LOCALAPPDATA%\BraveSoftware\Brave-Browser\User Data" (
    call :install_for "Brave" "%BASE_DIR%\brave" "brave://extensions"
)
if exist "%LOCALAPPDATA%\Vivaldi\User Data" (
    call :install_for "Vivaldi" "%BASE_DIR%\vivaldi" "vivaldi://extensions"
)
if exist "%APPDATA%\Opera Software\Opera Stable" (
    call :install_for "Opera" "%BASE_DIR%\opera" "opera://extensions"
)
if exist "%LOCALAPPDATA%\Arc\User Data" (
    call :install_for "Arc" "%BASE_DIR%\arc" "arc://extensions"
)

echo.
if %installed% EQU 0 (
    echo   No supported Chromium browsers detected.
    echo   Installing to default location...
    call :install_for "Default" "%BASE_DIR%\chromium" "chrome://extensions"
)

echo.
echo   ======================================================
echo   TO FINISH for each browser listed above:
echo     1. Open the extensions page (URL shown above)
echo     2. Enable "Developer mode" (top-right toggle)
echo     3. Click "Load unpacked"
echo     4. Browse to the folder path shown for that browser
echo   ======================================================
echo.
pause
goto :eof

:install_for
set "BROWSER_NAME=%~1"
set "INSTALL_DIR=%~2"
set "EXT_URL=%~3"

echo   [%BROWSER_NAME%] Installing to %INSTALL_DIR%

if exist "%INSTALL_DIR%" rmdir /s /q "%INSTALL_DIR%"
mkdir "%INSTALL_DIR%" 2>nul
mkdir "%INSTALL_DIR%\icons" 2>nul

powershell -NoProfile -Command "(New-Object System.Net.WebClient).DownloadFile('%GH_RAW%/manifest.json', '%INSTALL_DIR%\manifest.json')"
powershell -NoProfile -Command "(New-Object System.Net.WebClient).DownloadFile('%GH_RAW%/background.js', '%INSTALL_DIR%\background.js')"
powershell -NoProfile -Command "(New-Object System.Net.WebClient).DownloadFile('%GH_RAW%/content.js', '%INSTALL_DIR%\content.js')"
powershell -NoProfile -Command "(New-Object System.Net.WebClient).DownloadFile('%GH_RAW%/popup.html', '%INSTALL_DIR%\popup.html')"
powershell -NoProfile -Command "(New-Object System.Net.WebClient).DownloadFile('%GH_RAW%/popup.js', '%INSTALL_DIR%\popup.js')"
powershell -NoProfile -Command "(New-Object System.Net.WebClient).DownloadFile('%GH_RAW%/dimensions.json', '%INSTALL_DIR%\dimensions.json')"
powershell -NoProfile -Command "(New-Object System.Net.WebClient).DownloadFile('%GH_RAW%/report.html', '%INSTALL_DIR%\report.html')"
powershell -NoProfile -Command "(New-Object System.Net.WebClient).DownloadFile('%GH_RAW%/report.js', '%INSTALL_DIR%\report.js')"
powershell -NoProfile -Command "(New-Object System.Net.WebClient).DownloadFile('%GH_RAW%/icons/icon16.png', '%INSTALL_DIR%\icons\icon16.png')"
powershell -NoProfile -Command "(New-Object System.Net.WebClient).DownloadFile('%GH_RAW%/icons/icon48.png', '%INSTALL_DIR%\icons\icon48.png')"
powershell -NoProfile -Command "(New-Object System.Net.WebClient).DownloadFile('%GH_RAW%/icons/icon128.png', '%INSTALL_DIR%\icons\icon128.png')"

set /a count=0
for /r "%INSTALL_DIR%" %%f in (*) do set /a count+=1

echo   [%BROWSER_NAME%] Downloaded %count% files
echo   [%BROWSER_NAME%] Path: %INSTALL_DIR%
echo   [%BROWSER_NAME%] Extensions page: %EXT_URL%
echo.

set /a installed+=1
goto :eof
