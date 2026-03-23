<#
.SYNOPSIS
    PlenumNET Cube Daemon Deployer
    Pulls latest source from GitHub, builds the daemon, and verifies the binary.
    Run from any directory - handles C:\PlenumNET automatically.

.DESCRIPTION
    This script is served from https://plenumnet.replit.app/api/deploy-daemon
    Run it with:
      irm https://plenumnet.replit.app/api/deploy-daemon | iex

    Each run auto-detects existing daemon identities and creates the next one.
    First run creates identity-1 (ports 8080/8081), second creates identity-2
    (ports 8082/8083), and so on.

.NOTES
    Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
    Applied Physics Division
#>

$RepoDir = "C:\PlenumNET"
$BinaryName = "inter-cube-daemon.exe"
$BinaryPath = Join-Path $RepoDir "target\release\$BinaryName"
$RepoUrl = "https://github.com/SigmaWolf-8/Ternary.git"
$IdentityBase = Join-Path $env:USERPROFILE ".plenumnet"
$BaseEnginePort = 8080
$PortStep = 2

Write-Host ""
Write-Host "==========================================================" -ForegroundColor Cyan
Write-Host "  PlenumNET Cube Daemon Deployer" -ForegroundColor Cyan
Write-Host "  Applied Physics Division -- Capomastro Holdings Ltd." -ForegroundColor Cyan
Write-Host "==========================================================" -ForegroundColor Cyan
Write-Host ""

function Test-Command($cmd) {
    try { Get-Command $cmd -ErrorAction Stop | Out-Null; return $true }
    catch { return $false }
}

if (-not (Test-Command "git")) {
    Write-Host 'ERROR: git is not installed or not in PATH.' -ForegroundColor Red
    return
}
if (-not (Test-Command "cargo")) {
    Write-Host 'ERROR: cargo (Rust) is not installed or not in PATH.' -ForegroundColor Red
    Write-Host '       Install from https://rustup.rs/' -ForegroundColor Yellow
    return
}

if (-not (Test-Path $RepoDir)) {
    Write-Host 'CLONE: Repository not found -- cloning...' -ForegroundColor Yellow
    $null = & git clone $RepoUrl $RepoDir 2>&1
    if ($LASTEXITCODE -ne 0) {
        Write-Host 'ERROR: git clone failed.' -ForegroundColor Red
        return
    }
} elseif (-not (Test-Path (Join-Path $RepoDir ".git"))) {
    Write-Host 'SETUP: Directory exists but is not a git repo (installed via ZIP?).' -ForegroundColor Yellow
    Write-Host '       Converting to a git repo so we can pull updates...' -ForegroundColor Yellow
    Push-Location $RepoDir
    $null = & git init 2>&1
    $null = & git remote add origin $RepoUrl 2>&1
    $null = & git fetch origin main 2>&1
    if ($LASTEXITCODE -ne 0) {
        Write-Host 'ERROR: git fetch failed. Check your internet connection.' -ForegroundColor Red
        Pop-Location
        return
    }
    $null = & git reset --hard origin/main 2>&1
    if ($LASTEXITCODE -ne 0) {
        Write-Host 'ERROR: git reset failed.' -ForegroundColor Red
        Pop-Location
        return
    }
    Write-Host 'SETUP: Converted to git repo and synced to latest.' -ForegroundColor Green
    Pop-Location
}

