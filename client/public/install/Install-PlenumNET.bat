@echo off
setlocal EnableDelayedExpansion
title PlenumNET Installer v2.3.2
color 0B

set "VERSION=2.3.2"
set "REPO=https://github.com/SigmaWolf-8/Ternary"
set "INSTALLDIR=C:\PlenumNET"

echo.
echo   ========================================================
echo     PlenumNET Installer v%VERSION%
echo     Salvi Framework - Post-Quantum Internet Infrastructure
echo     Capomastro Holdings Ltd.
echo   ========================================================
echo.
echo   Install location: %INSTALLDIR%
echo.

:: ---- Step 1: Check Git ----
echo   Step 1 of 4: Checking prerequisites
echo   -----------------------------------
where git >nul 2>nul
if %ERRORLEVEL% NEQ 0 (
    echo.
    echo   [ERROR] Git is not installed.
    echo.
    echo   Please install Git for Windows first:
    echo   https://git-scm.com/download/win
    echo.
    set /p OPENGIT="  Open Git download page now? (Y/N): "
    if /i "!OPENGIT!"=="Y" start https://git-scm.com/download/win
    echo.
    echo   After installing Git, run this installer again.
    echo.
    pause
    exit /b 1
)
for /f "tokens=*" %%i in ('git --version 2^>^&1') do set "GITVER=%%i"
echo     [OK] Git: %GITVER%

where cargo >nul 2>nul
if %ERRORLEVEL% NEQ 0 (
    echo     [--] Rust/Cargo: not installed
    echo          Get it from: https://rustup.rs
    set "HASCARGO=0"
) else (
    for /f "tokens=*" %%i in ('cargo --version 2^>^&1') do set "CARGOVER=%%i"
    echo     [OK] Rust: !CARGOVER!
    set "HASCARGO=1"
)
echo.

:: ---- Step 2: Download ----
echo   Step 2 of 4: Downloading PlenumNET
echo   -----------------------------------
if exist "%INSTALLDIR%\.git" (
    echo     Found existing installation. Updating...
    pushd "%INSTALLDIR%"
    git pull origin main
    popd
) else (
    if exist "%INSTALLDIR%" (
        echo     Directory exists but is not a git repo. Removing...
        rmdir /s /q "%INSTALLDIR%"
    )
    echo     Cloning PlenumNET repository...
    git clone %REPO% "%INSTALLDIR%"
    if %ERRORLEVEL% NEQ 0 (
        echo.
        echo   [ERROR] Download failed. Check your internet connection.
        echo.
        pause
        exit /b 1
    )
)
echo     Download complete.
echo.

:: ---- Step 3: Build ----
echo   Step 3 of 4: Building framework
echo   -----------------------------------
if "%HASCARGO%"=="1" (
    echo     Building all modules (this may take several minutes)...
    echo.
    pushd "%INSTALLDIR%"
    cargo build --release
    if !ERRORLEVEL! EQU 0 (
        echo.
        echo     Build successful!
    ) else (
        echo.
        echo     Build had errors. Source code is still available at %INSTALLDIR%.
        echo     You can retry later: cd %INSTALLDIR% ^& cargo build --release
    )
    popd
) else (
    echo     Skipping build (Rust not installed).
    echo.
    echo     To build later:
    echo       1. Install Rust from https://rustup.rs
    echo       2. Open a new Command Prompt
    echo       3. Run:  cd %INSTALLDIR% ^& cargo build --release
)
echo.

:: ---- Step 4: Desktop shortcut ----
echo   Step 4 of 4: Creating desktop shortcut
echo   -----------------------------------
set "SHORTCUT=%USERPROFILE%\Desktop\PlenumNET.lnk"
(
    echo Set oWS = WScript.CreateObject("WScript.Shell"^)
    echo Set oLink = oWS.CreateShortcut("%SHORTCUT%"^)
    echo oLink.TargetPath = "%INSTALLDIR%"
    echo oLink.Description = "PlenumNET / Salvi Framework v%VERSION%"
    echo oLink.Save
) > "%TEMP%\create_shortcut.vbs"
cscript //nologo "%TEMP%\create_shortcut.vbs" >nul 2>nul
if exist "%SHORTCUT%" (
    echo     Desktop shortcut created: PlenumNET.lnk
) else (
    echo     Could not create desktop shortcut (non-critical).
)
del "%TEMP%\create_shortcut.vbs" >nul 2>nul
echo.

:: ---- Done ----
echo   ========================================================
echo     PlenumNET Installation Complete
echo   ========================================================
echo.
echo   Installed to:  %INSTALLDIR%
echo   Version:       v%VERSION%
echo   Documentation: https://plenumnet.replit.app/docs
echo   GitHub:        %REPO%
echo.
echo   What's inside:
echo     %INSTALLDIR%\src\kernel\       Ternary kernel + crypto (Rust)
echo     %INSTALLDIR%\ternary-math\     Math library
echo     %INSTALLDIR%\shared\           TypeScript shared modules
echo     %INSTALLDIR%\services\         TDNS, Inter-Cube services
echo.
echo   Next steps:
echo     cd %INSTALLDIR%
echo     cargo test          (Run 2,276 tests)
echo     cargo bench         (Run benchmarks)
echo.
set /p OPENFOLDER="  Open PlenumNET folder in File Explorer? (Y/N): "
if /i "%OPENFOLDER%"=="Y" start explorer.exe "%INSTALLDIR%"

echo.
pause
