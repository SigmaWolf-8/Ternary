@echo off
title PlenumNET Cube Daemon Deployer
color 0B
echo.
echo   ==========================================================
echo     PlenumNET Cube Daemon Deployer v0.4.0
echo     Applied Physics Division -- Capomastro Holdings Ltd.
echo   ==========================================================
echo.
echo   Launching deployer...
echo.

powershell.exe -NoProfile -ExecutionPolicy Bypass -File "%~dp0deploy-daemon.ps1"

echo.
echo   Press any key to close...
pause >nul
