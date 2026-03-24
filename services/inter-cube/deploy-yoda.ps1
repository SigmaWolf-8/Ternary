<#
.SYNOPSIS
    YODA 3-Daemon Deployer
    Builds the daemon binary, generates 3 PT26-DSA identities, starts a
    local 3-node Inter-Cube cluster, and posts a deployment summary to
    the PlenumNET CRS Daemon Registry for monitoring.

.DESCRIPTION
    Served from https://plenumnet.replit.app/api/deploy-yoda
    Run with:  irm https://plenumnet.replit.app/api/deploy-yoda | iex
    Or download the .bat wrapper from the Distribution page.

    Cluster topology:
      Daemon #1 (Engine A) : port 8081 / LLM 8080  — LOCAL CRS  (CUBE_MODE=crs)
      Daemon #2 (Engine B) : port 8083 / LLM 8082  — cube node  (registers with Daemon #1)
      Daemon #3 (Engine C) : port 8085 / LLM 8084  — cube node  (registers with Daemon #1)

    Daemon #1 is always the local CRS for the cluster. Daemons #2 and #3
    register with it at http://localhost:8081. The remote PlenumNET server
    (plenumnet.replit.app) only receives a deployment summary for the
    dashboard — it is NOT the CRS for local cube operations.

    All 3 daemons connect outbound to plenumnet.replit.app via WebSocket
    relay (RELAY_URL). This is the NAT-traversal tunnel through which
    YODA dispatches inference requests. Each daemon forwards inference
    to a local llama-server at 127.0.0.1:{LLM_PORT}.

    LLM engines are NOT installed by this script. LLM selection and
    setup is handled separately at YODA runtime.

.NOTES
    Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
    Applied Physics Division
#>

$DAEMON_COUNT  = 3
$REMOTE_CRS    = "https://plenumnet.replit.app"
$BASE_DAEMON_PORT = 8081
$PORT_STEP     = 2
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
Write-Host "  YODA 3-Daemon Deployer" -ForegroundColor Cyan
Write-Host "  PlenumNET Inter-Cube Infrastructure" -ForegroundColor Cyan
Write-Host "  Applied Physics Division -- Capomastro Holdings Ltd." -ForegroundColor Cyan
Write-Host "==========================================================" -ForegroundColor Cyan
Write-Host ""
Write-Host "  Daemons   : $DAEMON_COUNT instances" -ForegroundColor White
Write-Host "  Local CRS : Daemon #1 (port $LOCAL_CRS_PORT)" -ForegroundColor White
Write-Host "  Ports     : $BASE_DAEMON_PORT, $($BASE_DAEMON_PORT + $PORT_STEP), $($BASE_DAEMON_PORT + 2 * $PORT_STEP)" -ForegroundColor White
Write-Host "  LLM Ports : 8080, 8082, 8084" -ForegroundColor White
Write-Host "  Relay     : $REMOTE_CRS (WebSocket NAT traversal)" -ForegroundColor White
Write-Host "  Registry  : $REMOTE_CRS (monitoring only)" -ForegroundColor White
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
    Write-Host "  Stopping running daemon(s)..." -ForegroundColor Yellow
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

Write-Host "  Local daemon  : v$localVersion" -ForegroundColor White
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
Write-Host "STEP 6/8: Generating $DAEMON_COUNT daemon identities" -ForegroundColor Yellow
Write-Host "---"

New-Item -ItemType Directory -Force -Path $IdentityBase | Out-Null
New-Item -ItemType Directory -Force -Path $LOG_DIR | Out-Null

$daemonConfigs = @()
for ($i = 1; $i -le $DAEMON_COUNT; $i++) {
    $dir = Join-Path $IdentityBase "identity-$i"
    $keyFile = Join-Path $dir "master.key"
    $daemonPort = $BASE_DAEMON_PORT + (($i - 1) * $PORT_STEP)
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
            Write-Host "  [OK] Daemon #$i identity created" -ForegroundColor Green
        } else {
            Write-Host "  WARN: Daemon #$i keygen may have failed" -ForegroundColor Yellow
        }
    } else {
        Write-Host "  [OK] Daemon #$i identity exists" -ForegroundColor Green
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
        IdentityDir = $dir
        Endpoint = $endpoint
        PublicKey = $pubKey
        Mode = $mode
    }
}

# ── 7. Start daemons (CRS first, then cubes) + register with local CRS ──
Write-Host ""
Write-Host "STEP 7/8: Starting cluster (Daemon #1 = local CRS)" -ForegroundColor Yellow
Write-Host "---"

$daemonPids = @()

$crsCfg = $daemonConfigs[0]
$env:CUBE_MODE = "crs"
$env:CUBE_API_PORT = "$($crsCfg.Port)"
$env:CUBE_ENDPOINT = $crsCfg.Endpoint
$env:CUBE_IDENTITY_DIR = $crsCfg.IdentityDir
$env:RELAY_URL = $REMOTE_CRS
$env:LLM_PORT = "8080"

