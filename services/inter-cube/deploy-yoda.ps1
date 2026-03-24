<#
.SYNOPSIS
    PlenumNET Array3 Deployer
    Builds the node binary, generates 3 PT26-DSA identities, starts a
    3-node PlenumNET Array3 cluster, and posts a deployment summary to
    the PlenumNET Node Registry for monitoring.

.DESCRIPTION
    Served from https://plenumnet.replit.app/api/deploy-yoda
    Run with:  irm https://plenumnet.replit.app/api/deploy-yoda | iex
    Or download the .bat wrapper from the Distribution page.

    Array3 tri-port topology (port step = 3):
      Node #1 (Agent A) : peer 8079, app 8080, node 8081  -- Coordinator  (CUBE_MODE=crs)
      Node #2 (Agent B) : peer 8082, app 8083, node 8084  -- Worker       (registers with Node #1)
      Node #3 (Agent C) : peer 8085, app 8086, node 8087  -- Worker       (registers with Node #1)

    Each daemon uses 3 ports: peer (WebSocket LAN mesh), app (application
    forwarding), and node (CRS/cube HTTP API). Port step is 3.

    Node #1 is always the coordinator for the Array3. Nodes #2 and #3
    register with it at http://localhost:8081. The remote PlenumNET server
    (plenumnet.replit.app) only receives a deployment summary for the
    dashboard -- it is NOT the CRS for local node operations.

    All 3 nodes connect outbound to plenumnet.replit.app via WebSocket
    relay (RELAY_URL). This is the NAT-traversal tunnel through which
    applications like YODA dispatch requests. Each node forwards requests
    to a local application port at 127.0.0.1:{LLM_PORT}.

    LAN peers connect directly via the peer port for low-latency
    intra-cluster communication, bypassing the relay when possible.

    Application engines are NOT installed by this script. Application
    setup is handled separately by the consuming app (e.g. YODA).

.NOTES
    Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
    Applied Physics Division
#>

$DAEMON_COUNT  = 3
$REMOTE_CRS    = "https://plenumnet.replit.app"
$BASE_PEER_PORT = 8079
$PORT_STEP     = 3
$BASE_DAEMON_PORT = $BASE_PEER_PORT + 2
$LOCAL_CRS_PORT = $BASE_DAEMON_PORT
$LOCAL_CRS_URL = "http://localhost:$LOCAL_CRS_PORT"
$RepoDir       = "C:\PlenumNET"
$BinaryName    = "inter-cube-daemon.exe"
$BinaryPath    = Join-Path $RepoDir "target\release\$BinaryName"
$RepoUrl       = "https://github.com/SigmaWolf-8/Ternary.git"
$IdentityBase  = Join-Path $env:USERPROFILE ".plenumnet"
$LOG_DIR       = Join-Path $IdentityBase "logs"

Write-Host ""
Write-Host "==========================================================" -ForegroundColor Cyan
Write-Host "  PlenumNET Array3 Deployer" -ForegroundColor Cyan
Write-Host "  PlenumNET Inter-Cube Infrastructure" -ForegroundColor Cyan
Write-Host "  Applied Physics Division -- Capomastro Holdings Ltd." -ForegroundColor Cyan
Write-Host "==========================================================" -ForegroundColor Cyan
Write-Host ""
Write-Host "  Nodes        : $DAEMON_COUNT instances" -ForegroundColor White
Write-Host "  Coordinator  : Node #1 (port $LOCAL_CRS_PORT)" -ForegroundColor White
Write-Host "  Node Ports   : $BASE_DAEMON_PORT, $($BASE_DAEMON_PORT + $PORT_STEP), $($BASE_DAEMON_PORT + 2 * $PORT_STEP)" -ForegroundColor White
Write-Host "  App Ports    : $($BASE_PEER_PORT + 1), $($BASE_PEER_PORT + $PORT_STEP + 1), $($BASE_PEER_PORT + 2 * $PORT_STEP + 1)" -ForegroundColor White
Write-Host "  Peer Ports   : $BASE_PEER_PORT, $($BASE_PEER_PORT + $PORT_STEP), $($BASE_PEER_PORT + 2 * $PORT_STEP)" -ForegroundColor White
Write-Host "  Relay        : $REMOTE_CRS (WebSocket NAT traversal)" -ForegroundColor White
Write-Host "  Registry     : $REMOTE_CRS (monitoring only)" -ForegroundColor White
Write-Host ""