Write-Host 'PULL: Updating source from GitHub...' -ForegroundColor Yellow
Push-Location $RepoDir
try {
    $null = & git fetch origin main 2>&1
    $localHash = (& git rev-parse HEAD 2>&1) | Out-String
    $localHash = $localHash.Trim()
    $remoteHash = (& git rev-parse origin/main 2>&1) | Out-String
    $remoteHash = $remoteHash.Trim()
    if ($localHash -eq $remoteHash) {
        $shortH = $localHash.Substring(0,8)
        Write-Host "PULL: Already up to date ($shortH)" -ForegroundColor Green
    } else {
        $null = & git pull origin main --ff-only 2>&1
        if ($LASTEXITCODE -ne 0) {
            Write-Host 'ERROR: git pull failed. Resolve conflicts manually.' -ForegroundColor Red
            return
        }
        $shortHash = (& git rev-parse --short HEAD 2>&1) | Out-String
        Write-Host "PULL: Updated to $($shortHash.Trim())" -ForegroundColor Green
    }

    $runningDaemons = Get-Process -Name "inter-cube-daemon" -ErrorAction SilentlyContinue
    if ($runningDaemons) {
        Write-Host 'STOP: Stopping running daemon(s)...' -ForegroundColor Yellow
        $runningDaemons | Stop-Process -Force
        Start-Sleep -Seconds 2
        Write-Host 'STOP: Daemon(s) stopped.' -ForegroundColor Green
    }

    Write-Host 'BUILD: Compiling inter-cube daemon (release)...' -ForegroundColor Yellow
    Write-Host '       This may take a few minutes on first build.' -ForegroundColor DarkGray
    $env:CARGO_BUILD_JOBS = "1"
    & cargo build --release -p inter-cube 2>&1 | ForEach-Object {
        $line = $_.ToString()
        if ($line -match "error") {
            Write-Host "       $line" -ForegroundColor Red
        } elseif ($line -match "warning") {
            Write-Host "       $line" -ForegroundColor Yellow
        } elseif ($line -match "Compiling|Finished|Downloading|Downloaded") {
            Write-Host "       $line" -ForegroundColor DarkGray
        }
    }
    if ($LASTEXITCODE -ne 0) {
        Write-Host 'ERROR: Build failed. See error messages above.' -ForegroundColor Red
        return
    }

    if (-not (Test-Path $BinaryPath)) {
        Write-Host "ERROR: Binary not found at $BinaryPath after build." -ForegroundColor Red
        return
    }

    $fileInfo = Get-Item $BinaryPath
    $buildTime = $fileInfo.LastWriteTime.ToString("yyyy-MM-dd HH:mm:ss")
    $fileSizeMB = [math]::Round($fileInfo.Length / 1MB, 1)

    Write-Host ""
    Write-Host "==========================================================" -ForegroundColor Green
    Write-Host "  BUILD SUCCESSFUL" -ForegroundColor Green
    Write-Host "==========================================================" -ForegroundColor Green
    Write-Host "  Binary:    $BinaryPath" -ForegroundColor White
    Write-Host "  Size:      $fileSizeMB MB" -ForegroundColor White
    Write-Host "  Built:     $buildTime" -ForegroundColor White
    $commitHash = (& git rev-parse --short HEAD 2>&1) | Out-String
    Write-Host "  Commit:    $($commitHash.Trim())" -ForegroundColor White
    Write-Host ""

    if (-not (Test-Path $IdentityBase)) {
        New-Item -ItemType Directory -Path $IdentityBase -Force | Out-Null
    }

    $existingIds = @()
    if (Test-Path $IdentityBase) {
        $existingIds = Get-ChildItem -Path $IdentityBase -Directory -Filter "identity-*" |
            ForEach-Object {
                $num = $_.Name -replace 'identity-', ''
                if ($num -match '^\d+$') { [int]$num }
                elseif ($num -match '^[a-z]$') {
                    [int][char]$num - [int][char]'a' + 1
                }
            } | Sort-Object
    }

    if ($existingIds.Count -gt 0) {
        $nextId = ($existingIds | Measure-Object -Maximum).Maximum + 1
    } else {
        $nextId = 1
    }

    $dir = Join-Path $IdentityBase "identity-$nextId"
    if (-not (Test-Path $dir)) {
        Write-Host "IDENTITY: Creating identity directory: $dir" -ForegroundColor Yellow
        New-Item -ItemType Directory -Path $dir -Force | Out-Null
    }
    $keyFile = Join-Path $dir "master.key"
    if (-not (Test-Path $keyFile)) {
        Write-Host "IDENTITY: Generating identity #$nextId..." -ForegroundColor Yellow
        $env:CUBE_MODE = "keygen"
        $env:CUBE_IDENTITY_DIR = $dir
        $null = & $BinaryPath 2>&1
        Remove-Item Env:\CUBE_MODE -ErrorAction SilentlyContinue
        Remove-Item Env:\CUBE_IDENTITY_DIR -ErrorAction SilentlyContinue
        if (Test-Path $keyFile) {
            Write-Host "IDENTITY: Daemon #$nextId identity created." -ForegroundColor Green
        } else {
            Write-Host "IDENTITY: WARNING - Identity #$nextId key generation may have failed." -ForegroundColor Yellow
        }
    } else {
        Write-Host "IDENTITY: Daemon #$nextId identity already exists." -ForegroundColor Green
    }

    $enginePort = $BaseEnginePort + (($nextId - 1) * $PortStep)
    $daemonPort = $enginePort + 1

    Write-Host ""
    Write-Host "==========================================================" -ForegroundColor Cyan
    Write-Host "  READY TO LAUNCH" -ForegroundColor Cyan
    Write-Host "==========================================================" -ForegroundColor Cyan
    Write-Host ""

    $allIds = @()
    if (Test-Path $IdentityBase) {
        $allIds = Get-ChildItem -Path $IdentityBase -Directory -Filter "identity-*" |
            ForEach-Object {
                $num = $_.Name -replace 'identity-', ''
                if ($num -match '^\d+$') { [int]$num }
                elseif ($num -match '^[a-z]$') {
                    [int][char]$num - [int][char]'a' + 1
                }
            } | Sort-Object
    }

    foreach ($id in $allIds) {
        $ep = $BaseEnginePort + (($id - 1) * $PortStep)
        $dp = $ep + 1

        $idDir = Join-Path $IdentityBase "identity-$id"
        if (-not (Test-Path $idDir)) {
            $letterDir = Join-Path $IdentityBase ("identity-" + [char]([int][char]'a' + $id - 1))
            if (Test-Path $letterDir) { $idDir = $letterDir }
        }

        Write-Host "  Start Daemon #$id (engine=$ep, daemon=$dp):" -ForegroundColor White
        Write-Host "    `$env:CUBE_MODE=`"cube`"; `$env:CUBE_API_PORT=`"$dp`"; `$env:LLM_PORT=`"$ep`"" -ForegroundColor DarkGray
        Write-Host "    `$env:CUBE_CRS_URL=`"https://plenumnet.replit.app`"; `$env:CUBE_ROLE=`"inference`"" -ForegroundColor DarkGray
        Write-Host "    `$env:CUBE_IDENTITY_DIR=`"$idDir`"" -ForegroundColor DarkGray
        Write-Host ('    & "' + $BinaryPath + '"') -ForegroundColor DarkGray
        Write-Host ""
    }

    Write-Host "  Total identities: $($allIds.Count)" -ForegroundColor White
    Write-Host "  Run this script again to add another daemon." -ForegroundColor DarkGray
    Write-Host ""
} finally {
    Pop-Location
}
