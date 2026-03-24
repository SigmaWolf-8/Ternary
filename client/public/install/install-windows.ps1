#Requires -Version 5.1
<#
.SYNOPSIS
    PlenumNET / Salvi Framework Installer for Windows
.DESCRIPTION
    Downloads and installs PlenumNET v2.3.2 to C:\PlenumNET.
    Right-click this file and select "Run with PowerShell".
.NOTES
    Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
    Patent(s) Pending - All Rights Reserved
#>

$ErrorActionPreference = "Stop"
$Version = "2.3.2"
$RepoUrl = "https://github.com/SigmaWolf-8/Ternary"
$InstallDir = "C:\PlenumNET"

function Write-Banner {
    Write-Host ""
    Write-Host "  ========================================================" -ForegroundColor Cyan
    Write-Host "    PlenumNET Installer v$Version"                           -ForegroundColor Cyan
    Write-Host "    Salvi Framework - Post-Quantum Internet Infrastructure"  -ForegroundColor DarkCyan
    Write-Host "    Capomastro Holdings Ltd."                                -ForegroundColor DarkGray
    Write-Host "  ========================================================" -ForegroundColor Cyan
    Write-Host ""
    Write-Host "  Install location: $InstallDir" -ForegroundColor White
    Write-Host ""
}

function Test-Prerequisite {
    param([string]$Command, [string]$Name, [string]$InstallUrl)
    $found = Get-Command $Command -ErrorAction SilentlyContinue
    if ($found) {
        $ver = & $Command --version 2>&1 | Select-Object -First 1
        Write-Host "    [OK] $Name : $ver" -ForegroundColor Green
        return $true
    } else {
        Write-Host "    [--] $Name : not installed" -ForegroundColor Yellow
        Write-Host "         Get it from: $InstallUrl" -ForegroundColor DarkYellow
        return $false
    }
}

function Get-ServiceAccountName {
    $identity = [Security.Principal.WindowsIdentity]::GetCurrent()
    return $identity.Name
}

function Grant-LogonAsService {
    param([string]$AccountName)
    $tempDir = Join-Path $env:TEMP "plenumnet-secedit"
    if (-not (Test-Path $tempDir)) { New-Item -ItemType Directory -Path $tempDir -Force | Out-Null }
    $exportFile = Join-Path $tempDir "secpol-export.inf"
    $importFile = Join-Path $tempDir "secpol-import.inf"
    $seceditDb = Join-Path $tempDir "secedit.sdb"
    try {
        & secedit /export /cfg $exportFile /areas USER_RIGHTS 2>&1 | Out-Null
        if (-not (Test-Path $exportFile)) { return $false }
        $content = Get-Content $exportFile -Raw
        $sidObj = (New-Object System.Security.Principal.NTAccount($AccountName)).Translate(
            [System.Security.Principal.SecurityIdentifier])
        $sid = $sidObj.Value
        if ($content -match '(?m)^SeServiceLogonRight\s*=\s*(.*)$') {
            $existing = $Matches[1]
            if ($existing -notmatch [regex]::Escape($sid)) {
                $content = $content -replace "(?m)^(SeServiceLogonRight\s*=\s*.*)$", "`$1,*$sid"
            }
        } else {
            $content = $content -replace '(?m)(\[Privilege Rights\])', "`$1`r`nSeServiceLogonRight = *$sid"
        }
        Set-Content -Path $importFile -Value $content -Encoding Unicode
        & secedit /configure /db $seceditDb /cfg $importFile /areas USER_RIGHTS 2>&1 | Out-Null
        return $true
    } catch {
        return $false
    } finally {
        Remove-Item $tempDir -Recurse -Force -ErrorAction SilentlyContinue
    }
}

function Set-ServiceLogonAccount {
    param([string]$ServiceName, [string]$AccountName)
    Grant-LogonAsService -AccountName $AccountName | Out-Null
    & sc.exe config $ServiceName obj= $AccountName | Out-Null
}