function Test-Command($cmd) {
    try { Get-Command $cmd -ErrorAction Stop | Out-Null; return $true }
    catch { return $false }
}

# ── 1. Prerequisites ──────────────────────────────────────────────────────────
Write-Host "STEP 1/8: Checking prerequisites" -ForegroundColor Yellow
Write-Host "---"

if (-not (Test-Command "git")) {
    Write-Host 'ERROR: git is not installed or not in PATH.' -ForegroundColor Red
    Write-Host '       Install from https://git-scm.com/download/win' -ForegroundColor Yellow
    Read-Host "Press Enter to close"
    return
}
Write-Host "  [OK] git" -ForegroundColor Green

if (-not (Test-Command "cargo")) {
    Write-Host "  -> Rust not found -- installing rustup..." -ForegroundColor Yellow
    $rustupExe = Join-Path $env:TEMP "rustup-init.exe"
    Invoke-WebRequest -Uri "https://win.rustup.rs/x86_64" -OutFile $rustupExe -UseBasicParsing
    Start-Process -FilePath $rustupExe -ArgumentList "-y" -Wait -NoNewWindow
    Remove-Item $rustupExe -Force -ErrorAction SilentlyContinue
    $cargoBin = Join-Path (Join-Path $env:USERPROFILE ".cargo") "bin"
    $env:PATH += ";$cargoBin"
    if (-not (Test-Command "cargo")) {
        Write-Host 'ERROR: cargo not found after install -- restart in a new terminal.' -ForegroundColor Red
        Read-Host "Press Enter to close"
        return
    }
}
Write-Host "  [OK] cargo" -ForegroundColor Green

# ── 2. Build environment (MSVC + clang for ring crate) ─────────────────────
Write-Host ""
Write-Host "STEP 2/8: Build environment" -ForegroundColor Yellow
Write-Host "---"

$cpuArch = [System.Runtime.InteropServices.RuntimeInformation]::OSArchitecture.ToString()
Write-Host "  Architecture: $cpuArch"

$vsWhere = $null
$searchPaths = @(
    "${env:ProgramFiles(x86)}\Microsoft Visual Studio\Installer\vswhere.exe",
    "${env:ProgramFiles}\Microsoft Visual Studio\Installer\vswhere.exe",
    "C:\Program Files (x86)\Microsoft Visual Studio\Installer\vswhere.exe",
    "C:\Program Files\Microsoft Visual Studio\Installer\vswhere.exe"
)
foreach ($p in $searchPaths) {
    if (Test-Path -LiteralPath $p) { $vsWhere = $p; break }
}
if ($vsWhere) {
    $vsPath = & $vsWhere -latest -products * -property installationPath 2>$null
    if ($vsPath) {
        $vcvarsName = if ($cpuArch -eq "Arm64") { "vcvarsarm64.bat" } else { "vcvars64.bat" }
        $vcvars = Join-Path (Join-Path (Join-Path $vsPath "VC") "Auxiliary\Build") $vcvarsName
        if (Test-Path -LiteralPath $vcvars) {
            $envLines = cmd.exe /c ('"' + $vcvars + '" > nul 2>&1 && set')
            foreach ($line in $envLines) {
                if ($line -match '^([^=\r\n]+)=(.*)$') {
                    [System.Environment]::SetEnvironmentVariable($Matches[1], $Matches[2], "Process")
                }
            }
            Write-Host "  [OK] MSVC environment activated ($vcvarsName)" -ForegroundColor Green
        }
    }
}

