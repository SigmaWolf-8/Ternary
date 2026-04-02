@echo off
title PlenumNET NinjaExec Deployer
echo ==========================================================
echo   PlenumNET NinjaExec Deployer
echo   Capomastro Holdings Ltd.
echo ==========================================================
echo.

net session >nul 2>&1
if %errorlevel% neq 0 (
    echo   This script needs Administrator privileges.
    echo   Relaunching as Administrator...
    echo.
    powershell -Command "Start-Process cmd.exe -ArgumentList '/k \"%~f0\"' -Verb RunAs"
    exit /b 0
)

echo   [OK] Running as Administrator
echo.

cd /d C:\PlenumNET
if %errorlevel% neq 0 (
    echo   [FAIL] C:\PlenumNET not found. Run the full deployer first:
    echo          irm https://plenumnet.replit.app/api/deploy-yoda ^| iex
    pause
    exit /b 1
)

echo   Pulling latest code from GitHub...
git fetch origin main
git checkout origin/main -- services/inter-cube/deploy-yoda.ps1
echo   [OK] deploy-yoda.ps1 updated
echo.

echo   Launching deployer v0.6.0 (includes NinjaExec)...
echo   ──────────────────────────────────────────────────
echo.
powershell -NoProfile -ExecutionPolicy Bypass -File "services\inter-cube\deploy-yoda.ps1" -Force

echo.
echo ==========================================================
echo   Deployer finished. Review the output above.
echo ==========================================================
pause
