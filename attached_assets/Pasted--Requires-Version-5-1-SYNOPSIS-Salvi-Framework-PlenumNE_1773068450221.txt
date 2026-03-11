#Requires -Version 5.1
<#
.SYNOPSIS
    Salvi Framework / PlenumNET Installer for Windows
.DESCRIPTION
    Downloads and installs the Salvi Framework v3.0.0.
    Run this script by right-clicking and selecting "Run with PowerShell"
    or paste the one-liner from the distribution page into PowerShell.
.NOTES
    Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
    Patent(s) Pending - All Rights Reserved
#>

$ErrorActionPreference = "Stop"
$Version = "3.0.0"
$RepoUrl = "https://github.com/SigmaWolf-8/Ternary"
$ArchiveUrl = "$RepoUrl/releases/download/v$Version/salvi-framework-v$Version.tar.gz"
$InstallDir = "$env:USERPROFILE\SalviFramework"
$TempDir = "$env:TEMP\salvi-install"
$TempArchive = "$TempDir\salvi-framework.tar.gz"

function Write-Banner {
    Write-Host ""
    Write-Host "  =========================================" -ForegroundColor Cyan
    Write-Host "    Salvi Framework Installer v$Version"     -ForegroundColor Cyan
    Write-Host "    PlenumNET - Post-Quantum Infrastructure" -ForegroundColor DarkCyan
    Write-Host "  =========================================" -ForegroundColor Cyan
    Write-Host ""
    Write-Host "  Copyright 2025-2026 Capomastro Holdings Ltd." -ForegroundColor DarkGray
    Write-Host ""
}

function Test-Prerequisite {
    param([string]$Command, [string]$Name, [string]$InstallUrl)
    $found = Get-Command $Command -ErrorAction SilentlyContinue
    if ($found) {
        $ver = & $Command --version 2>&1 | Select-Object -First 1
        Write-Host "  [OK] $Name found: $ver" -ForegroundColor Green
        return $true
    } else {
        Write-Host "  [--] $Name not found" -ForegroundColor Yellow
        Write-Host "       Install from: $InstallUrl" -ForegroundColor DarkYellow
        return $false
    }
}

function Install-Framework {
    Write-Banner

    Write-Host "  Checking prerequisites..." -ForegroundColor White
    Write-Host ""

    $hasGit = Test-Prerequisite "git" "Git" "https://git-scm.com/download/win"
    $hasCargo = Test-Prerequisite "cargo" "Rust/Cargo" "https://rustup.rs"
    Write-Host ""

    if (-not $hasGit) {
        Write-Host "  Git is required. Install it from https://git-scm.com/download/win" -ForegroundColor Red
        Write-Host "  Then re-run this installer." -ForegroundColor Red
        Write-Host ""
        Read-Host "  Press Enter to open the Git download page"
        Start-Process "https://git-scm.com/download/win"
        return
    }

    Write-Host "  Install location: $InstallDir" -ForegroundColor White
    Write-Host ""

    if (Test-Path $InstallDir) {
        Write-Host "  Directory already exists. Updating..." -ForegroundColor Yellow
        Set-Location $InstallDir
        Write-Host "  Pulling latest changes..." -ForegroundColor White
        & git pull origin main 2>&1 | ForEach-Object { Write-Host "    $_" -ForegroundColor DarkGray }
    } else {
        Write-Host "  Step 1/3: Cloning repository..." -ForegroundColor White
        & git clone $RepoUrl $InstallDir 2>&1 | ForEach-Object { Write-Host "    $_" -ForegroundColor DarkGray }
        if ($LASTEXITCODE -ne 0) {
            Write-Host "  Clone failed. Check your internet connection." -ForegroundColor Red
            return
        }
        Set-Location $InstallDir
    }

    Write-Host ""
    Write-Host "  Step 2/3: Checking Rust toolchain..." -ForegroundColor White

    if ($hasCargo) {
        Write-Host "  Step 3/3: Building framework (this may take a few minutes)..." -ForegroundColor White
        Write-Host ""
        & cargo build --release 2>&1 | ForEach-Object {
            if ($_ -match "Compiling|Finished|Downloaded") {
                Write-Host "    $_" -ForegroundColor DarkGray
            }
        }
        if ($LASTEXITCODE -eq 0) {
            Write-Host ""
            Write-Host "  Build successful!" -ForegroundColor Green
        } else {
            Write-Host ""
            Write-Host "  Build had errors. The source code is still available at:" -ForegroundColor Yellow
            Write-Host "  $InstallDir" -ForegroundColor White
        }
    } else {
        Write-Host ""
        Write-Host "  Rust is not installed. The source code has been downloaded to:" -ForegroundColor Yellow
        Write-Host "  $InstallDir" -ForegroundColor White
        Write-Host ""
        Write-Host "  To build later:" -ForegroundColor White
        Write-Host "    1. Install Rust from https://rustup.rs" -ForegroundColor DarkGray
        Write-Host "    2. Open a new terminal" -ForegroundColor DarkGray
        Write-Host "    3. Run: cd $InstallDir; cargo build --release" -ForegroundColor DarkGray
    }

    Write-Host ""
    Write-Host "  =========================================" -ForegroundColor Cyan
    Write-Host "    Installation Complete" -ForegroundColor Green
    Write-Host "  =========================================" -ForegroundColor Cyan
    Write-Host ""
    Write-Host "  Location:    $InstallDir" -ForegroundColor White
    Write-Host "  Version:     v$Version" -ForegroundColor White
    Write-Host "  Docs:        https://plenumnet.replit.app/docs" -ForegroundColor White
    Write-Host "  GitHub:      $RepoUrl" -ForegroundColor White
    Write-Host ""
    Write-Host "  Quick start:" -ForegroundColor White
    Write-Host "    cd $InstallDir" -ForegroundColor DarkGray
    Write-Host "    cargo test        # Run 2,276 tests" -ForegroundColor DarkGray
    Write-Host "    cargo bench       # Run benchmarks" -ForegroundColor DarkGray
    Write-Host ""

    Read-Host "  Press Enter to close"
}

Install-Framework