$hasClang = Get-Command clang -ErrorAction SilentlyContinue
if (-not $hasClang) {
    $llvmBin = "C:\Program Files\LLVM\bin"
    if (Test-Path -LiteralPath (Join-Path $llvmBin "clang.exe")) {
        $env:PATH += ";$llvmBin"
        $hasClang = Get-Command clang -ErrorAction SilentlyContinue
    }
}
if ($hasClang) {
    Write-Host "  [OK] clang" -ForegroundColor Green
    $env:CC = "clang"
    $env:AR = "llvm-ar"
} else {
    Write-Host "  -> clang not found -- installing LLVM via winget..." -ForegroundColor Yellow
    $hasWinget = Get-Command winget -ErrorAction SilentlyContinue
    if ($hasWinget) {
        winget install --id LLVM.LLVM --silent --accept-package-agreements --accept-source-agreements
    } else {
        $llvmRelease = (Invoke-RestMethod "https://api.github.com/repos/llvm/llvm-project/releases/latest").tag_name
        $llvmVer = $llvmRelease -replace "llvmorg-",""
        $llvmUrl = "https://github.com/llvm/llvm-project/releases/download/$llvmRelease/LLVM-$llvmVer-win64.exe"
        $llvmInstaller = Join-Path $env:TEMP "llvm-installer.exe"
        if (Get-Command curl.exe -ErrorAction SilentlyContinue) {
            curl.exe -fL "$llvmUrl" -o "$llvmInstaller"
        } else {
            Invoke-WebRequest -Uri $llvmUrl -OutFile $llvmInstaller -UseBasicParsing
        }
        Start-Process -FilePath $llvmInstaller -ArgumentList "/S" -Wait -NoNewWindow
        Remove-Item $llvmInstaller -Force -ErrorAction SilentlyContinue
    }
    $env:PATH = [System.Environment]::GetEnvironmentVariable("PATH","Machine") + ";" + [System.Environment]::GetEnvironmentVariable("PATH","User")
    $llvmBin = "C:\Program Files\LLVM\bin"
    if (Test-Path -LiteralPath (Join-Path $llvmBin "clang.exe")) { $env:PATH += ";$llvmBin" }
    if (Get-Command clang -ErrorAction SilentlyContinue) {
        Write-Host "  [OK] clang installed" -ForegroundColor Green
        $env:CC = "clang"
        $env:AR = "llvm-ar"
    } else {
        Write-Host "ERROR: clang not found after LLVM install -- restart in a new terminal." -ForegroundColor Red
        Read-Host "Press Enter to close"
        return
    }
}

# ── 3. Clone/update and build daemon ──────────────────────────────────────
Write-Host ""
Write-Host "STEP 3/8: Source code + build" -ForegroundColor Yellow
Write-Host "---"

if (-not (Test-Path $RepoDir)) {
    Write-Host "  Cloning PlenumNET repository..." -ForegroundColor White
    $null = & git clone --depth 1 $RepoUrl $RepoDir 2>&1
    if ($LASTEXITCODE -ne 0) {
        Write-Host 'ERROR: git clone failed.' -ForegroundColor Red
        Read-Host "Press Enter to close"
        return
    }
} elseif (-not (Test-Path (Join-Path $RepoDir ".git"))) {
    Write-Host "  Converting ZIP install to git repo..." -ForegroundColor Yellow
    Push-Location $RepoDir
    $null = & git init 2>&1
    $null = & git remote add origin $RepoUrl 2>&1
    $null = & git fetch origin main 2>&1
    $null = & git reset --hard origin/main 2>&1
    Pop-Location
} else {
    Write-Host "  Updating source..." -ForegroundColor White
    Push-Location $RepoDir
    $null = & git pull origin main --ff-only 2>&1
    Pop-Location
}
Write-Host "  [OK] Source ready" -ForegroundColor Green

$runningDaemons = Get-Process -Name "inter-cube-daemon" -ErrorAction SilentlyContinue
if ($runningDaemons) {
    Write-Host "  Stopping running node(s)..." -ForegroundColor Yellow
    $runningDaemons | Stop-Process -Force
    Start-Sleep -Seconds 2
}

