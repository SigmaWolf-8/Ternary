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

    $hostname = $env:COMPUTERNAME
    $cpuArch = if ([System.Runtime.InteropServices.RuntimeInformation]::OSArchitecture -eq 'Arm64') { "Arm64" } else { "x64" }
    $ip = (Get-NetIPAddress -AddressFamily IPv4 |
        Where-Object { $_.IPAddress -notmatch '^127\.' -and $_.IPAddress -notmatch '^169\.254' -and $_.PrefixOrigin -ne 'WellKnown' } |
        Sort-Object @{ Expression = { switch -Wildcard ($_.InterfaceAlias) { 'Wi-Fi*' { 0 } 'Ethernet*' { 1 } default { 2 } } } } |
        Select-Object -First 1).IPAddress
    if (-not $ip) { $ip = "0.0.0.0" }
    Write-Host "  Hostname:  $hostname" -ForegroundColor White
    Write-Host "  IP:        $ip" -ForegroundColor White
    Write-Host "  Arch:      $cpuArch" -ForegroundColor White
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

    $CRS_URL = "https://plenumnet.replit.app"
    $daemonsArray = @()

    foreach ($id in $allIds) {
        $ep = $BaseEnginePort + (($id - 1) * $PortStep)
        $dp = $ep + 1

        $idDir = Join-Path $IdentityBase "identity-$id"
        if (-not (Test-Path $idDir)) {
            $letterDir = Join-Path $IdentityBase ("identity-" + [char]([int][char]'a' + $id - 1))
            if (Test-Path $letterDir) { $idDir = $letterDir }
        }

        $pubKeyFile = Join-Path $idDir "public.key"
        $pubKey = if (Test-Path $pubKeyFile) { (Get-Content $pubKeyFile -Raw).Trim() } else { "" }

        $daemonsArray += @{
            id = $id
            port = $dp
            address = ""
            publicKey = $pubKey
            endpoint = "${ip}:${dp}"
            identityDir = $idDir
            pid = 0
        }

        Write-Host "  Start Daemon #$id (engine=$ep, daemon=$dp):" -ForegroundColor White
        Write-Host "    `$env:CUBE_MODE=`"cube`"; `$env:CUBE_API_PORT=`"$dp`"; `$env:LLM_PORT=`"$ep`"" -ForegroundColor DarkGray
        Write-Host "    `$env:CUBE_CRS_URL=`"$CRS_URL`"; `$env:CUBE_ROLE=`"inference`"" -ForegroundColor DarkGray
        Write-Host "    `$env:CUBE_IDENTITY_DIR=`"$idDir`"" -ForegroundColor DarkGray
        Write-Host ('    & "' + $BinaryPath + '"') -ForegroundColor DarkGray
        Write-Host ""
    }

    Write-Host "  Total identities: $($allIds.Count)" -ForegroundColor White
    Write-Host ""

    $isAdmin = $false
    try {
        $currentPrincipal = New-Object Security.Principal.WindowsPrincipal([Security.Principal.WindowsIdentity]::GetCurrent())
        $isAdmin = $currentPrincipal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
    } catch {}

    if ($isAdmin) {
        Write-Host "==========================================================" -ForegroundColor Cyan
        Write-Host "  REGISTERING WINDOWS SERVICES" -ForegroundColor Cyan
        Write-Host "==========================================================" -ForegroundColor Cyan
        Write-Host ""

        $LogDir = Join-Path $IdentityBase "logs"
        if (-not (Test-Path $LogDir)) {
            New-Item -ItemType Directory -Path $LogDir -Force | Out-Null
        }

        foreach ($id in $allIds) {
            $ep = $BaseEnginePort + (($id - 1) * $PortStep)
            $dp = $ep + 1
            $idDir = Join-Path $IdentityBase "identity-$id"
            if (-not (Test-Path $idDir)) {
                $letterDir = Join-Path $IdentityBase ("identity-" + [char]([int][char]'a' + $id - 1))
                if (Test-Path $letterDir) { $idDir = $letterDir }
            }

            $svcName = "PlenumNET-Cube-$id"
            $logFile = Join-Path $LogDir "cube-${id}.log"

            $wrapperDir = Join-Path $RepoDir "services\wrappers"
            if (-not (Test-Path $wrapperDir)) {
                New-Item -ItemType Directory -Path $wrapperDir -Force | Out-Null
            }
            $wrapperBat = Join-Path $wrapperDir "cube-${id}-start.bat"
            @"
@echo off
set CUBE_MODE=cube
set CUBE_API_PORT=$dp
set LLM_PORT=$ep
set CUBE_CRS_URL=$CRS_URL
set CUBE_IDENTITY_DIR=$idDir
set CUBE_ROLE=inference
"$BinaryPath" >> "$logFile" 2>&1
"@ | Set-Content -Path $wrapperBat -Encoding ASCII

            $existingSvc = Get-Service -Name $svcName -ErrorAction SilentlyContinue
            if ($existingSvc) {
                Stop-Service -Name $svcName -Force -ErrorAction SilentlyContinue
                & sc.exe delete $svcName | Out-Null
                Start-Sleep -Seconds 1
            }

            try {
                $svcBinPath = "cmd.exe /c `"$wrapperBat`""
                New-Service -Name $svcName `
                    -BinaryPathName $svcBinPath `
                    -DisplayName "PlenumNET Inter-Cube Daemon (Identity #$id)" `
                    -Description "PlenumNET Inter-Cube infrastructure daemon for identity #$id" `
                    -StartupType Automatic | Out-Null

                & sc.exe failure $svcName reset= 86400 actions= restart/5000/restart/10000/restart/30000 | Out-Null
                & sc.exe config $svcName depend= Tcpip/Afd/Dnscache | Out-Null

                $svcAccount = Get-ServiceAccountName
                Set-ServiceLogonAccount -ServiceName $svcName -AccountName $svcAccount

                Start-Service -Name $svcName
                Write-Host "  [OK] Daemon #$id registered as Windows Service: $svcName" -ForegroundColor Green
            } catch {
                Write-Host "  [WARN] Could not register daemon #$id as service: $_" -ForegroundColor Yellow
            }
        }

        Write-Host ""
        Write-Host "  Service management:" -ForegroundColor White
        Write-Host "    Get-Service PlenumNET-Cube-*       # Check all daemon services" -ForegroundColor DarkGray
        Write-Host "    Restart-Service PlenumNET-Cube-1   # Restart daemon #1" -ForegroundColor DarkGray
        Write-Host "    powershell -File `"$RepoDir\client\public\install\plenumnet-service.ps1`" status" -ForegroundColor DarkGray
        Write-Host ""
    } else {
        Write-Host "  NOTE: Run as Administrator to register daemons as Windows Services." -ForegroundColor Yellow
        Write-Host "        Services auto-start on boot and restart on failure." -ForegroundColor DarkGray
        Write-Host ""
    }

    Write-Host "  Run this script again to add another daemon." -ForegroundColor DarkGray
    Write-Host ""

    $deploymentPayload = @{
        hostname = $hostname
        ip = $ip
        architecture = $cpuArch
        daemonCount = $allIds.Count
        daemons = $daemonsArray
        crsUrl = $CRS_URL
        binaryPath = $BinaryPath
        binarySizeMB = $fileSizeMB
        logDir = ""
        identityBase = $IdentityBase
        timestamp = (Get-Date -Format "o")
        deployer = "deploy-daemon/v0.3.0"
    } | ConvertTo-Json -Depth 3

    try {
        $null = Invoke-RestMethod -Uri "$CRS_URL/api/salvi/inter-cube/relay/deployment" -Method Post -Body $deploymentPayload -ContentType "application/json" -TimeoutSec 15 -ErrorAction Stop
        Write-Host "  [OK] Deployment recorded with CRS Daemon Registry" -ForegroundColor Green
        Write-Host "       Query: $CRS_URL/api/salvi/inter-cube/relay/deployments?hostname=$hostname" -ForegroundColor DarkGray
    } catch {
        Write-Host "  WARN: Could not post deployment record -- $_" -ForegroundColor Yellow
    }
    Write-Host ""
} finally {
    Pop-Location
}
