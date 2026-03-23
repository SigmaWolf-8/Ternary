<#
.SYNOPSIS
    YODA 3-Daemon Deployer
    Builds the daemon, generates 3 PT26-DSA identities, starts 3 cube
    daemons, registers all with PlenumNET CRS, and posts a deployment
    summary to the CRS API so any consumer (YODA, dashboards, etc.)
    can query the cluster state.

.DESCRIPTION
    Served from https://plenumnet.replit.app/api/deploy-yoda
    Run with:  irm https://plenumnet.replit.app/api/deploy-yoda | iex
    Or download the .bat wrapper from the Distribution page.

    Port layout:
      Daemon #1     : 8081  (identity-1)
      Daemon #2     : 8083  (identity-2)
      Daemon #3     : 8085  (identity-3)

    LLM engines are NOT installed by this script. LLM selection and
    setup is handled separately at YODA runtime.

.NOTES
    Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
    Applied Physics Division
#>

$DAEMON_COUNT  = 3
$CRS_URL       = "https://plenumnet.replit.app"
$BASE_DAEMON_PORT = 8081
$PORT_STEP     = 2
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
Write-Host "  CRS       : $CRS_URL" -ForegroundColor White
Write-Host "  Ports     : $BASE_DAEMON_PORT, $($BASE_DAEMON_PORT + $PORT_STEP), $($BASE_DAEMON_PORT + 2 * $PORT_STEP)" -ForegroundColor White
Write-Host ""

function Test-Command($cmd) {
    try { Get-Command $cmd -ErrorAction Stop | Out-Null; return $true }
    catch { return $false }
}

# ── 1. Prerequisites ──────────────────────────────────────────────────────────
Write-Host "STEP 1/7: Checking prerequisites" -ForegroundColor Yellow
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
Write-Host "STEP 2/7: Build environment" -ForegroundColor Yellow
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
Write-Host "STEP 3/7: Source code + build" -ForegroundColor Yellow
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

# ── 4. Detect local IP ────────────────────────────────────────────────────
Write-Host ""
Write-Host "STEP 4/7: Network detection" -ForegroundColor Yellow
Write-Host "---"

$ip = (Get-NetIPAddress -AddressFamily IPv4 |
    Where-Object { $_.IPAddress -notmatch '^127\.' -and $_.IPAddress -notmatch '^169\.254' -and $_.PrefixOrigin -ne 'WellKnown' } |
    Sort-Object @{ Expression = { switch -Wildcard ($_.InterfaceAlias) { 'Wi-Fi*' { 0 } 'Ethernet*' { 1 } default { 2 } } } } |
    Select-Object -First 1).IPAddress
if (-not $ip) { $ip = "0.0.0.0" }
Write-Host "  Local IP: $ip" -ForegroundColor White

# ── 5. Generate 3 identities ─────────────────────────────────────────────
Write-Host ""
Write-Host "STEP 5/7: Generating $DAEMON_COUNT daemon identities" -ForegroundColor Yellow
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

    $daemonConfigs += @{
        Id = $i
        Port = $daemonPort
        IdentityDir = $dir
        Endpoint = $endpoint
        PublicKey = $pubKey
    }
}

# ── 6. Register all 3 with CRS ───────────────────────────────────────────
Write-Host ""
Write-Host "STEP 6/7: Registering $DAEMON_COUNT daemons with CRS" -ForegroundColor Yellow
Write-Host "---"

$registeredAddresses = @()
foreach ($cfg in $daemonConfigs) {
    $regOk = $false
    for ($attempt = 1; $attempt -le 5; $attempt++) {
        try {
            $regResult = Invoke-RestMethod -Uri "$CRS_URL/api/salvi/inter-cube/relay/register?publicKey=$($cfg.PublicKey)&endpoint=$($cfg.Endpoint)" -TimeoutSec 15 -ErrorAction Stop
            $regOk = $true
            $cfg.Address = $regResult.address
            break
        } catch {
            Write-Host "  Daemon #$($cfg.Id) attempt $attempt failed -- retrying in 3s..."
            Start-Sleep -Seconds 3
        }
    }
    if ($regOk) {
        Write-Host "  [OK] Daemon #$($cfg.Id) -> address: $($cfg.Address)" -ForegroundColor Green
        $registeredAddresses += $cfg.Address
    } else {
        Write-Host "  WARN: Daemon #$($cfg.Id) CRS registration failed" -ForegroundColor Yellow
    }
}

# ── 7. Start daemons + notify CRS ────────────────────────────────────────
Write-Host ""
Write-Host "STEP 7/7: Starting daemons + posting deployment summary" -ForegroundColor Yellow
Write-Host "---"