Write-Host "  Building inter-cube daemon (CARGO_BUILD_JOBS=1)..." -ForegroundColor White
Push-Location $RepoDir
$env:CARGO_BUILD_JOBS = "1"
& cargo build --release -p inter-cube 2>&1 | ForEach-Object {
    $line = $_.ToString()
    if ($line -match "error") { Write-Host "  $line" -ForegroundColor Red }
    elseif ($line -match "Compiling|Finished") { Write-Host "  $line" -ForegroundColor DarkGray }
}
$buildExit = $LASTEXITCODE
Pop-Location
if ($buildExit -ne 0) {
    Write-Host 'ERROR: Build failed.' -ForegroundColor Red
    Read-Host "Press Enter to close"
    return
}
if (-not (Test-Path $BinaryPath)) {
    Write-Host "ERROR: Binary not found at $BinaryPath" -ForegroundColor Red
    Read-Host "Press Enter to close"
    return
}
$fileSizeMB = [math]::Round((Get-Item $BinaryPath).Length / 1MB, 1)
Write-Host "  [OK] Build successful ($fileSizeMB MB)" -ForegroundColor Green

# ── 4. Version check ─────────────────────────────────────────────────────
Write-Host ""
Write-Host "STEP 4/8: Version check" -ForegroundColor Yellow
Write-Host "---"

$localVersion = "unknown"
try {
    $env:CUBE_MODE = "keygen"
    $env:CUBE_IDENTITY_DIR = Join-Path $env:TEMP "plenumnet-version-probe"
    New-Item -ItemType Directory -Force -Path $env:CUBE_IDENTITY_DIR | Out-Null
    $versionOutput = & $BinaryPath 2>&1
    Remove-Item Env:\CUBE_MODE -ErrorAction SilentlyContinue
    Remove-Item Env:\CUBE_IDENTITY_DIR -ErrorAction SilentlyContinue
    $vLine = $versionOutput | Where-Object { $_ -match "version|v\d+\.\d+" } | Select-Object -First 1
    if ($vLine -match '(\d+\.\d+\.\d+)') { $localVersion = $Matches[1] }
} catch {}

$remoteVersion = "unknown"
try {
    $crsHealth = Invoke-RestMethod -Uri "$REMOTE_CRS/health/crs" -TimeoutSec 10 -ErrorAction Stop
    $remoteVersion = $crsHealth.version
} catch {}

Write-Host "  Local node    : v$localVersion" -ForegroundColor White
Write-Host "  CRS reference : v$remoteVersion" -ForegroundColor White

if ($localVersion -ne "unknown" -and $remoteVersion -ne "unknown" -and $localVersion -ne $remoteVersion) {
    Write-Host "  NOTE: Version mismatch -- local v$localVersion vs CRS v$remoteVersion" -ForegroundColor Yellow
    Write-Host "        Run the deployer again after 'git pull' to update." -ForegroundColor Yellow
} else {
    Write-Host "  [OK] Version aligned" -ForegroundColor Green
}

# ── 5. Detect local IP ────────────────────────────────────────────────────
Write-Host ""
Write-Host "STEP 5/8: Network detection" -ForegroundColor Yellow
Write-Host "---"

$ip = (Get-NetIPAddress -AddressFamily IPv4 |
    Where-Object { $_.IPAddress -notmatch '^127\.' -and $_.IPAddress -notmatch '^169\.254' -and $_.PrefixOrigin -ne 'WellKnown' } |
    Sort-Object @{ Expression = { switch -Wildcard ($_.InterfaceAlias) { 'Wi-Fi*' { 0 } 'Ethernet*' { 1 } default { 2 } } } } |
    Select-Object -First 1).IPAddress
if (-not $ip) { $ip = "0.0.0.0" }
Write-Host "  Local IP: $ip" -ForegroundColor White

# ── 6. Generate 3 identities ─────────────────────────────────────────────
Write-Host ""
Write-Host "STEP 6/8: Generating $DAEMON_COUNT node identities" -ForegroundColor Yellow
Write-Host "---"

New-Item -ItemType Directory -Force -Path $IdentityBase | Out-Null
New-Item -ItemType Directory -Force -Path $LOG_DIR | Out-Null

