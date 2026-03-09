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
echo   Downloading PlenumNET...
echo   (This may take a minute)
echo.
if exist "C:\PlenumNET" (
    echo   Removing previous installation...
    rmdir /s /q "C:\PlenumNET" >nul 2>nul
)
mkdir "C:\PlenumNET" >nul 2>nul
curl -L -o "%TEMP%\PlenumNET.zip" "https://github.com/SigmaWolf-8/Ternary/archive/refs/heads/main.zip" >nul 2>nul
if errorlevel 1 goto FAILDOWNLOAD
echo   Download complete. Extracting...
echo.
powershell -Command "Expand-Archive -Path '%TEMP%\PlenumNET.zip' -DestinationPath '%TEMP%\PlenumNET-extract' -Force" >nul 2>nul
if errorlevel 1 goto FAILEXTRACT
xcopy "%TEMP%\PlenumNET-extract\Ternary-main\*" "C:\PlenumNET\" /s /e /q /y >nul 2>nul
rmdir /s /q "%TEMP%\PlenumNET-extract" >nul 2>nul
del "%TEMP%\PlenumNET.zip" >nul 2>nul
echo   Files extracted to C:\PlenumNET
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
:FAILDOWNLOAD
echo.
echo   ERROR: Download failed.
echo   Check your internet connection and try again.
echo.
goto END
:FAILEXTRACT
echo.
echo   ERROR: Could not extract files.
echo   Try running as Administrator.
echo.
del "%TEMP%\PlenumNET.zip" >nul 2>nul
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
