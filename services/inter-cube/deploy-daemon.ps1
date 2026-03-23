<#
.SYNOPSIS
    PlenumNET Cube Daemon Deployer
    Pulls latest source from GitHub, builds the daemon, and verifies the binary.
    Run from any directory - handles C:\PlenumNET automatically.

.DESCRIPTION
    This script is served from https://plenumnet.replit.app/api/deploy-daemon
    Run it with:
      irm https://plenumnet.replit.app/api/deploy-daemon | iex

.NOTES
    Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
    Applied Physics Division
#>

$RepoDir = "C:\PlenumNET"
$BinaryName = "inter-cube-daemon.exe"
$BinaryPath = Join-Path $RepoDir "target\release\$BinaryName"
$RepoUrl = "https://github.com/SigmaWolf-8/Ternary.git"

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
    & cargo build --release -p inter-cube 2>&1 | ForEach-Object {
        $line = $_.ToString()
        if ($line -match "Compiling|Finished") { Write-Host "       $line" -ForegroundColor DarkGray }
    }
    if ($LASTEXITCODE -ne 0) {
        Write-Host 'ERROR: Build failed. Check output above.' -ForegroundColor Red
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

    $identityBase = Join-Path $env:USERPROFILE ".plenumnet"
    foreach ($agent in @("a", "b", "c")) {
        $dir = Join-Path $identityBase "identity-$agent"
        if (-not (Test-Path $dir)) {
            Write-Host "IDENTITY: Creating identity directory: $dir" -ForegroundColor Yellow
            New-Item -ItemType Directory -Path $dir -Force | Out-Null
        }
        $keyFile = Join-Path $dir "master.key"
        if (-not (Test-Path $keyFile)) {
            $label = $agent.ToUpper()
            Write-Host "IDENTITY: Generating identity for Agent $label..." -ForegroundColor Yellow
            $env:CUBE_MODE = "keygen"
            $env:CUBE_IDENTITY_DIR = $dir
            $null = & $BinaryPath 2>&1
            Remove-Item Env:\CUBE_MODE -ErrorAction SilentlyContinue
            Remove-Item Env:\CUBE_IDENTITY_DIR -ErrorAction SilentlyContinue
            if (Test-Path $keyFile) {
                Write-Host "IDENTITY: Agent $label identity created." -ForegroundColor Green
            } else {
                Write-Host "IDENTITY: WARNING - Agent $label key generation may have failed." -ForegroundColor Yellow
            }
        } else {
            Write-Host "IDENTITY: Agent $($agent.ToUpper()) identity exists." -ForegroundColor Green
        }
    }

    Write-Host ""
    Write-Host "==========================================================" -ForegroundColor Cyan
    Write-Host "  READY TO LAUNCH" -ForegroundColor Cyan
    Write-Host "==========================================================" -ForegroundColor Cyan
    Write-Host ""
    Write-Host "  Start Daemon A:" -ForegroundColor White
    Write-Host '    $env:CUBE_MODE="cube"; $env:CUBE_API_PORT="8081"; $env:LLM_PORT="8080"' -ForegroundColor DarkGray
    Write-Host '    $env:CUBE_CRS_URL="https://plenumnet.replit.app"; $env:CUBE_ROLE="inference"' -ForegroundColor DarkGray
    Write-Host '    $env:CUBE_IDENTITY_DIR="$env:USERPROFILE\.plenumnet\identity-a"' -ForegroundColor DarkGray
    Write-Host ('    & "' + $BinaryPath + '"') -ForegroundColor DarkGray
    Write-Host ""
    Write-Host "  Start Daemon B:" -ForegroundColor White
    Write-Host '    $env:CUBE_MODE="cube"; $env:CUBE_API_PORT="8083"; $env:LLM_PORT="8082"' -ForegroundColor DarkGray
    Write-Host '    $env:CUBE_CRS_URL="https://plenumnet.replit.app"; $env:CUBE_ROLE="inference"' -ForegroundColor DarkGray
    Write-Host '    $env:CUBE_IDENTITY_DIR="$env:USERPROFILE\.plenumnet\identity-b"' -ForegroundColor DarkGray
    Write-Host ('    & "' + $BinaryPath + '"') -ForegroundColor DarkGray
    Write-Host ""
    Write-Host "  Start Daemon C:" -ForegroundColor White
    Write-Host '    $env:CUBE_MODE="cube"; $env:CUBE_API_PORT="8085"; $env:LLM_PORT="8084"' -ForegroundColor DarkGray
    Write-Host '    $env:CUBE_CRS_URL="https://plenumnet.replit.app"; $env:CUBE_ROLE="inference"' -ForegroundColor DarkGray
    Write-Host '    $env:CUBE_IDENTITY_DIR="$env:USERPROFILE\.plenumnet\identity-c"' -ForegroundColor DarkGray
    Write-Host ('    & "' + $BinaryPath + '"') -ForegroundColor DarkGray
    Write-Host ""
} finally {
    Pop-Location
}