$daemonConfigs = @()
for ($i = 1; $i -le $DAEMON_COUNT; $i++) {
    $dir = Join-Path $IdentityBase "identity-$i"
    $keyFile = Join-Path $dir "master.key"
    $peerPort = $BASE_PEER_PORT + (($i - 1) * $PORT_STEP)
    $appPort = $peerPort + 1
    $daemonPort = $peerPort + 2
    $endpoint = "${ip}:${daemonPort}"

    New-Item -ItemType Directory -Force -Path $dir | Out-Null

    if (-not (Test-Path $keyFile)) {
        Write-Host "  Generating identity #$i..." -ForegroundColor White
        $env:CUBE_MODE = "keygen"
        $env:CUBE_IDENTITY_DIR = $dir
        $keygenOutput = & $BinaryPath 2>&1
        Remove-Item Env:\CUBE_MODE -ErrorAction SilentlyContinue
        Remove-Item Env:\CUBE_IDENTITY_DIR -ErrorAction SilentlyContinue
        if (Test-Path $keyFile) {
            Write-Host "  [OK] Node #$i identity created" -ForegroundColor Green
        } else {
            Write-Host "  WARN: Node #$i keygen may have failed" -ForegroundColor Yellow
        }
    } else {
        Write-Host "  [OK] Node #$i identity exists" -ForegroundColor Green
    }

    $pubKey = ""
    $env:CUBE_MODE = "keygen"
    $env:CUBE_IDENTITY_DIR = $dir
    $infoOutput = & $BinaryPath 2>&1
    Remove-Item Env:\CUBE_MODE -ErrorAction SilentlyContinue
    Remove-Item Env:\CUBE_IDENTITY_DIR -ErrorAction SilentlyContinue
    $pkLine = $infoOutput | Where-Object { $_ -match "PT26-DSA Public Key|Public Key|pk:" } | Select-Object -First 1
    if ($pkLine -match ':\s*([0-9a-fA-F]+)\s*$') {
        $pubKey = $Matches[1]
    }

    $mode = if ($i -eq 1) { "crs" } else { "cube" }

    $daemonConfigs += @{
        Id = $i
        Port = $daemonPort
        AppPort = $appPort
        PeerPort = $peerPort
        IdentityDir = $dir
        Endpoint = $endpoint
        PublicKey = $pubKey
        Mode = $mode
    }
}

# ── 7. Start daemons (CRS first, then cubes) + register with local CRS ──
Write-Host ""
Write-Host "STEP 7/8: Starting Array3 (Node #1 = coordinator)" -ForegroundColor Yellow
Write-Host "---"

$daemonPids = @()

$crsCfg = $daemonConfigs[0]
$env:CUBE_MODE = "crs"
$env:CUBE_API_PORT = "$($crsCfg.Port)"
$env:CUBE_PEER_PORT = "$($crsCfg.PeerPort)"
$env:CUBE_ENDPOINT = $crsCfg.Endpoint
$env:CUBE_IDENTITY_DIR = $crsCfg.IdentityDir
$env:RELAY_URL = $REMOTE_CRS
$env:LLM_PORT = "$($crsCfg.AppPort)"

$outLog = Join-Path $LOG_DIR "daemon-1-out.log"
$errLog = Join-Path $LOG_DIR "daemon-1-err.log"
$proc = Start-Process -FilePath $BinaryPath -NoNewWindow -PassThru -RedirectStandardOutput $outLog -RedirectStandardError $errLog
$daemonPids += $proc.Id
Write-Host "  [OK] Node #1 started as coordinator (PID $($proc.Id), node $($crsCfg.Port), app $($crsCfg.AppPort), peer $($crsCfg.PeerPort), relay -> $REMOTE_CRS)" -ForegroundColor Green

