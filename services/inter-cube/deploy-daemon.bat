@echo off
setlocal enabledelayedexpansion
title PlenumNET Cube Daemon Deployer
color 0B
echo.
echo   ==========================================================
echo     PlenumNET Cube Daemon Deployer v0.3.0
echo     Applied Physics Division -- Capomastro Holdings Ltd.
echo   ==========================================================
echo.

REM Ensure cargo is in PATH (rustup default install location)
if exist "%USERPROFILE%\.cargo\bin\cargo.exe" (
    set "PATH=%USERPROFILE%\.cargo\bin;%PATH%"
)

REM Check prerequisites
where git >nul 2>nul
if errorlevel 1 (
    echo   [ERROR] git is not installed or not in PATH.
    echo.
    goto END
)
where cargo >nul 2>nul
if errorlevel 1 (
    echo   [ERROR] cargo [Rust] is not installed or not in PATH.
    echo          Install from https://rustup.rs/
    echo.
    goto END
)

set "REPO_DIR=C:\PlenumNET"
set "REPO_URL=https://github.com/SigmaWolf-8/Ternary.git"
set "BINARY=inter-cube-daemon.exe"
set "BINARY_PATH=%REPO_DIR%\target\release\%BINARY%"

REM Handle three cases: no folder, folder but no git, git repo
if not exist "%REPO_DIR%" (
    echo   [CLONE] Repository not found -- cloning to %REPO_DIR%...
    git clone "%REPO_URL%" "%REPO_DIR%"
    if errorlevel 1 (
        echo   [ERROR] git clone failed.
        goto END
    )
    goto PULL_DONE
)

if not exist "%REPO_DIR%\.git" (
    echo   [SETUP] %REPO_DIR% exists but is not a git repo.
    echo          Converting to git repo so we can pull updates...
    echo.
    pushd "%REPO_DIR%"
    git init >nul 2>nul
    git remote add origin "%REPO_URL%" 2>nul
    git fetch origin main
    if errorlevel 1 (
        echo   [ERROR] git fetch failed. Check your internet connection.
        popd
        goto END
    )
    git reset --hard origin/main
    if errorlevel 1 (
        echo   [ERROR] git reset failed.
        popd
        goto END
    )
    echo   [SETUP] Converted to git repo and synced to latest.
    popd
    goto PULL_DONE
)

echo   [PULL] Updating source from GitHub...
pushd "%REPO_DIR%"
git fetch origin main >nul 2>nul
git pull origin main --ff-only
if errorlevel 1 (
    echo   [ERROR] git pull failed. Resolve conflicts manually.
    popd
    goto END
)
echo   [PULL] Updated to latest.
popd

:PULL_DONE
echo.

REM Stop running daemons
tasklist /FI "IMAGENAME eq %BINARY%" 2>nul | find /I "%BINARY%" >nul
if not errorlevel 1 (
    echo   [STOP] Stopping running daemon(s)...
    taskkill /F /IM "%BINARY%" >nul 2>nul
    timeout /t 2 /nobreak >nul
    echo   [STOP] Daemon(s) stopped.
    echo.
)

REM Build
pushd "%REPO_DIR%"
echo   [BUILD] Compiling inter-cube daemon (release)...
echo          This may take a few minutes on first build.
echo.
cargo build --release -p inter-cube
if errorlevel 1 (
    echo.
    echo   [ERROR] Build failed. Check output above.
    popd
    goto END
)

if not exist "%BINARY_PATH%" (
    echo   [ERROR] Binary not found at %BINARY_PATH% after build.
    popd
    goto END
)

echo.
echo   ==========================================================
echo     BUILD SUCCESSFUL
echo   ==========================================================
echo   Binary: %BINARY_PATH%
for /f %%H in ('git rev-parse --short HEAD 2^>nul') do echo   Commit: %%H
popd
echo.

REM Generate identities for A, B, C
echo   [IDENTITY] Checking daemon identities...
echo.
set "IDENTITY_BASE=%USERPROFILE%\.plenumnet"

for %%A in (a b c) do (
    if not exist "%IDENTITY_BASE%\identity-%%A" (
        mkdir "%IDENTITY_BASE%\identity-%%A" >nul 2>nul
    )
    if not exist "%IDENTITY_BASE%\identity-%%A\master.key" (
        echo   [IDENTITY] Generating identity for Agent %%A...
        set "CUBE_MODE=keygen"
        set "CUBE_IDENTITY_DIR=%IDENTITY_BASE%\identity-%%A"
        "%BINARY_PATH%" >nul 2>nul
        set "CUBE_MODE="
        set "CUBE_IDENTITY_DIR="
        if exist "%IDENTITY_BASE%\identity-%%A\master.key" (
            echo   [IDENTITY] Agent %%A identity created.
        ) else (
            echo   [IDENTITY] WARNING: Agent %%A key generation may have failed.
        )
    ) else (
        echo   [IDENTITY] Agent %%A identity exists.
    )
)

echo.
echo   ==========================================================
echo     READY TO LAUNCH
echo   ==========================================================
echo.
echo   Start Daemon A:
echo     set CUBE_MODE=cube
echo     set CUBE_API_PORT=8081
echo     set LLM_PORT=8080
echo     set CUBE_CRS_URL=https://plenumnet.replit.app
echo     set CUBE_ROLE=inference
echo     set CUBE_IDENTITY_DIR=%%USERPROFILE%%\.plenumnet\identity-a
echo     "%BINARY_PATH%"
echo.
echo   Start Daemon B:
echo     set CUBE_MODE=cube
echo     set CUBE_API_PORT=8083
echo     set LLM_PORT=8082
echo     set CUBE_CRS_URL=https://plenumnet.replit.app
echo     set CUBE_ROLE=inference
echo     set CUBE_IDENTITY_DIR=%%USERPROFILE%%\.plenumnet\identity-b
echo     "%BINARY_PATH%"
echo.
echo   Start Daemon C:
echo     set CUBE_MODE=cube
echo     set CUBE_API_PORT=8085
echo     set LLM_PORT=8084
echo     set CUBE_CRS_URL=https://plenumnet.replit.app
echo     set CUBE_ROLE=inference
echo     set CUBE_IDENTITY_DIR=%%USERPROFILE%%\.plenumnet\identity-c
echo     "%BINARY_PATH%"
echo.
:END
echo.
echo   Press any key to close...
pause >nul