$daemonPids = @()
foreach ($cfg in $daemonConfigs) {
    $env:CUBE_MODE = "cube"
    $env:CUBE_CRS_URL = $CRS_URL
    $env:CUBE_ENDPOINT = $cfg.Endpoint
    $env:CUBE_API_PORT = "$($cfg.Port)"
    $env:CUBE_IDENTITY_DIR = $cfg.IdentityDir

    $outLog = Join-Path $LOG_DIR "daemon-$($cfg.Id)-out.log"
    $errLog = Join-Path $LOG_DIR "daemon-$($cfg.Id)-err.log"
    $proc = Start-Process -FilePath $BinaryPath -NoNewWindow -PassThru -RedirectStandardOutput $outLog -RedirectStandardError $errLog
    $daemonPids += $proc.Id
    Write-Host "  [OK] Daemon #$($cfg.Id) started (PID $($proc.Id), port $($cfg.Port))" -ForegroundColor Green
    Start-Sleep -Seconds 1
}

Remove-Item Env:\CUBE_MODE -ErrorAction SilentlyContinue
Remove-Item Env:\CUBE_CRS_URL -ErrorAction SilentlyContinue
Remove-Item Env:\CUBE_ENDPOINT -ErrorAction SilentlyContinue
Remove-Item Env:\CUBE_API_PORT -ErrorAction SilentlyContinue
Remove-Item Env:\CUBE_IDENTITY_DIR -ErrorAction SilentlyContinue

Start-Sleep -Seconds 3

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
        pid = $daemonPids[$cfg.Id - 1]
    }
}

$deploymentPayload = @{
    hostname = $hostname
    ip = $ip
    architecture = $cpuArch
    daemonCount = $DAEMON_COUNT
    daemons = $daemonsArray
    crsUrl = $CRS_URL
    binaryPath = $BinaryPath
    binarySizeMB = $fileSizeMB
    logDir = $LOG_DIR
    identityBase = $IdentityBase
    timestamp = (Get-Date -Format "o")
    deployer = "deploy-yoda/v0.3.0"
} | ConvertTo-Json -Depth 3

try {
    $notifyCrs = Invoke-RestMethod -Uri "$CRS_URL/api/salvi/inter-cube/relay/deployment" -Method Post -Body $deploymentPayload -ContentType "application/json" -TimeoutSec 15 -ErrorAction Stop
    Write-Host "  [OK] Deployment summary posted to CRS API" -ForegroundColor Green
    Write-Host "       Query at: $CRS_URL/api/salvi/inter-cube/relay/deployments" -ForegroundColor DarkGray
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
    "echo ========================================"
    "echo."
    ""
    ":: Kill existing daemon instances"
    "taskkill /f /im inter-cube-daemon.exe >nul 2>&1"
    "timeout /t 1 /nobreak >nul"
    ""
)
foreach ($cfg in $daemonConfigs) {
    $launchLines += @(
        ":: Start Daemon #$($cfg.Id)"
        "set CUBE_MODE=cube"
        "set CUBE_CRS_URL=$CRS_URL"
        "set CUBE_ENDPOINT=$($cfg.Endpoint)"
        "set CUBE_API_PORT=$($cfg.Port)"
        "set CUBE_IDENTITY_DIR=$($cfg.IdentityDir)"
        "echo Starting Daemon #$($cfg.Id) on port $($cfg.Port)..."
        "start `"`" /b `"$BinaryPath`""
        "timeout /t 2 /nobreak >nul"
        ""
    )
}
$launchLines += @(
    "echo."
    "echo ========================================"
    "echo   YODA Daemons Running -- 3-Node Cluster"
)
foreach ($cfg in $daemonConfigs) {
    $launchLines += "echo   Node #$($cfg.Id) : http://localhost:$($cfg.Port)"
}
$launchLines += @(
    "echo   CRS     : $CRS_URL"
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
foreach ($cfg in $daemonConfigs) {
    Write-Host "  Daemon #$($cfg.Id): port $($cfg.Port), address $($cfg.Address)" -ForegroundColor White
}
Write-Host ""
Write-Host "  CRS          : $CRS_URL" -ForegroundColor White
Write-Host "  Deployment API: $CRS_URL/api/salvi/inter-cube/relay/deployments" -ForegroundColor White
Write-Host "  Launcher     : $startYodaPath" -ForegroundColor White
Write-Host "  Logs         : $LOG_DIR" -ForegroundColor White
Write-Host ""
Write-Host "  LLM engines are configured separately at YODA runtime." -ForegroundColor DarkGray
Write-Host ""
Read-Host "Press Enter to close"