Remove-Item Env:\CUBE_MODE -ErrorAction SilentlyContinue
Remove-Item Env:\CUBE_API_PORT -ErrorAction SilentlyContinue
Remove-Item Env:\CUBE_PEER_PORT -ErrorAction SilentlyContinue
Remove-Item Env:\CUBE_ENDPOINT -ErrorAction SilentlyContinue
Remove-Item Env:\CUBE_IDENTITY_DIR -ErrorAction SilentlyContinue
Remove-Item Env:\RELAY_URL -ErrorAction SilentlyContinue
Remove-Item Env:\LLM_PORT -ErrorAction SilentlyContinue

Write-Host "  Waiting for coordinator to be ready..." -ForegroundColor DarkGray
$crsReady = $false
for ($w = 1; $w -le 15; $w++) {
    Start-Sleep -Seconds 2
    try {
        $healthCheck = Invoke-RestMethod -Uri "$LOCAL_CRS_URL/health" -TimeoutSec 5 -ErrorAction Stop
        if ($healthCheck.status -eq "ok") { $crsReady = $true; break }
    } catch {}
}
if ($crsReady) {
    Write-Host "  [OK] Coordinator ready at $LOCAL_CRS_URL" -ForegroundColor Green
} else {
    Write-Host "  WARN: Coordinator health check did not respond -- continuing anyway" -ForegroundColor Yellow
}

for ($i = 1; $i -lt $DAEMON_COUNT; $i++) {
    $cfg = $daemonConfigs[$i]
    $env:CUBE_MODE = "cube"
    $env:CUBE_CRS_URL = $LOCAL_CRS_URL
    $env:CUBE_ENDPOINT = $cfg.Endpoint
    $env:CUBE_API_PORT = "$($cfg.Port)"
    $env:CUBE_PEER_PORT = "$($cfg.PeerPort)"
    $env:CUBE_IDENTITY_DIR = $cfg.IdentityDir
    $env:RELAY_URL = $REMOTE_CRS
    $env:LLM_PORT = "$($cfg.AppPort)"

    $outLog = Join-Path $LOG_DIR "daemon-$($cfg.Id)-out.log"
    $errLog = Join-Path $LOG_DIR "daemon-$($cfg.Id)-err.log"
    $proc = Start-Process -FilePath $BinaryPath -NoNewWindow -PassThru -RedirectStandardOutput $outLog -RedirectStandardError $errLog
    $daemonPids += $proc.Id
    Write-Host "  [OK] Node #$($cfg.Id) started (PID $($proc.Id), node $($cfg.Port), app $($cfg.AppPort), peer $($cfg.PeerPort), relay -> $REMOTE_CRS)" -ForegroundColor Green
    Start-Sleep -Seconds 1
}

Remove-Item Env:\CUBE_MODE -ErrorAction SilentlyContinue
Remove-Item Env:\CUBE_CRS_URL -ErrorAction SilentlyContinue
Remove-Item Env:\CUBE_ENDPOINT -ErrorAction SilentlyContinue
Remove-Item Env:\CUBE_API_PORT -ErrorAction SilentlyContinue
Remove-Item Env:\CUBE_PEER_PORT -ErrorAction SilentlyContinue
Remove-Item Env:\CUBE_IDENTITY_DIR -ErrorAction SilentlyContinue
Remove-Item Env:\RELAY_URL -ErrorAction SilentlyContinue
Remove-Item Env:\LLM_PORT -ErrorAction SilentlyContinue

Start-Sleep -Seconds 3