function Install-PlenumNET {
    Write-Banner

    Write-Host "  Step 1 of 5: Checking prerequisites" -ForegroundColor White
    Write-Host "  -----------------------------------" -ForegroundColor DarkGray
    $hasGit = Test-Prerequisite "git" "Git" "https://git-scm.com/download/win"
    $hasCargo = Test-Prerequisite "cargo" "Rust/Cargo" "https://rustup.rs"
    Write-Host ""

    if (-not $hasGit) {
        Write-Host "  ERROR: Git is required but not installed." -ForegroundColor Red
        Write-Host ""
        Write-Host "  Please install Git for Windows first:" -ForegroundColor White
        Write-Host "  https://git-scm.com/download/win" -ForegroundColor Cyan
        Write-Host ""
        Write-Host "  After installing Git, run this installer again." -ForegroundColor White
        Write-Host ""
        $response = Read-Host "  Open Git download page now? (Y/N)"
        if ($response -match "^[Yy]") {
            Start-Process "https://git-scm.com/download/win"
        }
        Read-Host "  Press Enter to close"
        return
    }

    Write-Host "  Step 2 of 5: Downloading PlenumNET" -ForegroundColor White
    Write-Host "  -----------------------------------" -ForegroundColor DarkGray

    if (Test-Path $InstallDir) {
        Write-Host "    Found existing installation at $InstallDir" -ForegroundColor Yellow
        Write-Host "    Updating to latest version..." -ForegroundColor White
        Push-Location $InstallDir
        try {
            & git pull origin main 2>&1 | ForEach-Object { Write-Host "    $_" -ForegroundColor DarkGray }
        } catch {
            Write-Host "    Update failed: $_" -ForegroundColor Red
        }
        Pop-Location
    } else {
        Write-Host "    Cloning PlenumNET repository to $InstallDir ..." -ForegroundColor White
        & git clone $RepoUrl $InstallDir 2>&1 | ForEach-Object { Write-Host "    $_" -ForegroundColor DarkGray }
        if ($LASTEXITCODE -ne 0) {
            Write-Host ""
            Write-Host "  ERROR: Download failed." -ForegroundColor Red
            Write-Host "  Check your internet connection and try again." -ForegroundColor White
            Read-Host "  Press Enter to close"
            return
        }
    }

    Write-Host ""
    Write-Host "  Step 3 of 5: Building framework" -ForegroundColor White
    Write-Host "  -----------------------------------" -ForegroundColor DarkGray

    if ($hasCargo) {
        Write-Host "    Building all modules (this takes a few minutes)..." -ForegroundColor White
        Write-Host ""
        Push-Location $InstallDir
        & cargo build --release 2>&1 | ForEach-Object {
            if ($_ -match "Compiling|Finished|Downloaded|error\[") {
                Write-Host "    $_" -ForegroundColor DarkGray
            }
        }
        $buildResult = $LASTEXITCODE
        Pop-Location

        if ($buildResult -eq 0) {
            Write-Host ""
            Write-Host "    Build successful!" -ForegroundColor Green
        } else {
            Write-Host ""
            Write-Host "    Build had errors (source code is still available)." -ForegroundColor Yellow
            Write-Host "    You can retry: cd $InstallDir; cargo build --release" -ForegroundColor DarkGray
        }
    } else {
        Write-Host "    Skipping build (Rust not installed)." -ForegroundColor Yellow
        Write-Host ""
        Write-Host "    To build later:" -ForegroundColor White
        Write-Host "      1. Install Rust from https://rustup.rs" -ForegroundColor DarkGray
        Write-Host "      2. Open a new PowerShell window" -ForegroundColor DarkGray
        Write-Host "      3. Run:  cd C:\PlenumNET; cargo build --release" -ForegroundColor DarkGray
    }

    Write-Host ""
    Write-Host "  Step 4 of 5: Creating desktop shortcut" -ForegroundColor White
    Write-Host "  -----------------------------------" -ForegroundColor DarkGray

    try {
        $desktopPath = [Environment]::GetFolderPath("Desktop")
        $shortcutPath = Join-Path $desktopPath "PlenumNET.lnk"
        $shell = New-Object -ComObject WScript.Shell
        $shortcut = $shell.CreateShortcut($shortcutPath)
        $shortcut.TargetPath = $InstallDir
        $shortcut.Description = "PlenumNET / Salvi Framework v$Version"
        $shortcut.Save()
        Write-Host "    Desktop shortcut created: PlenumNET.lnk" -ForegroundColor Green
    } catch {
        Write-Host "    Could not create desktop shortcut (non-critical)." -ForegroundColor DarkGray
    }

    Write-Host ""
    Write-Host "  Step 5 of 5: Daemon identity & service registration" -ForegroundColor White
    Write-Host "  -----------------------------------" -ForegroundColor DarkGray

    $IdentityBase = Join-Path $env:USERPROFILE ".plenumnet"
    $DaemonExe = Join-Path $InstallDir "target\release\inter-cube-daemon.exe"
    $ServiceInstalled = $false
    $NextId = 0

    if (Test-Path $DaemonExe) {
        if (-not (Test-Path $IdentityBase)) {
            New-Item -ItemType Directory -Path $IdentityBase -Force | Out-Null
        }

        $existingIds = @()
        Get-ChildItem -Path $IdentityBase -Directory -Filter "identity-*" -ErrorAction SilentlyContinue | ForEach-Object {
            $num = $_.Name -replace 'identity-', ''
            if ($num -match '^\d+$') { $existingIds += [int]$num }
        }
        if ($existingIds.Count -gt 0) {
            $NextId = ($existingIds | Measure-Object -Maximum).Maximum + 1
        } else {
            $NextId = 1
        }

        $agentDir = Join-Path $IdentityBase "identity-$NextId"
        if (-not (Test-Path $agentDir)) {
            New-Item -ItemType Directory -Path $agentDir -Force | Out-Null
        }
        $keyFile = Join-Path $agentDir "master.key"
        if (-not (Test-Path $keyFile)) {
            Write-Host "    Generating identity #$NextId..." -ForegroundColor White
            $env:CUBE_MODE = "keygen"
            $env:CUBE_IDENTITY_DIR = $agentDir
            $null = & $DaemonExe 2>&1
            Remove-Item Env:\CUBE_MODE -ErrorAction SilentlyContinue
            Remove-Item Env:\CUBE_IDENTITY_DIR -ErrorAction SilentlyContinue
            if (Test-Path $keyFile) {
                Write-Host "    Daemon #$NextId identity created." -ForegroundColor Green
            } else {
                Write-Host "    WARNING: Identity #$NextId key generation may have failed." -ForegroundColor Yellow
            }
        } else {
            Write-Host "    Daemon #$NextId identity already exists." -ForegroundColor Green
        }

        $enginePort = 8080 + (($NextId - 1) * 2)
        $daemonPort = $enginePort + 1
        $CRS_URL = "https://plenumnet.replit.app"

        $isAdmin = $false
        try {
            $currentPrincipal = New-Object Security.Principal.WindowsPrincipal([Security.Principal.WindowsIdentity]::GetCurrent())
            $isAdmin = $currentPrincipal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
        } catch {}

        if ($isAdmin) {
            Write-Host "    Registering Windows Service for daemon #$NextId..." -ForegroundColor Cyan

            $LogDir = Join-Path $IdentityBase "logs"
            if (-not (Test-Path $LogDir)) {
                New-Item -ItemType Directory -Path $LogDir -Force | Out-Null
            }
            $logFile = Join-Path $LogDir "cube-${NextId}.log"

            $wrapperDir = Join-Path $InstallDir "services\wrappers"
            if (-not (Test-Path $wrapperDir)) {
                New-Item -ItemType Directory -Path $wrapperDir -Force | Out-Null
            }
            $wrapperBat = Join-Path $wrapperDir "cube-${NextId}-start.bat"
            @"
@echo off
set CUBE_MODE=cube
set CUBE_API_PORT=$daemonPort
set LLM_PORT=$enginePort
set CUBE_CRS_URL=$CRS_URL
set RELAY_URL=$CRS_URL
set CUBE_IDENTITY_DIR=$agentDir
set CUBE_ROLE=inference
"$DaemonExe" >> "$logFile" 2>&1
"@ | Set-Content -Path $wrapperBat -Encoding ASCII

            $svcName = "PlenumNET-Cube-$NextId"
            $existingSvc = Get-Service -Name $svcName -ErrorAction SilentlyContinue
            if ($existingSvc) {
                Stop-Service -Name $svcName -Force -ErrorAction SilentlyContinue
                & sc.exe delete $svcName | Out-Null
                Start-Sleep -Seconds 2
            }

            try {
                $svcBinPath = "cmd.exe /c `"$wrapperBat`""
                New-Service -Name $svcName `
                    -BinaryPathName $svcBinPath `
                    -DisplayName "PlenumNET Inter-Cube Daemon (Identity #$NextId)" `
                    -Description "PlenumNET Inter-Cube infrastructure daemon for identity #$NextId" `
                    -StartupType Automatic | Out-Null

                & sc.exe failure $svcName reset= 86400 actions= restart/5000/restart/10000/restart/30000 | Out-Null
                & sc.exe config $svcName depend= Tcpip/Afd/Dnscache | Out-Null

                $svcAccount = Get-ServiceAccountName
                Set-ServiceLogonAccount -ServiceName $svcName -AccountName $svcAccount

                Start-Service -Name $svcName
                $ServiceInstalled = $true
                Write-Host "    Daemon #$NextId registered and started as Windows Service." -ForegroundColor Green
            } catch {
                Write-Host "    Could not register Windows Service: $_" -ForegroundColor Yellow
            }
        } else {
            Write-Host "    Skipping service registration (not running as Administrator)." -ForegroundColor Yellow
            Write-Host "    To register as a service, re-run as Administrator or use:" -ForegroundColor DarkGray
            Write-Host "    powershell -File `"$InstallDir\client\public\install\plenumnet-service.ps1`" install $NextId" -ForegroundColor DarkGray
        }
    } else {
        Write-Host "    Daemon binary not found. Skipping identity & service setup." -ForegroundColor Yellow
    }

    Write-Host ""
    Write-Host "  ========================================================" -ForegroundColor Cyan
    Write-Host "    PlenumNET Installation Complete" -ForegroundColor Green
    Write-Host "  ========================================================" -ForegroundColor Cyan
    Write-Host ""
    Write-Host "  Installed to:  $InstallDir" -ForegroundColor White
    Write-Host "  Version:       v$Version" -ForegroundColor White
    Write-Host "  Documentation: https://plenumnet.replit.app/docs" -ForegroundColor White
    Write-Host "  GitHub:        $RepoUrl" -ForegroundColor White
    Write-Host ""
    Write-Host "  What's inside:" -ForegroundColor White
    Write-Host "    C:\PlenumNET\src\kernel\     - Ternary kernel + crypto (Rust)" -ForegroundColor DarkGray
    Write-Host "    C:\PlenumNET\ternary-math\   - Math library" -ForegroundColor DarkGray
    Write-Host "    C:\PlenumNET\shared\          - TypeScript shared modules" -ForegroundColor DarkGray
    Write-Host "    C:\PlenumNET\services\        - TDNS, Inter-Cube services" -ForegroundColor DarkGray
    Write-Host ""

    if ($ServiceInstalled -and $NextId -gt 0) {
        Write-Host "  Daemon #$NextId is running as a Windows Service and will auto-start on boot." -ForegroundColor Green
        Write-Host ""
        Write-Host "  Service management:" -ForegroundColor White
        Write-Host "    Get-Service PlenumNET-Cube-$NextId          # Check status" -ForegroundColor DarkGray
        Write-Host "    Get-Content ~\.plenumnet\logs\cube-$NextId.log -Wait  # View logs" -ForegroundColor DarkGray
        Write-Host "    Restart-Service PlenumNET-Cube-$NextId      # Restart" -ForegroundColor DarkGray
        Write-Host ""
        Write-Host "  Or use the service manager:" -ForegroundColor White
        Write-Host "    powershell -File `"$InstallDir\client\public\install\plenumnet-service.ps1`" status" -ForegroundColor DarkGray
        Write-Host "    powershell -File `"$InstallDir\client\public\install\plenumnet-service.ps1`" logs $NextId" -ForegroundColor DarkGray
    } else {
        Write-Host "  Next steps:" -ForegroundColor White
        Write-Host "    cd C:\PlenumNET" -ForegroundColor DarkGray
        Write-Host "    cargo test          # Run 2,276 tests" -ForegroundColor DarkGray
        Write-Host "    cargo bench         # Run benchmarks" -ForegroundColor DarkGray
        if ($NextId -gt 0) {
            $enginePort = 8080 + (($NextId - 1) * 2)
            $daemonPort = $enginePort + 1
            Write-Host "" -ForegroundColor White
            Write-Host "  Manual start daemon #${NextId}:" -ForegroundColor White
            Write-Host "    `$env:CUBE_MODE=`"cube`"; `$env:CUBE_API_PORT=`"$daemonPort`"; `$env:LLM_PORT=`"$enginePort`"" -ForegroundColor DarkGray
            Write-Host "    `$env:CUBE_CRS_URL=`"https://plenumnet.replit.app`"; `$env:CUBE_ROLE=`"inference`"" -ForegroundColor DarkGray
            Write-Host "    `$env:CUBE_IDENTITY_DIR=`"$(Join-Path $IdentityBase "identity-$NextId")`"" -ForegroundColor DarkGray
            Write-Host "    & `"$DaemonExe`"" -ForegroundColor DarkGray
        }
    }

    Write-Host ""
    Write-Host "  Run this installer again to add another daemon." -ForegroundColor DarkGray
    Write-Host ""
    Write-Host "  Open the PlenumNET folder now in File Explorer?" -ForegroundColor White
    $response = Read-Host "  (Y/N)"
    if ($response -match "^[Yy]") {
        Start-Process explorer.exe $InstallDir
    }
}

Install-PlenumNET