$outLog = Join-Path $LOG_DIR "daemon-1-out.log"
$errLog = Join-Path $LOG_DIR "daemon-1-err.log"
$proc = Start-Process -FilePath $BinaryPath -NoNewWindow -PassThru -RedirectStandardOutput $outLog -RedirectStandardError $errLog
$daemonPids += $proc.Id
Write-Host "  [OK] Daemon #1 started as LOCAL CRS (PID $($proc.Id), port $($crsCfg.Port), relay -> $REMOTE_CRS, LLM -> 8080)" -ForegroundColor Green

Remove-Item Env:\CUBE_MODE -ErrorAction SilentlyContinue
Remove-Item Env:\CUBE_API_PORT -ErrorAction SilentlyContinue
Remove-Item Env:\CUBE_ENDPOINT -ErrorAction SilentlyContinue
Remove-Item Env:\CUBE_IDENTITY_DIR -ErrorAction SilentlyContinue
Remove-Item Env:\RELAY_URL -ErrorAction SilentlyContinue
Remove-Item Env:\LLM_PORT -ErrorAction SilentlyContinue

Write-Host "  Waiting for local CRS to be ready..." -ForegroundColor DarkGray
$crsReady = $false
for ($w = 1; $w -le 15; $w++) {
    Start-Sleep -Seconds 2
    try {
        $healthCheck = Invoke-RestMethod -Uri "$LOCAL_CRS_URL/health" -TimeoutSec 5 -ErrorAction Stop
        if ($healthCheck.status -eq "ok") { $crsReady = $true; break }
    } catch {}
}
if ($crsReady) {
    Write-Host "  [OK] Local CRS ready at $LOCAL_CRS_URL" -ForegroundColor Green
} else {
    Write-Host "  WARN: Local CRS health check did not respond -- continuing anyway" -ForegroundColor Yellow
}

$LLM_PORTS = @(8080, 8082, 8084)
for ($i = 1; $i -lt $DAEMON_COUNT; $i++) {
    $cfg = $daemonConfigs[$i]
    $llmPort = $LLM_PORTS[$cfg.Id - 1]
    $env:CUBE_MODE = "cube"
    $env:CUBE_CRS_URL = $LOCAL_CRS_URL
    $env:CUBE_ENDPOINT = $cfg.Endpoint
    $env:CUBE_API_PORT = "$($cfg.Port)"
    $env:CUBE_IDENTITY_DIR = $cfg.IdentityDir
    $env:RELAY_URL = $REMOTE_CRS
    $env:LLM_PORT = "$llmPort"

    $outLog = Join-Path $LOG_DIR "daemon-$($cfg.Id)-out.log"
    $errLog = Join-Path $LOG_DIR "daemon-$($cfg.Id)-err.log"
    $proc = Start-Process -FilePath $BinaryPath -NoNewWindow -PassThru -RedirectStandardOutput $outLog -RedirectStandardError $errLog
    $daemonPids += $proc.Id
    Write-Host "  [OK] Daemon #$($cfg.Id) started (PID $($proc.Id), port $($cfg.Port), relay -> $REMOTE_CRS, LLM -> $llmPort)" -ForegroundColor Green
    Start-Sleep -Seconds 1
}

Remove-Item Env:\CUBE_MODE -ErrorAction SilentlyContinue
Remove-Item Env:\CUBE_CRS_URL -ErrorAction SilentlyContinue
Remove-Item Env:\CUBE_ENDPOINT -ErrorAction SilentlyContinue
Remove-Item Env:\CUBE_API_PORT -ErrorAction SilentlyContinue
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
            Write-Host "  [OK] Daemon #1 (CRS) address: $($cfg.Address)" -ForegroundColor Green
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
            Write-Host "  Daemon #$($cfg.Id) registration attempt $attempt failed -- retrying in 3s..."
            Start-Sleep -Seconds 3
        }
    }
    if ($regOk) {
        Write-Host "  [OK] Daemon #$($cfg.Id) registered -> address: $($cfg.Address)" -ForegroundColor Green
        $registeredAddresses += $cfg.Address
    } else {
        Write-Host "  WARN: Daemon #$($cfg.Id) local CRS registration failed" -ForegroundColor Yellow
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
    Write-Host "  [OK] Deployment summary posted to CRS Daemon Registry" -ForegroundColor Green
    Write-Host "       Query: $REMOTE_CRS/api/salvi/inter-cube/relay/deployments" -ForegroundColor DarkGray
} catch {
    Write-Host "  WARN: Could not post deployment summary -- $_" -ForegroundColor Yellow
}