$registeredAddresses = @()
foreach ($cfg in $daemonConfigs) {
    if ($cfg.Mode -eq "crs") {
        try {
            $crsInfo = Invoke-RestMethod -Uri "$LOCAL_CRS_URL/health" -TimeoutSec 5 -ErrorAction Stop
            $cfg.Address = $crsInfo.address
            $registeredAddresses += $crsInfo.address
            Write-Host "  [OK] Node #1 (coordinator) address: $($cfg.Address)" -ForegroundColor Green
        } catch {
            Write-Host "  WARN: Could not read CRS address" -ForegroundColor Yellow
        }
        continue
    }
    $regOk = $false
    for ($attempt = 1; $attempt -le 5; $attempt++) {
        try {
            $regResult = Invoke-RestMethod -Uri "$LOCAL_CRS_URL/api/salvi/inter-cube/crs/register" -Method Post -ContentType "application/json" -Body (@{ publicKey = $cfg.PublicKey; endpoint = $cfg.Endpoint } | ConvertTo-Json) -TimeoutSec 15 -ErrorAction Stop
            $regOk = $true
            $cfg.Address = $regResult.address
            break
        } catch {
            Write-Host "  Node #$($cfg.Id) registration attempt $attempt failed -- retrying in 3s..."
            Start-Sleep -Seconds 3
        }
    }
    if ($regOk) {
        Write-Host "  [OK] Node #$($cfg.Id) registered -> address: $($cfg.Address)" -ForegroundColor Green
        $registeredAddresses += $cfg.Address
    } else {
        Write-Host "  WARN: Node #$($cfg.Id) registration with coordinator failed" -ForegroundColor Yellow
    }
}

# ── 8. Post deployment summary to remote CRS + create launcher ───────────
Write-Host ""
Write-Host "STEP 8/8: Deployment summary + desktop launcher" -ForegroundColor Yellow
Write-Host "---"

$hostname = $env:COMPUTERNAME

$daemonsArray = @()
foreach ($cfg in $daemonConfigs) {
    $daemonsArray += @{
        id = $cfg.Id
        port = $cfg.Port
        address = if ($cfg.Address) { $cfg.Address } else { "" }
        publicKey = if ($cfg.PublicKey) { $cfg.PublicKey } else { "" }
        endpoint = $cfg.Endpoint
        identityDir = $cfg.IdentityDir
        mode = $cfg.Mode
        pid = $daemonPids[$cfg.Id - 1]
    }
}

$deploymentPayload = @{
    hostname = $hostname
    ip = $ip
    architecture = $cpuArch
    daemonCount = $DAEMON_COUNT
    daemons = $daemonsArray
    localCrsUrl = $LOCAL_CRS_URL
    localCrsPort = $LOCAL_CRS_PORT
    crsUrl = $REMOTE_CRS
    binaryPath = $BinaryPath
    binarySizeMB = $fileSizeMB
    logDir = $LOG_DIR
    identityBase = $IdentityBase
    localVersion = $localVersion
    timestamp = (Get-Date -Format "o")
    deployer = "deploy-yoda/v0.4.0"
} | ConvertTo-Json -Depth 3

try {
    $notifyCrs = Invoke-RestMethod -Uri "$REMOTE_CRS/api/salvi/inter-cube/relay/deployment" -Method Post -Body $deploymentPayload -ContentType "application/json" -TimeoutSec 15 -ErrorAction Stop
    Write-Host "  [OK] Deployment summary posted to PlenumNET Node Registry" -ForegroundColor Green
    Write-Host "       Query: $REMOTE_CRS/api/salvi/inter-cube/relay/deployments" -ForegroundColor DarkGray
} catch {
    Write-Host "  WARN: Could not post deployment summary -- $_" -ForegroundColor Yellow
}

# ── Desktop launcher ─────────────────────────────────────────────────────
$startYodaPath = Join-Path ([Environment]::GetFolderPath("Desktop")) "Start PlenumNET Array3.bat"
$launchLines = @(
    "@echo off"
    "title PlenumNET Array3"
    "echo ========================================"
    "echo   PlenumNET Array3 -- Starting 3-Node Cluster"
    "echo   Node #1 = Coordinator"
    "echo ========================================"
    "echo."
    ""
    ":: Kill existing node instances"
    "taskkill /f /im inter-cube-daemon.exe >nul 2>&1"
    "timeout /t 1 /nobreak >nul"
    ""
    ":: Start Node #1 as coordinator"
    "set CUBE_MODE=crs"
    "set CUBE_API_PORT=$($crsCfg.Port)"
    "set CUBE_PEER_PORT=$($crsCfg.PeerPort)"
    "set CUBE_ENDPOINT=$($crsCfg.Endpoint)"
    "set CUBE_IDENTITY_DIR=$($crsCfg.IdentityDir)"
    "set RELAY_URL=$REMOTE_CRS"
    "set LLM_PORT=$($crsCfg.AppPort)"
    "echo Starting Node #1 as coordinator (node $($crsCfg.Port), app $($crsCfg.AppPort), peer $($crsCfg.PeerPort))..."
    "start `"`" /b `"$BinaryPath`""
    "timeout /t 5 /nobreak >nul"
    ""
)

