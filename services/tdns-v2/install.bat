@echo off
title PlenumNET TDNS Extension Installer v1.0.7
echo.
echo   PlenumNET TDNS - Browser Extension Installer v1.0.7
echo   Capomastro Holdings Ltd. - Applied Physics Division
echo.

set "GH_RAW=https://raw.githubusercontent.com/SigmaWolf-8/Ternary/main/services/tdns-v2/extension-chromium"
set "BASE_DIR=%LOCALAPPDATA%\PlenumNET\tdns-extensions"
set "OLD_DIR=%LOCALAPPDATA%\PlenumNET\tdns-extension"

rem Clean up old single-directory install
if exist "%OLD_DIR%" (
    echo   Removing old install at %OLD_DIR%...
    rmdir /s /q "%OLD_DIR%"
    echo   [OK] Old install removed
    echo.
    echo   IMPORTANT: If you previously loaded the extension from:
    echo     %OLD_DIR%
    echo   Go to your browser extensions page and REMOVE that entry first,
    echo   then load the new path shown below.
    echo.
)

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
echo   IMPORTANT: Remove any old PlenumNET TDNS extension
echo   from your browser FIRST, then load the new path.
echo.
echo   TO FINISH for each browser listed above:
echo     1. Open the extensions page (URL shown above)
echo     2. Remove any existing PlenumNET TDNS extension
echo     3. Enable "Developer mode" (top-right toggle)
echo     4. Click "Load unpacked"
echo     5. Browse to the folder path shown for that browser
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

powershell -NoProfile -Command "$cb=[DateTimeOffset]::UtcNow.ToUnixTimeSeconds(); [Net.ServicePointManager]::SecurityProtocol=[Net.SecurityProtocolType]::Tls12; $wc=New-Object System.Net.WebClient; $wc.Headers.Add('Cache-Control','no-cache'); $wc.Headers.Add('User-Agent','PlenumNET-Installer/1.0.7'); foreach($f in @('manifest.json','background.js','content.js','popup.html','popup.js','dimensions.json','report.html','report.js')){ $wc.DownloadFile('%GH_RAW%/'+$f+'?cb='+$cb, '%INSTALL_DIR%\'+$f) }; foreach($i in @('icon16.png','icon48.png','icon128.png')){ $wc.DownloadFile('%GH_RAW%/icons/'+$i+'?cb='+$cb, '%INSTALL_DIR%\icons\'+$i) }"

set /a count=0
for /r "%INSTALL_DIR%" %%f in (*) do set /a count+=1

rem Verify version
powershell -NoProfile -Command "$m = Get-Content '%INSTALL_DIR%\manifest.json' -Raw | ConvertFrom-Json; Write-Host ('  [%BROWSER_NAME%] Version: ' + $m.version); if ($m.version -ne '1.0.7') { Write-Host '  [%BROWSER_NAME%] WARNING: Expected v1.0.7 but got' $m.version -ForegroundColor Red }"

echo   [%BROWSER_NAME%] Downloaded %count% files
echo   [%BROWSER_NAME%] Path: %INSTALL_DIR%
echo   [%BROWSER_NAME%] Extensions page: %EXT_URL%
echo.

set /a installed+=1
goto :eof
