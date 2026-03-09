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
echo   Press any key to begin...
pause >nul
echo.
echo   Checking for Git...
git --version >nul 2>nul
if errorlevel 1 goto NOGIT
echo   Git found.
echo.
echo   Downloading PlenumNET to C:\PlenumNET ...
echo   (This may take a minute)
echo.
if exist "C:\PlenumNET\.git" goto UPDATE
if exist "C:\PlenumNET" rmdir /s /q "C:\PlenumNET"
git clone https://github.com/SigmaWolf-8/Ternary.git "C:\PlenumNET"
if errorlevel 1 goto FAILCLONE
goto CLONED
:UPDATE
pushd "C:\PlenumNET"
git pull origin main
popd
:CLONED
echo.
echo   Download complete.
echo.
echo   Checking for Rust compiler...
cargo --version >nul 2>nul
if errorlevel 1 goto NORUST
echo   Rust found. Building framework...
echo   (This may take several minutes)
echo.
pushd "C:\PlenumNET"
cargo build --release
popd
echo.
goto DONE
:NORUST
echo   Rust is not installed (optional).
echo   To build later install Rust from https://rustup.rs
echo   Then run: cd C:\PlenumNET
echo             cargo build --release
echo.
goto DONE
:NOGIT
echo.
echo   ERROR: Git is not installed.
echo   Opening download page...
start https://git-scm.com/download/win
echo   Install Git then run this installer again.
echo.
goto END
:FAILCLONE
echo.
echo   ERROR: Download failed.
echo   Check your internet connection and try again.
echo.
goto END
:DONE
echo   ========================================================
echo     Installation Complete
echo   ========================================================
echo.
echo   PlenumNET is at: C:\PlenumNET
echo.
echo   Opening folder...
start explorer.exe "C:\PlenumNET"
echo.
:END
echo   Press any key to close this window...
pause >nul
