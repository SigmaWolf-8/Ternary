@echo off
title PlenumNET Installer
color 0B

echo.
echo   ========================================================
echo     PlenumNET Installer v2.3.2
echo     Salvi Framework - Post-Quantum Internet Infrastructure
echo     Capomastro Holdings Ltd.
echo   ========================================================
echo.
echo   This will install PlenumNET to C:\PlenumNET
echo.
pause

echo.
echo   Checking for Git...
where git >nul 2>nul
if errorlevel 1 (
    echo.
    echo   Git is not installed. You need Git to continue.
    echo   Opening download page: https://git-scm.com/download/win
    echo.
    start https://git-scm.com/download/win
    echo   Install Git, then run this installer again.
    echo.
    pause
    exit /b
)
echo   Git found.

echo.
echo   Downloading PlenumNET to C:\PlenumNET ...
echo   (This may take a minute)
echo.

if exist "C:\PlenumNET\.git" (
    echo   Found existing install. Updating...
    pushd "C:\PlenumNET"
    git pull origin main
    popd
) else (
    if exist "C:\PlenumNET" rmdir /s /q "C:\PlenumNET"
    git clone https://github.com/SigmaWolf-8/Ternary.git "C:\PlenumNET"
)

if errorlevel 1 (
    echo.
    echo   Download failed. Check your internet connection.
    pause
    exit /b
)

echo.
echo   Download complete.
echo.

echo   Checking for Rust...
where cargo >nul 2>nul
if errorlevel 1 (
    echo   Rust is not installed. Skipping build.
    echo   To build later, install Rust from https://rustup.rs
    echo   Then open Command Prompt and run:
    echo     cd C:\PlenumNET
    echo     cargo build --release
    goto :DONE
)

echo   Rust found. Building framework...
echo   (This may take several minutes)
echo.
pushd "C:\PlenumNET"
cargo build --release
popd
echo.

:DONE
echo.
echo   ========================================================
echo     Installation Complete
echo   ========================================================
echo.
echo   PlenumNET is installed at: C:\PlenumNET
echo.
echo   Folder contents:
echo     src\kernel\       Ternary kernel and crypto (Rust)
echo     ternary-math\     Math library
echo     shared\           TypeScript shared modules
echo     services\         TDNS, Inter-Cube services
echo.
echo   Opening C:\PlenumNET in File Explorer...
start explorer.exe "C:\PlenumNET"
echo.
pause