for ($i = 1; $i -lt $DAEMON_COUNT; $i++) {
    $cfg = $daemonConfigs[$i]
    $launchLines += @(
        ":: Start Node #$($cfg.Id) (worker -- registers with coordinator)"
        "set CUBE_MODE=cube"
        "set CUBE_CRS_URL=$LOCAL_CRS_URL"
        "set CUBE_ENDPOINT=$($cfg.Endpoint)"
        "set CUBE_API_PORT=$($cfg.Port)"
        "set CUBE_PEER_PORT=$($cfg.PeerPort)"
        "set CUBE_IDENTITY_DIR=$($cfg.IdentityDir)"
        "set RELAY_URL=$REMOTE_CRS"
        "set LLM_PORT=$($cfg.AppPort)"
        "echo Starting Node #$($cfg.Id) (node $($cfg.Port), app $($cfg.AppPort), peer $($cfg.PeerPort), relay -> $REMOTE_CRS)..."
        "start `"`" /b `"$BinaryPath`""
        "timeout /t 2 /nobreak >nul"
        ""
    )
}
$launchLines += @(
    "echo."
    "echo ========================================"
    "echo   PlenumNET Array3 Running"
    "echo   Node #1 (coordinator) : http://localhost:$($crsCfg.Port)"
)
for ($i = 1; $i -lt $DAEMON_COUNT; $i++) {
    $cfg = $daemonConfigs[$i]
    $launchLines += "echo   Node #$($cfg.Id) (worker)     : http://localhost:$($cfg.Port)"
}
$launchLines += @(
    "echo ========================================"
    "echo."
    "echo Press any key to stop all nodes..."
    "pause >nul"
    ""
    "taskkill /f /im inter-cube-daemon.exe >nul 2>&1"
    "echo Nodes stopped."
    "timeout /t 2 /nobreak >nul"
)
$launchContent = $launchLines -join "`r`n"
Set-Content -Path $startYodaPath -Value $launchContent -Encoding ASCII
Write-Host ""
Write-Host "  [OK] Desktop launcher created: $startYodaPath" -ForegroundColor Green

# ── Summary ──────────────────────────────────────────────────────────────
Write-Host ""
Write-Host "==========================================================" -ForegroundColor Green
Write-Host "  PlenumNET Array3 Deployment Complete" -ForegroundColor Green
Write-Host "==========================================================" -ForegroundColor Green
Write-Host ""
Write-Host "  Node #1 (coordinator): port $($crsCfg.Port), address $($crsCfg.Address)" -ForegroundColor White
for ($i = 1; $i -lt $DAEMON_COUNT; $i++) {
    $cfg = $daemonConfigs[$i]
    Write-Host "  Node #$($cfg.Id) (worker)     : port $($cfg.Port), address $($cfg.Address)" -ForegroundColor White
}
Write-Host ""
Write-Host "  Coordinator    : $LOCAL_CRS_URL (Node #1)" -ForegroundColor White
Write-Host "  Relay          : $REMOTE_CRS (WebSocket NAT traversal)" -ForegroundColor White
Write-Host "  Registry       : $REMOTE_CRS (monitoring dashboard)" -ForegroundColor White
Write-Host "  Node Registry  : $REMOTE_CRS/api/salvi/inter-cube/relay/deployments" -ForegroundColor White
Write-Host "  Launcher       : $startYodaPath" -ForegroundColor White
Write-Host "  Logs           : $LOG_DIR" -ForegroundColor White
Write-Host ""
Write-Host "  Applications (e.g. YODA) connect via the relay to reach these nodes." -ForegroundColor DarkGray
Write-Host ""
Read-Host "Press Enter to close"