# ── Desktop launcher ─────────────────────────────────────────────────────
$startYodaPath = Join-Path ([Environment]::GetFolderPath("Desktop")) "Start YODA Daemons.bat"
$launchLines = @(
    "@echo off"
    "title YODA -- PlenumNET 3-Node Cluster"
    "echo ========================================"
    "echo   YODA -- Starting PlenumNET 3-Node Cluster"
    "echo   Daemon #1 = Local CRS"
    "echo ========================================"
    "echo."
    ""
    ":: Kill existing daemon instances"
    "taskkill /f /im inter-cube-daemon.exe >nul 2>&1"
    "timeout /t 1 /nobreak >nul"
    ""
    ":: Start Daemon #1 as LOCAL CRS"
    "set CUBE_MODE=crs"
    "set CUBE_API_PORT=$($crsCfg.Port)"
    "set CUBE_ENDPOINT=$($crsCfg.Endpoint)"
    "set CUBE_IDENTITY_DIR=$($crsCfg.IdentityDir)"
    "set RELAY_URL=$REMOTE_CRS"
    "set LLM_PORT=8080"
    "echo Starting Daemon #1 as LOCAL CRS on port $($crsCfg.Port)..."
    "start `"`" /b `"$BinaryPath`""
    "timeout /t 5 /nobreak >nul"
    ""
)

for ($i = 1; $i -lt $DAEMON_COUNT; $i++) {
    $cfg = $daemonConfigs[$i]
    $llmPort = $LLM_PORTS[$cfg.Id - 1]
    $launchLines += @(
        ":: Start Daemon #$($cfg.Id) (registers with local CRS)"
        "set CUBE_MODE=cube"
        "set CUBE_CRS_URL=$LOCAL_CRS_URL"
        "set CUBE_ENDPOINT=$($cfg.Endpoint)"
        "set CUBE_API_PORT=$($cfg.Port)"
        "set CUBE_IDENTITY_DIR=$($cfg.IdentityDir)"
        "set RELAY_URL=$REMOTE_CRS"
        "set LLM_PORT=$llmPort"
        "echo Starting Daemon #$($cfg.Id) on port $($cfg.Port) (relay -> $REMOTE_CRS, LLM -> $llmPort)..."
        "start `"`" /b `"$BinaryPath`""
        "timeout /t 2 /nobreak >nul"
        ""
    )
}
$launchLines += @(
    "echo."
    "echo ========================================"
    "echo   YODA Daemons Running -- 3-Node Cluster"
    "echo   Daemon #1 (CRS) : http://localhost:$($crsCfg.Port)"
)
for ($i = 1; $i -lt $DAEMON_COUNT; $i++) {
    $cfg = $daemonConfigs[$i]
    $launchLines += "echo   Daemon #$($cfg.Id) (cube): http://localhost:$($cfg.Port)"
}
$launchLines += @(
    "echo ========================================"
    "echo."
    "echo Press any key to stop all daemons..."
    "pause >nul"
    ""
    "taskkill /f /im inter-cube-daemon.exe >nul 2>&1"
    "echo Daemons stopped."
    "timeout /t 2 /nobreak >nul"
)
$launchContent = $launchLines -join "`r`n"
Set-Content -Path $startYodaPath -Value $launchContent -Encoding ASCII
Write-Host ""
Write-Host "  [OK] Desktop launcher created: $startYodaPath" -ForegroundColor Green

# ── Summary ──────────────────────────────────────────────────────────────
Write-Host ""
Write-Host "==========================================================" -ForegroundColor Green
Write-Host "  YODA 3-Node Deployment Complete" -ForegroundColor Green
Write-Host "==========================================================" -ForegroundColor Green
Write-Host ""
Write-Host "  Daemon #1 (CRS) : port $($crsCfg.Port), address $($crsCfg.Address)" -ForegroundColor White
for ($i = 1; $i -lt $DAEMON_COUNT; $i++) {
    $cfg = $daemonConfigs[$i]
    Write-Host "  Daemon #$($cfg.Id) (cube): port $($cfg.Port), address $($cfg.Address)" -ForegroundColor White
}
Write-Host ""
Write-Host "  Local CRS     : $LOCAL_CRS_URL (Daemon #1)" -ForegroundColor White
Write-Host "  Relay         : $REMOTE_CRS (WebSocket NAT traversal)" -ForegroundColor White
Write-Host "  Remote Registry: $REMOTE_CRS (monitoring dashboard)" -ForegroundColor White
Write-Host "  CRS Daemon Registry: $REMOTE_CRS/api/salvi/inter-cube/relay/deployments" -ForegroundColor White
Write-Host "  Launcher      : $startYodaPath" -ForegroundColor White
Write-Host "  Logs          : $LOG_DIR" -ForegroundColor White
Write-Host ""
Write-Host "  LLM engines are configured separately at YODA runtime." -ForegroundColor DarkGray
Write-Host ""
Read-Host "Press Enter to close"
