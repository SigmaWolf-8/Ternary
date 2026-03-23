@echo off
title PlenumNET Installer
color 0B
echo.
echo   ========================================================
echo     PlenumNET Installer v2.4.0
echo     Salvi Framework - Post-Quantum Internet Infrastructure
echo     Capomastro Holdings Ltd.
echo   ========================================================
echo.
echo   This will install PlenumNET to C:\PlenumNET
echo.
echo   Press any key to begin...
pause >nul
echo.

set "INSTALL_DIR=C:\PlenumNET"
set "IDENTITY_BASE=%USERPROFILE%\.plenumnet"
set "REPO_URL=https://github.com/SigmaWolf-8/Ternary.git"
set "CARGO_BUILD_JOBS=1"

git --version >nul 2>nul
if errorlevel 1 goto NOGIT

if exist "%INSTALL_DIR%\.git" (
    echo   Existing repo found. Pulling latest...
    pushd "%INSTALL_DIR%"
    git pull --ff-only origin main
    popd
    echo   Updated to latest.
    echo.
    goto BUILD
)

if exist "%INSTALL_DIR%" (
    echo   Removing previous non-git installation...
    rmdir /s /q "%INSTALL_DIR%" >nul 2>nul
)

echo   Cloning PlenumNET repository...
echo   (This may take a few minutes)
echo.
git clone --depth 1 "%REPO_URL%" "%INSTALL_DIR%"
if errorlevel 1 goto FAILDOWNLOAD
echo   Clone complete.
echo.
goto BUILD

:NOGIT
echo   Git not found. Downloading ZIP archive...
echo   (This may take a minute)
echo.
if exist "%INSTALL_DIR%" (
    echo   Removing previous installation...
    rmdir /s /q "%INSTALL_DIR%" >nul 2>nul
)
mkdir "%INSTALL_DIR%" >nul 2>nul
curl -L -o "%TEMP%\PlenumNET.zip" "https://github.com/SigmaWolf-8/Ternary/archive/refs/heads/main.zip" >nul 2>nul
if errorlevel 1 goto FAILDOWNLOAD
echo   Download complete. Extracting...
echo.
powershell -Command "Expand-Archive -Path '%TEMP%\PlenumNET.zip' -DestinationPath '%TEMP%\PlenumNET-extract' -Force" >nul 2>nul
if errorlevel 1 goto FAILEXTRACT
xcopy "%TEMP%\PlenumNET-extract\Ternary-main\*" "%INSTALL_DIR%\" /s /e /q /y >nul 2>nul
rmdir /s /q "%TEMP%\PlenumNET-extract" >nul 2>nul
del "%TEMP%\PlenumNET.zip" >nul 2>nul
echo   Files extracted to %INSTALL_DIR%
echo.

:BUILD
echo   Checking for Rust compiler...
cargo --version >nul 2>nul
if errorlevel 1 goto NORUST
echo   Rust found. Building inter-cube daemon...
echo   (This may take several minutes on first build)
echo.
pushd "%INSTALL_DIR%"
cargo build --release -p inter-cube
popd
if errorlevel 1 (
    echo   WARNING: Build failed. You can retry later with:
    echo     cd %INSTALL_DIR%
    echo     set CARGO_BUILD_JOBS=1
    echo     cargo build --release -p inter-cube
    echo.
)
goto IDENTITY

:NORUST
echo   Rust is not installed (optional).
echo   To build later install Rust from https://rustup.rs
echo   Then run: cd %INSTALL_DIR%
echo             set CARGO_BUILD_JOBS=1
echo             cargo build --release -p inter-cube
echo.
goto DONE

:IDENTITY
set "DAEMON_EXE=%INSTALL_DIR%\target\release\inter-cube-daemon.exe"
if not exist "%DAEMON_EXE%" goto DONE
echo   Generating first daemon identity...
echo.
set "NEXT_ID=1"
:FIND_NEXT_ID
if exist "%IDENTITY_BASE%\identity-%NEXT_ID%\master.key" (
    set /a NEXT_ID+=1
    goto FIND_NEXT_ID
)
set "ID_DIR=%IDENTITY_BASE%\identity-%NEXT_ID%"
if not exist "%ID_DIR%" mkdir "%ID_DIR%" >nul 2>nul
echo   Generating identity #%NEXT_ID%...
set "CUBE_MODE=keygen"
set "CUBE_IDENTITY_DIR=%ID_DIR%"
"%DAEMON_EXE%" >nul 2>nul
set "CUBE_MODE="
set "CUBE_IDENTITY_DIR="
if exist "%ID_DIR%\master.key" (
    echo   Daemon #%NEXT_ID% identity created.
) else (
    echo   WARNING: Identity #%NEXT_ID% key generation may have failed.
)
set /a ENGINE_PORT=8080 + (%NEXT_ID% - 1) * 2
set /a DAEMON_PORT=%ENGINE_PORT% + 1
echo.
echo   To start daemon #%NEXT_ID%:
echo     set CUBE_MODE=cube
echo     set CUBE_API_PORT=%DAEMON_PORT%
echo     set LLM_PORT=%ENGINE_PORT%
echo     set CUBE_CRS_URL=https://plenumnet.replit.app
echo     set CUBE_IDENTITY_DIR=%ID_DIR%
echo     "%DAEMON_EXE%"
echo.
echo   Run the deployer again to add another daemon:
echo     %INSTALL_DIR%\services\inter-cube\deploy-daemon.bat
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
echo.
echo   ========================================================
echo     Installation Complete
echo   ========================================================
echo.
echo   PlenumNET is at: %INSTALL_DIR%
echo.
echo   Opening folder...
start explorer.exe "%INSTALL_DIR%"
echo.
:END
echo   Press any key to close this window...
pause >nul
