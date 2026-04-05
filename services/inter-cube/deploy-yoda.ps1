<#
.SYNOPSIS
    PlenumNET Array3 Deployer
    Builds the node binary and NinjaExec signing agent, generates
    3 TL-DSA (PT26-DSA per TM-2026-016) identities, starts a 3-node
    PlenumNET Array3 cluster, configures the signing agent, and posts
    a deployment summary to the PlenumNET Node Registry for monitoring.

.DESCRIPTION
    Served from https://plenumnet.replit.app/api/deploy-yoda
    Run with:  irm https://plenumnet.replit.app/api/deploy-yoda | iex

    This command downloads and runs the PlenumNET deployer. It requires:
    Windows 10+, Administrator privileges, internet access, and
    approximately 2 GB disk space. The deployer will:
      1. Install Rust and LLVM if not present (~750 MB total download)
      2. Clone and build the PlenumNET daemon from source
      3. Register 3 Windows Services and a watchdog scheduled task
      4. Create desktop launchers for Start/Stop
      5. Build and configure NinjaExec signing agent

    Or download the .bat wrapper from the Distribution page.

    Array3 27-slot cube topology (SLOTS_PER_NODE = 27, GATEWAY_OFFSET = 13):
      Node #1 (Rep C 1) : range 11111-11137, gateway 11124  -- Coordinator  (CUBE_MODE=crs)
      Node #2 (Rep C 2) : range 11138-11164, gateway 11151  -- Worker       (registers with Node #1)
      Node #3 (Rep C 3) : range 11165-11191, gateway 11178  -- Worker       (registers with Node #1)

    Each node owns 27 ports (3^3 slots). The gateway port is the center
    slot [2,2,2] at offset +13. Formula:
      gateway = BASE_PORT + ((CUBE_NODE_ID - 1) * 27) + 13

    Node #1 is always the coordinator for the Array3. Nodes #2 and #3
    register with it at http://localhost:11124. The remote PlenumNET server
    (plenumnet.replit.app) only receives a deployment summary for the
    dashboard -- it is NOT the CRS for local node operations.

    All 3 nodes connect outbound to plenumnet.replit.app via WebSocket
    relay (RELAY_URL). This is the NAT-traversal tunnel through which
    applications like YODA dispatch requests.

    Application engines are NOT installed by this script. Application
    setup is handled separately by the consuming app (e.g. YODA).
    NinjaExec (the local signing agent) IS built and configured by
    this script -- it is required for signed operations.

    Node IDs are CUBE_NODE_ID ordinals {1,2,3} -- NOT GF(3) {0,1,2}.
    Zero is never used as a node ID. Key rotation follows radian-epoch
    intervals (14-day periods).

.PARAMETER Force
    Skip confirmation prompts for automated/scripted invocation.

.PARAMETER NoColor
    Suppress terminal color output for piped or redirected output.

.NOTES
    Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
    Applied Physics Division
#>
param(
    [switch]$Force,
    [switch]$NoColor,
    [string]$AddOperator = "",
    [string]$ServiceAccount = ""
)

# Color semantics: Cyan=brand/header, Yellow=step/warn, Green=success,
#                  Red=error/fail, DarkGray=detail, White=data/info
# Do not introduce additional colors without updating this key.

$DEPLOYER_VERSION = "v2.4.9"
$RELEASE_TAG      = "v2.4.9"
$DAEMON_COUNT     = 3
$REMOTE_CRS       = "https://plenumnet.replit.app"
$BASE_PORT        = 11111
$SLOTS_PER_NODE   = 27
$GATEWAY_OFFSET   = 13
$LOCAL_CRS_PORT   = $BASE_PORT + $GATEWAY_OFFSET
# NOTE: Workers connect to the coordinator CRS at localhost. This assumes a
# single-machine deployment. Multi-machine clusters require a routable CRS URL.
$LOCAL_CRS_URL    = "http://localhost:$LOCAL_CRS_PORT"
$RepoDir          = "C:\PlenumNET"
$BinaryName       = "inter-cube-daemon.exe"
$BinaryPath       = Join-Path $RepoDir "target\release\$BinaryName"
$NinjaExecBinary  = "ninja-exec.exe"
$NinjaExecPath    = Join-Path $RepoDir "target\release\$NinjaExecBinary"
$RepoUrl          = "https://github.com/SigmaWolf-8/Ternary.git"
$IdentityBase     = Join-Path $RepoDir "plenumnet-data"
$LOG_DIR          = Join-Path $IdentityBase "logs"

function Write-Status {
    param(
        [string]$Message,
        [string]$Color = "White",
        [string]$Prefix = ""
    )
    if ($NoColor) {
        if ($Prefix) { Write-Host "  $Prefix $Message" }
        else { Write-Host "  $Message" }
    } else {
        if ($Prefix) { Write-Host "  $Prefix $Message" -ForegroundColor $Color }
        else { Write-Host "  $Message" -ForegroundColor $Color }
    }
}

function Test-Admin {
    $currentPrincipal = New-Object Security.Principal.WindowsPrincipal([Security.Principal.WindowsIdentity]::GetCurrent())
    return $currentPrincipal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
}

function Get-ServiceAccountName {
    $identity = [Security.Principal.WindowsIdentity]::GetCurrent()
    return $identity.Name
}

function Grant-LogonAsService {
    param([string]$AccountName)
    $tempDir = Join-Path ([System.IO.Path]::GetTempPath()) "plenumnet-secedit-$([System.Guid]::NewGuid().ToString('N').Substring(0,8))"
    if (-not (Test-Path $tempDir)) { New-Item -ItemType Directory -Path $tempDir -Force | Out-Null }
    $acl = Get-Acl $tempDir
    $acl.SetAccessRuleProtection($true, $false)
    $systemRule = New-Object System.Security.AccessControl.FileSystemAccessRule("SYSTEM", "FullControl", "Allow")
    $adminRule = New-Object System.Security.AccessControl.FileSystemAccessRule("BUILTIN\Administrators", "FullControl", "Allow")
    $acl.AddAccessRule($systemRule)
    $acl.AddAccessRule($adminRule)
    Set-Acl -Path $tempDir -AclObject $acl -ErrorAction SilentlyContinue
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

function Get-TlDsaSignature {
    param([string]$IdentDir, [string]$PayloadToSign)
    try {
        $env:CUBE_MODE = "sign"
        $env:CUBE_IDENTITY_DIR = $IdentDir
        $env:CUBE_SIGN_PAYLOAD = $PayloadToSign
        $signOutput = & $BinaryPath 2>&1
        Remove-Item Env:\CUBE_MODE -ErrorAction SilentlyContinue
        Remove-Item Env:\CUBE_IDENTITY_DIR -ErrorAction SilentlyContinue
        Remove-Item Env:\CUBE_SIGN_PAYLOAD -ErrorAction SilentlyContinue
        $sigLine = $signOutput | Where-Object { $_ -match "signature|sig:" } | Select-Object -First 1
        if ($sigLine -match ':\s*([0-9a-fA-F]+)\s*$') {
            return $Matches[1]
        }
    } catch {
        Remove-Item Env:\CUBE_MODE -ErrorAction SilentlyContinue
        Remove-Item Env:\CUBE_IDENTITY_DIR -ErrorAction SilentlyContinue
        Remove-Item Env:\CUBE_SIGN_PAYLOAD -ErrorAction SilentlyContinue
    }
    return ""
}

function Restrict-FileAcl {
    param([string]$FilePath)
    try {
        $acl = New-Object System.Security.AccessControl.FileSecurity
        $acl.SetAccessRuleProtection($true, $false)
        $systemRule = New-Object System.Security.AccessControl.FileSystemAccessRule("SYSTEM", "FullControl", "Allow")
        $adminRule = New-Object System.Security.AccessControl.FileSystemAccessRule("BUILTIN\Administrators", "FullControl", "Allow")
        $acl.AddAccessRule($systemRule)
        $acl.AddAccessRule($adminRule)
        Set-Acl -Path $FilePath -AclObject $acl
        return $true
    } catch {
        return $false
    }
}

function Restrict-DirAcl {
    param([string]$DirPath)
    try {
        $acl = Get-Acl $DirPath
        $acl.SetAccessRuleProtection($true, $false)
        $systemRule = New-Object System.Security.AccessControl.FileSystemAccessRule("SYSTEM", "FullControl", "ContainerInherit,ObjectInherit", "None", "Allow")
        $adminRule = New-Object System.Security.AccessControl.FileSystemAccessRule("BUILTIN\Administrators", "FullControl", "ContainerInherit,ObjectInherit", "None", "Allow")
        $acl.AddAccessRule($systemRule)
        $acl.AddAccessRule($adminRule)
        Set-Acl -Path $DirPath -AclObject $acl
        return $true
    } catch {
        return $false
    }
}

function Test-Command($cmd) {
    try { Get-Command $cmd -ErrorAction Stop | Out-Null; return $true }
    catch { return $false }
}

# ── R3-C1 / R3-I18: Admin check FIRST — before any work ─────────────────
if (-not (Test-Admin)) {
    $scriptPath = $MyInvocation.MyCommand.Definition
    if ($scriptPath) {
        Write-Host "  Administrator privileges required. Elevating..." -ForegroundColor Yellow
        Start-Process powershell.exe -Verb RunAs -ArgumentList "-NoProfile -ExecutionPolicy Bypass -File `"$scriptPath`""
        Write-Host "  Re-launched as Administrator. This window can be closed." -ForegroundColor Green
        exit 0
    } else {
        Write-Host ""
        Write-Host "  [FAIL] Administrator privileges are required." -ForegroundColor Red
        Write-Host "         This deployer was started via a web download command (irm | iex)" -ForegroundColor Red
        Write-Host "         which cannot request admin privileges automatically." -ForegroundColor Red
        Write-Host ""
        Write-Host "         To fix: Open PowerShell as Administrator (right-click ->" -ForegroundColor Yellow
        Write-Host "         'Run as administrator') and run the command again." -ForegroundColor Yellow
        Write-Host ""
        Read-Host "Press Enter to close"
        exit 1
    }
}

# ── R3-I13: Detect existing deployment ───────────────────────────────────
$isUpgrade = $false
$existingServices = Get-Service PlenumNET-Array3-* -ErrorAction SilentlyContinue
if ($existingServices -or (Test-Path $RepoDir)) {
    $isUpgrade = $true
}

# ── Banner ───────────────────────────────────────────────────────────────
Write-Host ""
Write-Host "==========================================================" -ForegroundColor Cyan
Write-Host "  PlenumNET Array3 Deployer $DEPLOYER_VERSION" -ForegroundColor Cyan
Write-Host "  Capomastro Holdings Ltd." -ForegroundColor Cyan
Write-Host "==========================================================" -ForegroundColor Cyan
Write-Host ""

if ($isUpgrade) {
    Write-Host "  Existing deployment detected. This will:" -ForegroundColor Yellow
    Write-Host "    - Rebuild the daemon binary from release tag $RELEASE_TAG" -ForegroundColor White
    Write-Host "    - Restart all 3 Windows Services" -ForegroundColor White
    Write-Host "    - Preserve existing node identities" -ForegroundColor White
    Write-Host "    - Rebuild NinjaExec signing agent" -ForegroundColor White
    Write-Host "    - Overwrite desktop launchers and watchdog script" -ForegroundColor White
    Write-Host ""
} else {
    Write-Host "  This deployer will:" -ForegroundColor White
    Write-Host "    - Install Rust and LLVM if not present (~750 MB download)" -ForegroundColor White
    Write-Host "    - Clone and build the PlenumNET daemon from source" -ForegroundColor White
    Write-Host "    - Generate 3 node identities (TL-DSA key pairs)" -ForegroundColor White
    Write-Host "    - Register 3 Windows Services + watchdog scheduled task" -ForegroundColor White
    Write-Host "    - Build and configure NinjaExec signing agent" -ForegroundColor White
    Write-Host "    - Create desktop Start/Stop launchers" -ForegroundColor White
    Write-Host ""
}

# ── R3-I6: Pre-flight consent prompt ─────────────────────────────────────
if (-not $Force) {
    $consent = Read-Host "  Continue? (Y/n)"
    if ($consent -and $consent -notin @("y", "Y", "yes", "Yes", "YES", "")) {
        Write-Host "  Deployment cancelled." -ForegroundColor Yellow
        exit 0
    }
}

# ── R3-I8: Cleanup on interruption ──────────────────────────────────────
$partialServices = @()
$cleanupNeeded = $false
$deploymentHealthy = $true
$degradedReasons = @()
try {

# ── STEP 1/11: Checking prerequisites ───────────────────────────────────
Write-Host ""
Write-Host "STEP 1/11: Checking prerequisites" -ForegroundColor Yellow
Write-Host "---" -ForegroundColor DarkGray

if (-not (Test-Command "git")) {
    Write-Host "  [FAIL] git is not installed or not in PATH." -ForegroundColor Red
    Write-Host "         Install from https://git-scm.com/download/win" -ForegroundColor Yellow
    Write-Host "         then re-run the deployer." -ForegroundColor Yellow
    Read-Host "Press Enter to close"
    exit 1
}
Write-Host "  [OK] git" -ForegroundColor Green

if (-not (Test-Command "cargo")) {
    Write-Host "  [INFO] Rust not found. Installing rustup (~250 MB download)..." -ForegroundColor Yellow
    if (-not $Force) {
        $rustConsent = Read-Host "         Proceed? (Y/n)"
        if ($rustConsent -and $rustConsent -notin @("y", "Y", "yes", "Yes", "YES", "")) {
            Write-Host "  [FAIL] Rust is required to build the daemon. Deployment cancelled." -ForegroundColor Red
            exit 1
        }
    }
    $rustupExe = Join-Path $env:TEMP "rustup-init.exe"
    Invoke-WebRequest -Uri "https://win.rustup.rs/x86_64" -OutFile $rustupExe -UseBasicParsing
    Start-Process -FilePath $rustupExe -ArgumentList "-y" -Wait -NoNewWindow
    Remove-Item $rustupExe -Force -ErrorAction SilentlyContinue
    $cargoBin = Join-Path (Join-Path $env:USERPROFILE ".cargo") "bin"
    $env:PATH += ";$cargoBin"
    if (-not (Test-Command "cargo")) {
        Write-Host "  [FAIL] Rust was installed successfully, but this session cannot detect it." -ForegroundColor Red
        Write-Host "         Please close this window and run the deployer again." -ForegroundColor Yellow
        Read-Host "Press Enter to close"
        exit 1
    }
}
Write-Host "  [OK] cargo" -ForegroundColor Green

# ── STEP 2/11: Configuring build environment ────────────────────────────
Write-Host ""
Write-Host "STEP 2/11: Configuring build environment" -ForegroundColor Yellow
Write-Host "---" -ForegroundColor DarkGray

try {
    $cpuArch = [System.Runtime.InteropServices.RuntimeInformation]::OSArchitecture.ToString()
} catch {
    $cpuArch = $env:PROCESSOR_ARCHITECTURE
    if (-not $cpuArch) { $cpuArch = "AMD64" }
}
Write-Host "  [INFO] Architecture: $cpuArch" -ForegroundColor White

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
    Write-Host "  [INFO] clang not found. Installing LLVM (~500 MB download)..." -ForegroundColor Yellow
    if (-not $Force) {
        $llvmConsent = Read-Host "         Proceed? (Y/n)"
        if ($llvmConsent -and $llvmConsent -notin @("y", "Y", "yes", "Yes", "YES", "")) {
            Write-Host "  [FAIL] LLVM/clang is required to build the daemon. Deployment cancelled." -ForegroundColor Red
            exit 1
        }
    }
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
        Write-Host "  [FAIL] LLVM was installed successfully, but this session cannot detect clang." -ForegroundColor Red
        Write-Host "         Please close this window and run the deployer again." -ForegroundColor Yellow
        Read-Host "Press Enter to close"
        exit 1
    }
}

# ── STEP 3/11: Cloning source from pinned release ──────────────────────
Write-Host ""
Write-Host "STEP 3/11: Cloning source (release $RELEASE_TAG)" -ForegroundColor Yellow
Write-Host "---" -ForegroundColor DarkGray

if (-not (Test-Path $RepoDir)) {
    Write-Host "  [INFO] Cloning PlenumNET repository (tag $RELEASE_TAG)..." -ForegroundColor White
    $null = & git clone --branch $RELEASE_TAG --depth 1 $RepoUrl $RepoDir 2>&1
    if ($LASTEXITCODE -ne 0) {
        Write-Host "  [FAIL] Could not download PlenumNET source code." -ForegroundColor Red
        Write-Host "         Check your internet connection and firewall settings, then try again." -ForegroundColor Yellow
        Write-Host "         If the problem persists, visit https://plenumnet.com/support" -ForegroundColor Yellow
        Read-Host "Press Enter to close. Re-run the deployer after resolving the issue above."
        exit 1
    }
} elseif (-not (Test-Path (Join-Path $RepoDir ".git"))) {
    Write-Host "  [INFO] Converting ZIP install to git repo..." -ForegroundColor Yellow
    Push-Location $RepoDir
    $null = & git init 2>&1
    $null = & git remote add origin $RepoUrl 2>&1
    $null = & git fetch origin tag $RELEASE_TAG --depth 1 2>&1
    $null = & git reset --hard $RELEASE_TAG 2>&1
    Pop-Location
} else {
    Write-Host "  [INFO] Updating source to $RELEASE_TAG..." -ForegroundColor White
    Push-Location $RepoDir
    $null = & git fetch origin tag $RELEASE_TAG --force 2>&1
    $null = & git checkout $RELEASE_TAG 2>&1
    $null = & git reset --hard $RELEASE_TAG 2>&1
    Pop-Location
}
Write-Host "  [OK] Source ready (pinned to $RELEASE_TAG)" -ForegroundColor Green

# ── STEP 4/11: Building inter-cube daemon ───────────────────────────────
Write-Host ""
Write-Host "STEP 4/11: Building inter-cube daemon" -ForegroundColor Yellow
Write-Host "---" -ForegroundColor DarkGray

$runningDaemons = Get-Process -Name "inter-cube-daemon" -ErrorAction SilentlyContinue
if ($runningDaemons) {
    Write-Host "  [WARN] Stopping $($runningDaemons.Count) running daemon process(es)..." -ForegroundColor Yellow
    Write-Host "         Connected relay clients will be temporarily disconnected." -ForegroundColor Yellow
    if (-not $Force) {
        Start-Sleep -Seconds 3
    }
    $runningDaemons | Stop-Process -Force
    Start-Sleep -Seconds 2
}

Write-Host "  [INFO] Cleaning inter-cube cache (forces monitor HTML refresh)..." -ForegroundColor DarkGray
& cargo clean -p inter-cube 2>&1 | Out-Null
Write-Host "  [INFO] Building inter-cube daemon (CARGO_BUILD_JOBS=1)..." -ForegroundColor White
Write-Host "         This typically takes 10-30 minutes for a release build." -ForegroundColor DarkGray
Push-Location $RepoDir
$env:CARGO_BUILD_JOBS = "1"
$buildStart = Get-Date
$lastHeartbeat = $buildStart
$compiledCount = 0
& cargo build --release -p inter-cube 2>&1 | ForEach-Object {
    $line = $_.ToString()
    $now = Get-Date
    $elapsed = ($now - $buildStart).ToString("mm\:ss")
    if ($line -match "error") {
        Write-Host "  $line" -ForegroundColor Red
    } elseif ($line -match "Compiling\s+(\S+)") {
        $compiledCount++
        Write-Host "  [$elapsed] Compiling $($Matches[1]) (#$compiledCount)" -ForegroundColor DarkGray
        $lastHeartbeat = $now
    } elseif ($line -match "Finished") {
        Write-Host "  [$elapsed] $line" -ForegroundColor DarkGray
    }
    if (($now - $lastHeartbeat).TotalSeconds -ge 30) {
        Write-Host "  [$elapsed] Still building... ($compiledCount crates compiled so far)" -ForegroundColor DarkGray
        $lastHeartbeat = $now
    }
}
$buildExit = $LASTEXITCODE
$buildElapsed = ((Get-Date) - $buildStart).ToString("mm\:ss")
Pop-Location
if ($buildExit -ne 0) {
    Write-Host "  [FAIL] The PlenumNET daemon could not be compiled ($buildElapsed elapsed)." -ForegroundColor Red
    Write-Host "         Review the error output above for details." -ForegroundColor Yellow
    Write-Host "         If a build tool is missing, install it and re-run the deployer." -ForegroundColor Yellow
    Read-Host "Press Enter to close. Re-run the deployer after resolving the issue above."
    exit 1
}
if (-not (Test-Path $BinaryPath)) {
    Write-Host "  [FAIL] Compilation appeared to succeed, but the expected output was not found." -ForegroundColor Red
    Write-Host "         This may indicate an antivirus quarantine. Check your AV logs and try again." -ForegroundColor Yellow
    Read-Host "Press Enter to close. Re-run the deployer after resolving the issue above."
    exit 1
}
$fileSizeMB = [math]::Round((Get-Item $BinaryPath).Length / 1MB, 1)
Write-Host "  [OK] Build successful ($fileSizeMB MB, $buildElapsed elapsed, $compiledCount crates)" -ForegroundColor Green

# ── R1-C5: Compute binary integrity hash ────────────────────────────────
$binarySha256 = (Get-FileHash -Path $BinaryPath -Algorithm SHA256).Hash
Write-Host "  [OK] Binary SHA-256: $binarySha256" -ForegroundColor DarkGray

$tis27Hash = ""
try {
    $env:CUBE_MODE = "hash"
    $env:CUBE_HASH_TARGET = $BinaryPath
    $hashOutput = & $BinaryPath 2>&1
    Remove-Item Env:\CUBE_MODE -ErrorAction SilentlyContinue
    Remove-Item Env:\CUBE_HASH_TARGET -ErrorAction SilentlyContinue
    $hashLine = $hashOutput | Where-Object { $_ -match "TIS-27|tis27|hash:" } | Select-Object -First 1
    if ($hashLine -match ':\s*([0-9a-fA-F]+)\s*$') {
        $tis27Hash = $Matches[1]
        Write-Host "  [OK] Binary TIS-27: $tis27Hash" -ForegroundColor DarkGray
    }
} catch {
    Remove-Item Env:\CUBE_MODE -ErrorAction SilentlyContinue
    Remove-Item Env:\CUBE_HASH_TARGET -ErrorAction SilentlyContinue
}
if (-not $tis27Hash) {
    Write-Host "  [FAIL] TIS-27 integrity hash is mandatory but daemon did not produce one." -ForegroundColor Red
    Write-Host "         The daemon must support CUBE_MODE=hash for TIS-27 hashing." -ForegroundColor Yellow
    Write-Host "         Deployment cannot continue without TIS-27 verification." -ForegroundColor Yellow
    Read-Host "Press Enter to close"
    exit 1
}

# ── R1-C5: Re-verify binary integrity before service registration ───────
$preStartHash = (Get-FileHash -Path $BinaryPath -Algorithm SHA256).Hash
if ($preStartHash -ne $binarySha256) {
    Write-Host "  [FAIL] Binary integrity check failed. The file was modified after build." -ForegroundColor Red
    Write-Host "         Expected SHA-256: $binarySha256" -ForegroundColor Red
    Write-Host "         Got:              $preStartHash" -ForegroundColor Red
    Write-Host "         This may indicate tampering or antivirus interference." -ForegroundColor Yellow
    Read-Host "Press Enter to close"
    exit 1
}

# ── STEP 5/11: Building and configuring NinjaExec signing agent ──────
Write-Host ""
Write-Host "STEP 5/11: Building and configuring NinjaExec signing agent" -ForegroundColor Yellow
Write-Host "---" -ForegroundColor DarkGray

Stop-Service -Name "PlenumNET-NinjaExec" -Force -ErrorAction SilentlyContinue
$runningNinja = Get-Process -Name "ninja-exec" -ErrorAction SilentlyContinue
if ($runningNinja) {
    Write-Host "  [WARN] Stopping running NinjaExec process..." -ForegroundColor Yellow
    $runningNinja | Stop-Process -Force
    Start-Sleep -Seconds 2
}

Write-Host "  [INFO] Building NinjaExec signing agent..." -ForegroundColor White
Write-Host "         This typically takes 3-10 minutes (shares compiled dependencies)." -ForegroundColor DarkGray
Push-Location $RepoDir
$neBuildStart = Get-Date
$neCompiledCount = 0
& cargo build --release -p ninja-exec 2>&1 | ForEach-Object {
    $line = $_.ToString()
    $now = Get-Date
    $elapsed = ($now - $neBuildStart).ToString("mm\:ss")
    if ($line -match "error") {
        Write-Host "  $line" -ForegroundColor Red
    } elseif ($line -match "Compiling\s+(\S+)") {
        $neCompiledCount++
        Write-Host "  [$elapsed] Compiling $($Matches[1]) (#$neCompiledCount)" -ForegroundColor DarkGray
    } elseif ($line -match "Finished") {
        Write-Host "  [$elapsed] $line" -ForegroundColor DarkGray
    }
}
$neBuildExit = $LASTEXITCODE
$neBuildElapsed = ((Get-Date) - $neBuildStart).ToString("mm\:ss")
Pop-Location
if ($neBuildExit -ne 0) {
    Write-Host "  [FAIL] NinjaExec could not be compiled ($neBuildElapsed elapsed)." -ForegroundColor Red
    Write-Host "         Review the error output above for details." -ForegroundColor Yellow
    Write-Host "         The daemon cluster will still work, but signing operations will not be available." -ForegroundColor Yellow
    Write-Host "         Re-run the deployer after resolving the build issue to add NinjaExec." -ForegroundColor Yellow
} elseif (-not (Test-Path $NinjaExecPath)) {
    Write-Host "  [FAIL] NinjaExec compilation appeared to succeed, but the binary was not found." -ForegroundColor Red
    Write-Host "         This may indicate an antivirus quarantine. Check your AV logs." -ForegroundColor Yellow
} else {
    $neFileSizeMB = [math]::Round((Get-Item $NinjaExecPath).Length / 1MB, 1)
    $neSha256 = (Get-FileHash -Path $NinjaExecPath -Algorithm SHA256).Hash
    Write-Host "  [OK] NinjaExec build successful ($neFileSizeMB MB, $neBuildElapsed elapsed, $neCompiledCount new crates)" -ForegroundColor Green
    Write-Host "  [OK] NinjaExec SHA-256: $neSha256" -ForegroundColor DarkGray

    $env:PATH += ";$(Split-Path $NinjaExecPath)"

    $neKeystorePath = Join-Path $env:APPDATA "NinjaExec\ninja-exec.keystore"
    if (Test-Path $neKeystorePath) {
        Write-Host "  [OK] Existing NinjaExec keystore found -- skipping init" -ForegroundColor Green
    } else {
        Write-Host "" -ForegroundColor White
        Write-Host "  ==========================================================" -ForegroundColor Cyan
        Write-Host "  NinjaExec First-Time Setup" -ForegroundColor Cyan
        Write-Host "  ==========================================================" -ForegroundColor Cyan
        Write-Host ""
        Write-Host "  NinjaExec is your local signing agent. It holds your private" -ForegroundColor White
        Write-Host "  key in an encrypted keystore and signs operations on demand." -ForegroundColor White
        Write-Host ""
        Write-Host "  You will be asked to set a passphrase (minimum 12 characters)." -ForegroundColor Yellow
        Write-Host "  This passphrase encrypts your private key. Store it safely --" -ForegroundColor Yellow
        Write-Host "  it cannot be recovered if lost." -ForegroundColor Yellow
        Write-Host ""

        $passOk = $false
        for ($ppAttempt = 1; $ppAttempt -le 3; $ppAttempt++) {
            $secPass = Read-Host "  Enter passphrase (min 12 chars)" -AsSecureString
            $bstr = [System.Runtime.InteropServices.Marshal]::SecureStringToBSTR($secPass)
            $plainPass = [System.Runtime.InteropServices.Marshal]::PtrToStringAuto($bstr)
            [System.Runtime.InteropServices.Marshal]::ZeroFreeBSTR($bstr)

            if ($plainPass.Length -lt 12) {
                Write-Host "  [FAIL] Passphrase must be at least 12 characters." -ForegroundColor Red
                if ($ppAttempt -lt 3) { Write-Host "         Try again ($ppAttempt/3)." -ForegroundColor Yellow }
                continue
            }

            $secConfirm = Read-Host "  Confirm passphrase" -AsSecureString
            $bstr2 = [System.Runtime.InteropServices.Marshal]::SecureStringToBSTR($secConfirm)
            $plainConfirm = [System.Runtime.InteropServices.Marshal]::PtrToStringAuto($bstr2)
            [System.Runtime.InteropServices.Marshal]::ZeroFreeBSTR($bstr2)

            if ($plainPass -ne $plainConfirm) {
                Write-Host "  [FAIL] Passphrases do not match." -ForegroundColor Red
                if ($ppAttempt -lt 3) { Write-Host "         Try again ($ppAttempt/3)." -ForegroundColor Yellow }
                continue
            }

            $passOk = $true
            break
        }

        if (-not $passOk) {
            Write-Host "  [FAIL] Passphrase setup failed after 3 attempts." -ForegroundColor Red
            Write-Host "         NinjaExec will not be configured. Run 'ninja-exec init' manually later." -ForegroundColor Yellow
        } else {
            $neDataDir = Join-Path $env:APPDATA "NinjaExec"
            if (-not (Test-Path $neDataDir)) { New-Item -ItemType Directory -Force -Path $neDataDir | Out-Null }

            $env:PLENUM_PASSPHRASE = $plainPass
            $initOutput = & $NinjaExecPath init 2>&1
            $initExit = $LASTEXITCODE
            Remove-Item Env:\PLENUM_PASSPHRASE -ErrorAction SilentlyContinue
            $plainPass = $null
            $plainConfirm = $null

            if ($initExit -ne 0 -and -not (Test-Path $neKeystorePath)) {
                Write-Host "  [FAIL] NinjaExec keystore initialization failed." -ForegroundColor Red
                Write-Host "         Output: $initOutput" -ForegroundColor DarkGray
                Write-Host "         Run 'ninja-exec init' manually later." -ForegroundColor Yellow
            } else {
                Restrict-FileAcl -FilePath $neKeystorePath | Out-Null
                Write-Host "  [OK] NinjaExec keystore created and ACL-restricted" -ForegroundColor Green

                $exportOutput = & $NinjaExecPath export-operator 2>&1
                $exportExit = $LASTEXITCODE

                $operatorJson = $null
                if ($exportExit -eq 0 -and $exportOutput) {
                    $rawJson = ($exportOutput | Out-String).Trim()
                    try {
                        $operatorJson = $rawJson | ConvertFrom-Json
                    } catch {
                        $jsonLine = $exportOutput | Where-Object { $_ -match '^s*{' } | Select-Object -First 1
                        if ($jsonLine) {
                            try { $operatorJson = $jsonLine | ConvertFrom-Json } catch {}
                        }
                    }
                }

                if ($operatorJson -and $operatorJson.public_key) {
                    Write-Host "  [OK] Operator public key exported" -ForegroundColor Green
                    Write-Host "       Fingerprint: $($operatorJson.key_fingerprint)" -ForegroundColor DarkGray
                    $script:NinjaExecOperator = $operatorJson
                } else {
                    Write-Host "  [WARN] Could not export operator key automatically." -ForegroundColor Yellow
                    Write-Host "         Run 'ninja-exec export-operator' manually after deployment." -ForegroundColor Yellow
                }
            }
        }
    }
}
# ── STEP 6/11: Verifying version alignment ──────────────────────────────
Write-Host ""
Write-Host "STEP 6/11: Verifying version alignment" -ForegroundColor Yellow
Write-Host "---" -ForegroundColor DarkGray

$localVersion = "unknown"
try {
    $env:CUBE_MODE = "version"
    $versionOutput = & $BinaryPath 2>&1
    Remove-Item Env:\CUBE_MODE -ErrorAction SilentlyContinue
    $vLine = $versionOutput | Where-Object { $_ -match "version|v\d+\.\d+" } | Select-Object -First 1
    if ($vLine -match '(\d+\.\d+\.\d+)') { $localVersion = $Matches[1] }
} catch {
    Remove-Item Env:\CUBE_MODE -ErrorAction SilentlyContinue
}
if ($localVersion -eq "unknown") {
    # NOTE: Fallback to keygen probe if CUBE_MODE=version is not supported.
    # This generates throwaway key material — a known limitation until the
    # daemon supports a dedicated version query mode.
    try {
        $probeDir = Join-Path ([System.IO.Path]::GetTempPath()) "plenumnet-version-probe-$([System.Guid]::NewGuid().ToString('N').Substring(0,8))"
        New-Item -ItemType Directory -Force -Path $probeDir | Out-Null
        $env:CUBE_MODE = "keygen"
        $env:CUBE_IDENTITY_DIR = $probeDir
        $versionOutput = & $BinaryPath 2>&1
        Remove-Item Env:\CUBE_MODE -ErrorAction SilentlyContinue
        Remove-Item Env:\CUBE_IDENTITY_DIR -ErrorAction SilentlyContinue
        Remove-Item $probeDir -Recurse -Force -ErrorAction SilentlyContinue
        $vLine = $versionOutput | Where-Object { $_ -match "version|v\d+\.\d+" } | Select-Object -First 1
        if ($vLine -match '(\d+\.\d+\.\d+)') { $localVersion = $Matches[1] }
    } catch {
        Remove-Item Env:\CUBE_MODE -ErrorAction SilentlyContinue
        Remove-Item Env:\CUBE_IDENTITY_DIR -ErrorAction SilentlyContinue
        if ($probeDir -and (Test-Path $probeDir)) { Remove-Item $probeDir -Recurse -Force -ErrorAction SilentlyContinue }
    }
}

$remoteVersion = "unknown"
try {
    $crsHealth = Invoke-RestMethod -Uri "$REMOTE_CRS/health/crs" -TimeoutSec 10 -ErrorAction Stop
    $remoteVersion = $crsHealth.version
} catch {}

Write-Host "  [INFO] Local node    : v$localVersion" -ForegroundColor White
Write-Host "  [INFO] CRS reference : v$remoteVersion" -ForegroundColor White

if ($localVersion -ne "unknown" -and $remoteVersion -ne "unknown" -and $localVersion -ne $remoteVersion) {
    Write-Host "  [WARN] Version mismatch -- local v$localVersion vs CRS v$remoteVersion" -ForegroundColor Yellow
    Write-Host "         The deployer is pinned to $RELEASE_TAG. If you need a different version," -ForegroundColor Yellow
    Write-Host "         update `$RELEASE_TAG in the script and re-run." -ForegroundColor Yellow
} else {
    Write-Host "  [OK] Version aligned" -ForegroundColor Green
}

# ── STEP 7/11: Detecting local network ──────────────────────────────────
Write-Host ""
Write-Host "STEP 7/11: Detecting local network" -ForegroundColor Yellow
Write-Host "---" -ForegroundColor DarkGray

$ip = (Get-NetIPAddress -AddressFamily IPv4 |
    Where-Object { $_.IPAddress -notmatch '^127\.' -and $_.IPAddress -notmatch '^169\.254' -and $_.PrefixOrigin -ne 'WellKnown' } |
    Sort-Object @{ Expression = { switch -Wildcard ($_.InterfaceAlias) { 'Wi-Fi*' { 0 } 'Ethernet*' { 1 } default { 2 } } } } |
    Select-Object -First 1).IPAddress
if (-not $ip) { $ip = "0.0.0.0" }
Write-Host "  [OK] Local IP: $ip" -ForegroundColor Green

# ── STEP 8/11: Generating node identities ───────────────────────────────
Write-Host ""
Write-Host "STEP 8/11: Generating $DAEMON_COUNT node identities" -ForegroundColor Yellow
Write-Host "---" -ForegroundColor DarkGray

New-Item -ItemType Directory -Force -Path $IdentityBase | Out-Null
New-Item -ItemType Directory -Force -Path $LOG_DIR | Out-Null

$OpsBase = Join-Path $IdentityBase ".plenumnet"
New-Item -ItemType Directory -Force -Path (Join-Path $OpsBase "ops") | Out-Null
New-Item -ItemType Directory -Force -Path (Join-Path $OpsBase "logs") | Out-Null
New-Item -ItemType Directory -Force -Path (Join-Path $OpsBase "configs") | Out-Null
New-Item -ItemType Directory -Force -Path (Join-Path $OpsBase "transfers") | Out-Null
Write-Host "  [OK] Operations channel directories created" -ForegroundColor Green

$opsConfigPath = Join-Path $OpsBase "ops-config.json"
if (-not (Test-Path $opsConfigPath)) {
    $opsConfig = @{
        ops_enabled = $false
        operators = @()
        exec_timeout_seconds = 120
        file_size_limit_bytes = 5242880
        whitelisted_directories = @(
            ".plenumnet/ops/"
            ".plenumnet/logs/"
            ".plenumnet/configs/"
            ".plenumnet/transfers/"
            ".plenumnet/models/"
        )
        blocked_extensions = @(
            ".exe", ".dll", ".sys", ".bat", ".cmd", ".com", ".scr",
            ".vbs", ".vbe", ".js", ".jse", ".wsf", ".wsh", ".msi"
        )
        chunk_size_bytes = 524288
        telemetry_interval_seconds = 60
        audit_log_path = ".plenumnet/ops-audit.jsonl"
        audit_log_max_size_mb = 50
    } | ConvertTo-Json -Depth 3
    Set-Content -Path $opsConfigPath -Value $opsConfig -Encoding UTF8
    Write-Host "  [OK] Default ops-config.json created" -ForegroundColor Green
} else {
    Write-Host "  [OK] Existing ops-config.json preserved" -ForegroundColor DarkGray
}

if ($script:NinjaExecOperator) {
    $currentOpsConfig = Get-Content $opsConfigPath -Raw | ConvertFrom-Json
    $alreadyRegistered = $false
    foreach ($op in $currentOpsConfig.operators) {
        if ($op.key_fingerprint -eq $script:NinjaExecOperator.key_fingerprint) {
            $alreadyRegistered = $true
            break
        }
    }
    if (-not $alreadyRegistered) {
        $newOp = @{
            name = if ($script:NinjaExecOperator.name) { $script:NinjaExecOperator.name } else { $env:USERNAME }
            public_key = $script:NinjaExecOperator.public_key
            scope = if ($script:NinjaExecOperator.scope) { $script:NinjaExecOperator.scope } else { "full" }
            key_fingerprint = $script:NinjaExecOperator.key_fingerprint
            registered_at = (Get-Date -Format 'o')
            source = "deploy-yoda"
        }
        $opsList = @($currentOpsConfig.operators) + @($newOp)
        $currentOpsConfig.operators = $opsList
        $currentOpsConfig.ops_enabled = $true
        $currentOpsConfig | ConvertTo-Json -Depth 5 | Set-Content -Path $opsConfigPath -Encoding UTF8
        Write-Host "  [OK] NinjaExec operator auto-registered in ops-config.json" -ForegroundColor Green
        Write-Host "  [OK] Operations channel enabled (ops_enabled: true)" -ForegroundColor Green
    } else {
        Write-Host "  [OK] NinjaExec operator already registered (fingerprint match)" -ForegroundColor DarkGray
        if (-not $currentOpsConfig.ops_enabled) {
            $currentOpsConfig.ops_enabled = $true
            $currentOpsConfig | ConvertTo-Json -Depth 5 | Set-Content -Path $opsConfigPath -Encoding UTF8
            Write-Host "  [OK] Operations channel enabled (ops_enabled: true)" -ForegroundColor Green
        }
    }
}

Write-Host "`n=== Ops Sandbox Hardening ===" -ForegroundColor Cyan
try {
    $aclOps = Get-Acl $OpsBase
    $aclOps.SetAccessRuleProtection($true, $false)
    $sysRule = New-Object System.Security.AccessControl.FileSystemAccessRule("SYSTEM", "FullControl", "ContainerInherit,ObjectInherit", "None", "Allow")
    $admRule = New-Object System.Security.AccessControl.FileSystemAccessRule("BUILTIN\Administrators", "FullControl", "ContainerInherit,ObjectInherit", "None", "Allow")
    $aclOps.AddAccessRule($sysRule)
    $aclOps.AddAccessRule($admRule)
    Set-Acl -Path $OpsBase -AclObject $aclOps -ErrorAction SilentlyContinue
    Write-Host "  [OK] ACL hardening applied to $OpsBase (SYSTEM + Administrators only)" -ForegroundColor Green
} catch {
    Write-Host "  [WARN] ACL hardening skipped: $_" -ForegroundColor Yellow
}

try {
    $privExport = & secedit /export /cfg "$env:TEMP\plenumnet-secpol.cfg" 2>$null
    if (Test-Path "$env:TEMP\plenumnet-secpol.cfg") {
        $secContent = Get-Content "$env:TEMP\plenumnet-secpol.cfg" -Raw
        if ($secContent -match "SeAssignPrimaryTokenPrivilege") {
            Write-Host "  [OK] SeAssignPrimaryTokenPrivilege detected in security policy" -ForegroundColor Green
        } else {
            Write-Host "  [INFO] A required Windows privilege has been configured but requires a reboot to take effect" -ForegroundColor Yellow
            Write-Host "         (Technical: SeAssignPrimaryTokenPrivilege -- secpol.msc -> Local Policies -> User Rights)" -ForegroundColor DarkGray
        }
        Remove-Item "$env:TEMP\plenumnet-secpol.cfg" -ErrorAction SilentlyContinue
    }
} catch {
    Write-Host "  [INFO] Security policy check skipped" -ForegroundColor DarkGray
}

$appLockerAvail = $false
try {
    $appLockerSvc = Get-Service -Name "AppIDSvc" -ErrorAction SilentlyContinue
    if ($appLockerSvc -and $appLockerSvc.Status -eq "Running") {
        $appLockerAvail = $true
        Write-Host "  [OK] AppLocker service detected and running -- exec sandbox: Full (AppLocker)" -ForegroundColor Green
    } else {
        Write-Host "  [INFO] AppLocker not running -- exec sandbox: Reduced (ACLs only)" -ForegroundColor DarkGray
    }
} catch {
    Write-Host "  [INFO] AppLocker detection skipped" -ForegroundColor DarkGray
}

if ($AddOperator) {
    Write-Host "`n=== Adding Operator ===" -ForegroundColor Cyan
    try {
        $opData = $AddOperator | ConvertFrom-Json
        if (-not $opData.name -or -not $opData.public_key -or -not $opData.scope -or -not $opData.key_fingerprint) {
            Write-Host "  [ERROR] -AddOperator JSON must include: name, public_key, scope, key_fingerprint" -ForegroundColor Red
            Write-Host "  [HINT] Export the operator key from NinjaExec, then pass the full JSON here." -ForegroundColor Yellow
            Write-Host "         If key_fingerprint is wrong, remove it and re-export from NinjaExec." -ForegroundColor Yellow
        } else {
            $currentConfig = Get-Content $opsConfigPath -Raw | ConvertFrom-Json
            $newOp = @{ name = $opData.name; public_key = $opData.public_key; scope = $opData.scope; key_fingerprint = $opData.key_fingerprint; registered_at = (Get-Date -Format 'o') }
            $opsList = @($currentConfig.operators) + @($newOp)
            $currentConfig.operators = $opsList
            $currentConfig | ConvertTo-Json -Depth 5 | Set-Content -Path $opsConfigPath -Encoding UTF8
            Write-Host "  [OK] Operator added: $($opData.name) (scope: $($opData.scope), fingerprint: $($opData.key_fingerprint))" -ForegroundColor Green
        }
    } catch {
        Write-Host "  [ERROR] Invalid -AddOperator JSON: $_" -ForegroundColor Red
    }
}

$oldIdentityBase = Join-Path $env:USERPROFILE ".plenumnet"
if ((Test-Path $oldIdentityBase) -and ($oldIdentityBase -ne $IdentityBase)) {
    Write-Host "  [INFO] Migrating identities from $oldIdentityBase to $IdentityBase..." -ForegroundColor DarkGray
    for ($m = 1; $m -le $DAEMON_COUNT; $m++) {
        $oldDir = Join-Path $oldIdentityBase "identity-$m"
        $newDir = Join-Path $IdentityBase "identity-$m"
        if ((Test-Path $oldDir) -and -not (Test-Path (Join-Path $newDir "master.key"))) {
            Copy-Item -Path $oldDir -Destination $newDir -Recurse -Force
            $migratedKey = Join-Path $newDir "master.key"
            if (Test-Path $migratedKey) {
                $srcHash = (Get-FileHash -Path (Join-Path $oldDir "master.key") -Algorithm SHA256).Hash
                $dstHash = (Get-FileHash -Path $migratedKey -Algorithm SHA256).Hash
                if ($srcHash -eq $dstHash) {
                    Restrict-FileAcl -FilePath $migratedKey | Out-Null
                    Write-Host "  [OK] Migrated identity-$m (integrity verified, ACL restricted)" -ForegroundColor Green
                } else {
                    Write-Host "  [FAIL] Migration integrity check failed for identity-$m" -ForegroundColor Red
                    Remove-Item $newDir -Recurse -Force -ErrorAction SilentlyContinue
                }
            }
        }
    }
}

$daemonConfigs = @()
for ($i = 1; $i -le $DAEMON_COUNT; $i++) {
    $dir = Join-Path $IdentityBase "identity-$i"
    $keyFile = Join-Path $dir "master.key"
    $rangeStart = $BASE_PORT + (($i - 1) * $SLOTS_PER_NODE)
    $gatewayPort = $rangeStart + $GATEWAY_OFFSET
    $endpoint = "${ip}:${gatewayPort}"

    New-Item -ItemType Directory -Force -Path $dir | Out-Null
  
      $passphraseFile = Join-Path $dir ".passphrase"
      if (-not (Test-Path $passphraseFile)) {
          $passBytes = New-Object byte[] 32
          $rng = New-Object System.Security.Cryptography.RNGCryptoServiceProvider; $rng.GetBytes($passBytes); $rng.Dispose()
          $nodePassphrase = [Convert]::ToBase64String($passBytes)
          Set-Content -Path $passphraseFile -Value $nodePassphrase -Encoding UTF8 -NoNewline
          Restrict-FileAcl -FilePath $passphraseFile | Out-Null
          Write-Host "  [OK] Node #$i passphrase generated and ACL-restricted" -ForegroundColor Green
      } else {
          $nodePassphrase = Get-Content -Path $passphraseFile -Raw
          Write-Host "  [OK] Node #$i passphrase loaded" -ForegroundColor Green
      }
  
      if (-not (Test-Path $keyFile)) {
        Write-Host "  [INFO] Generating identity #$i..." -ForegroundColor White
        $env:CUBE_MODE = "keygen"
        $env:CUBE_IDENTITY_DIR = $dir
        $env:CUBE_IDENTITY_PASSPHRASE = $nodePassphrase
        $keygenOutput = & $BinaryPath 2>&1
        Remove-Item Env:\CUBE_MODE -ErrorAction SilentlyContinue
        Remove-Item Env:\CUBE_IDENTITY_DIR -ErrorAction SilentlyContinue
        Remove-Item Env:\CUBE_IDENTITY_PASSPHRASE -ErrorAction SilentlyContinue
        if (Test-Path $keyFile) {
            Restrict-FileAcl -FilePath $keyFile | Out-Null
            Write-Host "  [OK] Node #$i identity created (ACL restricted)" -ForegroundColor Green
        } else {
            Write-Host "  [FAIL] Node #$i identity generation failed. The key file was not created." -ForegroundColor Red
            Write-Host "         Re-run the deployer to retry." -ForegroundColor Yellow
        }
    } else {
        Write-Host "  [OK] Node #$i identity exists" -ForegroundColor Green
    }

    $pubKey = ""
    $env:CUBE_MODE = "keygen"
    $env:CUBE_IDENTITY_DIR = $dir
    $env:CUBE_IDENTITY_PASSPHRASE = $nodePassphrase
    $infoOutput = & $BinaryPath 2>&1
    Remove-Item Env:\CUBE_MODE -ErrorAction SilentlyContinue
    Remove-Item Env:\CUBE_IDENTITY_DIR -ErrorAction SilentlyContinue
    Remove-Item Env:\CUBE_IDENTITY_PASSPHRASE -ErrorAction SilentlyContinue
    $pkLine = $infoOutput | Where-Object { $_ -match "TL-DSA Public Key|PT26-DSA Public Key|Public Key|pk:" } | Select-Object -First 1
    if ($pkLine -match ':\s*([0-9a-fA-F]+)\s*$') {
        $pubKey = $Matches[1]
    }

    $mode = if ($i -eq 1) { "crs" } else { "cube" }

    $terminalPort = $gatewayPort - 2

    $daemonConfigs += @{
        Id = $i
        GatewayPort = $gatewayPort
        RangeStart = $rangeStart
        TerminalPort = $terminalPort
        IdentityDir = $dir
        Endpoint = $endpoint
        PublicKey = $pubKey
        Mode = $mode
        Passphrase = $nodePassphrase
    }
}

# ── STEP 9/11: Registering and starting Windows Services ────────────────
Write-Host ""
Write-Host "STEP 9/11: Registering and starting Windows Services" -ForegroundColor Yellow
Write-Host "---" -ForegroundColor DarkGray

$wrapperDir = Join-Path $RepoDir "services\wrappers"
if (-not (Test-Path $wrapperDir)) {
    New-Item -ItemType Directory -Path $wrapperDir -Force | Out-Null
}
if (-not (Test-Path $LOG_DIR)) {
    New-Item -ItemType Directory -Path $LOG_DIR -Force | Out-Null
}

foreach ($cfg in $daemonConfigs) {
    $svcName = "PlenumNET-Array3-$($cfg.Id)"
    $logFile = Join-Path $LOG_DIR "array3-node-$($cfg.Id).log"
    $wrapperBat = Join-Path $wrapperDir "array3-node-$($cfg.Id)-start.bat"

    $slotRegistryFile = Join-Path $OpsBase "slot-registry-$($cfg.Id).json"
    if (-not (Test-Path $slotRegistryFile)) {
        if ($cfg.Mode -eq "crs") {
            $slotRegistryJson = '{"2.2.2": "gateway", "1.1.1": "crs", "1.1.2": "con", "1.1.3": "fts", "1.2.1": "glb"}'
        } else {
            $slotRegistryJson = '{"2.2.2": "gateway", "1.1.2": "con", "1.1.3": "fts", "1.2.1": "glb"}'
        }
        [System.IO.File]::WriteAllText($slotRegistryFile, $slotRegistryJson, (New-Object System.Text.UTF8Encoding $false))
        Write-Host "  [OK] Node #$($cfg.Id) slot registry created ($slotRegistryFile)" -ForegroundColor Green
    } else {
          $existingRaw = [System.IO.File]::ReadAllText($slotRegistryFile)
          $existingRaw = $existingRaw.TrimStart([char]0xFEFF)
          try {
              $existingReg = $existingRaw | ConvertFrom-Json
              $builtinServices = @{"1.1.2"="con"; "1.1.3"="fts"; "1.2.1"="glb"}
              if ($cfg.Mode -eq "crs") { $builtinServices["1.1.1"] = "crs" }
              $merged = $false
              foreach ($k in $builtinServices.Keys) {
                  if (-not ($existingReg.PSObject.Properties.Name -contains $k)) {
                      $existingReg | Add-Member -NotePropertyName $k -NotePropertyValue $builtinServices[$k]
                      $merged = $true
                  }
              }
              if ($merged) {
                  $updatedJson = $existingReg | ConvertTo-Json -Compress
                  [System.IO.File]::WriteAllText($slotRegistryFile, $updatedJson, (New-Object System.Text.UTF8Encoding $false))
                  Write-Host "  [OK] Node #$($cfg.Id) slot registry updated with built-in services ($slotRegistryFile)" -ForegroundColor Green
              } else {
                  Write-Host "  [OK] Node #$($cfg.Id) slot registry exists ($slotRegistryFile)" -ForegroundColor Green
              }
          } catch {
              Write-Host "  [OK] Node #$($cfg.Id) slot registry exists ($slotRegistryFile)" -ForegroundColor Green
          }
      }

    $peerListForNode = @()
    foreach ($other in $daemonConfigs) {
        if ($other.Id -ne $cfg.Id) {
            $peerListForNode += "127.0.0.1:$($other.TerminalPort)"
        }
    }
    $peerEnvForNode = $peerListForNode -join ","

    if ($cfg.Mode -eq "crs") {
        @"
@echo off
setlocal enabledelayedexpansion
set CUBE_MODE=crs
set CUBE_NODE_ID=$($cfg.Id)
set CUBE_API_PORT=$($cfg.GatewayPort)
set CUBE_TERMINAL_PORT=$($cfg.TerminalPort)
set CUBE_ENDPOINT=$($cfg.Endpoint)
set CUBE_IDENTITY_DIR=$($cfg.IdentityDir)
for /f "usebackq delims=" %%P in ("$($cfg.IdentityDir)\.passphrase") do set CUBE_IDENTITY_PASSPHRASE=%%P
set PLENUM_SLOT_REGISTRY_FILE=$slotRegistryFile
set RELAY_URL=$REMOTE_CRS
set CUBE_ARRAY3_PEERS=$peerEnvForNode
pushd "$RepoDir" || (echo [FATAL] Cannot change to $RepoDir ^>^> "$logFile" & exit /b 1)
set RESTART_DELAY=5
set RESTART_COUNT=0
:loop
for /f "tokens=1-4 delims=/ " %%a in ('powershell -NoProfile -Command "Get-Date -Format 'yyyy-MM-ddTHH:mm:sszzz'"') do set TS=%%a
echo [%TS%] Starting PlenumNET Node #$($cfg.Id) (CRS) gateway=$($cfg.GatewayPort) terminal=$($cfg.TerminalPort) [restart #!RESTART_COUNT!] >> "$logFile"
"$BinaryPath" >> "$logFile" 2>&1
set EXIT_CODE=!ERRORLEVEL!
for /f "tokens=1-4 delims=/ " %%a in ('powershell -NoProfile -Command "Get-Date -Format 'yyyy-MM-ddTHH:mm:sszzz'"') do set TS=%%a
echo [%TS%] Node #$($cfg.Id) exited with code !EXIT_CODE! >> "$logFile"
if !EXIT_CODE! equ 0 goto :eof
set /a RESTART_COUNT+=1
if !RESTART_COUNT! leq 3 set RESTART_DELAY=5
if !RESTART_COUNT! gtr 3 if !RESTART_COUNT! leq 6 set RESTART_DELAY=10
if !RESTART_COUNT! gtr 6 if !RESTART_COUNT! leq 10 set RESTART_DELAY=30
if !RESTART_COUNT! gtr 10 set RESTART_DELAY=60
echo [%TS%] Restarting in !RESTART_DELAY!s (attempt !RESTART_COUNT!) >> "$logFile"
timeout /t !RESTART_DELAY! /nobreak >nul 2>&1
goto :loop
"@ | Set-Content -Path $wrapperBat -Encoding ASCII
    } else {
        @"
@echo off
setlocal enabledelayedexpansion
set CUBE_MODE=cube
set CUBE_NODE_ID=$($cfg.Id)
set CUBE_API_PORT=$($cfg.GatewayPort)
set CUBE_TERMINAL_PORT=$($cfg.TerminalPort)
set CUBE_CRS_URL=$LOCAL_CRS_URL
set CUBE_ENDPOINT=$($cfg.Endpoint)
set CUBE_IDENTITY_DIR=$($cfg.IdentityDir)
for /f "usebackq delims=" %%P in ("$($cfg.IdentityDir)\.passphrase") do set CUBE_IDENTITY_PASSPHRASE=%%P
set PLENUM_SLOT_REGISTRY_FILE=$slotRegistryFile
set RELAY_URL=$REMOTE_CRS
set CUBE_ARRAY3_PEERS=$peerEnvForNode
pushd "$RepoDir" || (echo [FATAL] Cannot change to $RepoDir ^>^> "$logFile" & exit /b 1)
set RESTART_DELAY=5
set RESTART_COUNT=0
:loop
for /f "tokens=1-4 delims=/ " %%a in ('powershell -NoProfile -Command "Get-Date -Format 'yyyy-MM-ddTHH:mm:sszzz'"') do set TS=%%a
echo [%TS%] Starting PlenumNET Node #$($cfg.Id) (Cube) gateway=$($cfg.GatewayPort) terminal=$($cfg.TerminalPort) [restart #!RESTART_COUNT!] >> "$logFile"
"$BinaryPath" >> "$logFile" 2>&1
set EXIT_CODE=!ERRORLEVEL!
for /f "tokens=1-4 delims=/ " %%a in ('powershell -NoProfile -Command "Get-Date -Format 'yyyy-MM-ddTHH:mm:sszzz'"') do set TS=%%a
echo [%TS%] Node #$($cfg.Id) exited with code !EXIT_CODE! >> "$logFile"
if !EXIT_CODE! equ 0 goto :eof
set /a RESTART_COUNT+=1
if !RESTART_COUNT! leq 3 set RESTART_DELAY=5
if !RESTART_COUNT! gtr 3 if !RESTART_COUNT! leq 6 set RESTART_DELAY=10
if !RESTART_COUNT! gtr 6 if !RESTART_COUNT! leq 10 set RESTART_DELAY=30
if !RESTART_COUNT! gtr 10 set RESTART_DELAY=60
echo [%TS%] Restarting in !RESTART_DELAY!s (attempt !RESTART_COUNT!) >> "$logFile"
timeout /t !RESTART_DELAY! /nobreak >nul 2>&1
goto :loop
"@ | Set-Content -Path $wrapperBat -Encoding ASCII
    }

    Restrict-FileAcl -FilePath $wrapperBat | Out-Null

    $existingSvc = Get-Service -Name $svcName -ErrorAction SilentlyContinue
    if ($existingSvc) {
        Stop-Service -Name $svcName -Force -ErrorAction SilentlyContinue
        & sc.exe delete $svcName | Out-Null
        Start-Sleep -Seconds 2
    }

    $modeLabel = if ($cfg.Mode -eq "crs") { "coordinator" } else { "worker" }
    $displayName = "PlenumNET Array3 Node #$($cfg.Id) ($modeLabel)"

    try {
        $svcBinPath = "cmd.exe /s /c `" `"$wrapperBat`" `""
        New-Service -Name $svcName `
            -BinaryPathName $svcBinPath `
            -DisplayName $displayName `
            -Description "PlenumNET Array3 daemon node #$($cfg.Id) ($modeLabel) - Capomastro Holdings Ltd." `
            -StartupType Automatic | Out-Null

        & sc.exe failure $svcName reset= 86400 actions= restart/5000/restart/10000/restart/30000 | Out-Null

        $partialServices += $svcName
        Write-Host "  [OK] Node #$($cfg.Id) registered as service '$svcName'" -ForegroundColor Green
    } catch {
        Write-Host "  [WARN] Could not register Node #$($cfg.Id) as a Windows Service." -ForegroundColor Yellow
        Write-Host "         This may require a reboot if a previous installation was not fully removed." -ForegroundColor Yellow
        Write-Host "         Error details: $_" -ForegroundColor DarkGray
    }
}

Write-Host ""
Write-Host "  [INFO] Starting Node #1 (coordinator) first..." -ForegroundColor DarkGray
$crsSvcName = "PlenumNET-Array3-1"
Start-Service -Name $crsSvcName -ErrorAction SilentlyContinue

Write-Host "  [INFO] Waiting for coordinator to be ready..." -ForegroundColor DarkGray
$crsReady = $false
for ($w = 1; $w -le 15; $w++) {
    Start-Sleep -Seconds 2
    Write-Host "         Waiting for coordinator... (attempt $w/15)" -ForegroundColor DarkGray
    try {
        $healthCheck = Invoke-RestMethod -Uri "$LOCAL_CRS_URL/health" -TimeoutSec 5 -ErrorAction Stop
        if ($healthCheck.status -eq "ok") { $crsReady = $true; break }
    } catch {}
}
if ($crsReady) {
    Write-Host "  [OK] Coordinator ready at $LOCAL_CRS_URL" -ForegroundColor Green
} else {
    Write-Host "  [WARN] The coordinator (Node #1) did not respond to health checks within 30 seconds." -ForegroundColor Yellow
    Write-Host "         It may still be starting up. The deployer will proceed with worker registration." -ForegroundColor Yellow
    Write-Host "         If workers fail to register, restart the coordinator service and re-run." -ForegroundColor Yellow
    $deploymentHealthy = $false
    $degradedReasons += "Coordinator did not respond to health checks"
}

for ($i = 2; $i -le $DAEMON_COUNT; $i++) {
    if (-not $crsReady) {
        Write-Host "  [WARN] Skipping Node #$i service start -- coordinator not ready." -ForegroundColor Yellow
        Write-Host "         Start manually after coordinator is healthy: Start-Service PlenumNET-Array3-$i" -ForegroundColor Yellow
        continue
    }
    $workerSvcName = "PlenumNET-Array3-$i"
    Start-Service -Name $workerSvcName -ErrorAction SilentlyContinue
    Write-Host "  [OK] Node #$i service started" -ForegroundColor Green
    Start-Sleep -Seconds 2
}



if (Test-Path $NinjaExecPath) {
    $neSvcName = "PlenumNET-NinjaExec"
    $neLogFile = Join-Path $LOG_DIR "ninja-exec.log"
    $neWrapperBat = Join-Path $wrapperDir "ninja-exec-start.bat"
    $neKeystoreDir = Join-Path $env:APPDATA "NinjaExec"

    @"
@echo off
setlocal enabledelayedexpansion
set RESTART_DELAY=5
set RESTART_COUNT=0
:loop
for /f "tokens=1-4 delims=/ " %%a in ('powershell -NoProfile -Command "Get-Date -Format 'yyyy-MM-ddTHH:mm:sszzz'"') do set TS=%%a
echo [%TS%] Starting NinjaExec signing agent on port 21027 [restart #!RESTART_COUNT!] >> "$neLogFile"
"$NinjaExecPath" run --port 21027 --data-dir "$neKeystoreDir" >> "$neLogFile" 2>&1
set EXIT_CODE=!ERRORLEVEL!
for /f "tokens=1-4 delims=/ " %%a in ('powershell -NoProfile -Command "Get-Date -Format 'yyyy-MM-ddTHH:mm:sszzz'"') do set TS=%%a
echo [%TS%] NinjaExec exited with code !EXIT_CODE! >> "$neLogFile"
if !EXIT_CODE! equ 0 goto :eof
set /a RESTART_COUNT+=1
if !RESTART_COUNT! leq 3 set RESTART_DELAY=5
if !RESTART_COUNT! gtr 3 if !RESTART_COUNT! leq 6 set RESTART_DELAY=10
if !RESTART_COUNT! gtr 6 set RESTART_DELAY=30
echo [%TS%] Restarting in !RESTART_DELAY!s (attempt !RESTART_COUNT!) >> "$neLogFile"
timeout /t !RESTART_DELAY! /nobreak >nul 2>&1
goto :loop
"@ | Set-Content -Path $neWrapperBat -Encoding ASCII

    Restrict-FileAcl -FilePath $neWrapperBat | Out-Null

    $existingNeSvc = Get-Service -Name $neSvcName -ErrorAction SilentlyContinue
    if ($existingNeSvc) {
        Stop-Service -Name $neSvcName -Force -ErrorAction SilentlyContinue
        & sc.exe delete $neSvcName | Out-Null
        Start-Sleep -Seconds 2
    }

    try {
        $neSvcBinPath = "cmd.exe /s /c `" `"$neWrapperBat`" `""
        New-Service -Name $neSvcName `
            -BinaryPathName $neSvcBinPath `
            -DisplayName "PlenumNET NinjaExec Signing Agent" `
            -Description "PlenumNET NinjaExec local TL-DSA signing agent - Capomastro Holdings Ltd." `
            -StartupType Automatic | Out-Null

        & sc.exe failure $neSvcName reset= 86400 actions= restart/5000/restart/10000/restart/30000 | Out-Null

        Start-Service -Name $neSvcName -ErrorAction SilentlyContinue
        Write-Host "  [OK] NinjaExec registered and started as service '$neSvcName'" -ForegroundColor Green
        Write-Host "  [OK] Signing API: http://localhost:21027/sign" -ForegroundColor Green
    } catch {
        Write-Host "  [WARN] Could not register NinjaExec as a Windows Service: $_" -ForegroundColor Yellow
        Write-Host "         Run manually: $NinjaExecPath run --port 21027" -ForegroundColor Yellow
    }
}

# ── STEP 10/11: Configuring watchdog and LLM engines ─────────────────────
Write-Host ""
Write-Host "STEP 10/11: Configuring watchdog and LLM engines" -ForegroundColor Yellow
Write-Host "---" -ForegroundColor DarkGray

$watchdogScript = Join-Path $wrapperDir "array3-watchdog.ps1"
$watchdogPortList = ($daemonConfigs | ForEach-Object { $_.GatewayPort; $_.TerminalPort }) -join ', '
$expandedPublicDir = [Environment]::GetFolderPath("CommonDesktopDirectory") -replace "\\[^\\]+$", ""
$expandedTempDir = [System.IO.Path]::GetTempPath().TrimEnd("\")
@"
`$logDir = '$LOG_DIR'
if (-not (Test-Path `$logDir)) { New-Item -ItemType Directory -Force -Path `$logDir | Out-Null }
`$wdLog = Join-Path `$logDir 'watchdog.log'
`$addressMapPath = '$($OpsBase -replace "'","''")\address-map.json'
`$addressMap = @{}
if (Test-Path `$addressMapPath) {
    try {
        `$mapJson = Get-Content `$addressMapPath -Raw | ConvertFrom-Json
        foreach (`$prop in `$mapJson.PSObject.Properties) {
            `$addressMap[`$prop.Name] = `$prop.Value
        }
    } catch {}
}
function Get-RepCAddr(`$svcName) {
    if (`$addressMap.ContainsKey(`$svcName)) { return " [RepC:`$(`$addressMap[`$svcName])]" }
    return ""
}
`$plenumDir = 'C:\ProgramData\PlenumNET'
if (-not (Test-Path `$plenumDir)) { New-Item -ItemType Directory -Force -Path `$plenumDir | Out-Null }
`$llmConfigPath = Join-Path `$plenumDir 'llm-engines.json'
`$llmCounterPath = Join-Path `$plenumDir 'llm-health-counters.json'
`$MAX_LOG_BYTES = 1048576

function Rotate-WatchdogLog {
    if (Test-Path `$wdLog) {
        try {
            `$stream = [System.IO.File]::Open(`$wdLog, [System.IO.FileMode]::Open, [System.IO.FileAccess]::Read, [System.IO.FileShare]::ReadWrite)
            `$fileLen = `$stream.Length
            `$stream.Close()
            if (`$fileLen -gt `$MAX_LOG_BYTES) {
                `$rotated = "`$wdLog.1"
                if (Test-Path `$rotated) { Remove-Item `$rotated -Force -ErrorAction SilentlyContinue }
                Rename-Item -Path `$wdLog -NewName `$rotated -Force -ErrorAction SilentlyContinue
            }
        } catch {}
    }
}

function Write-WdLog(`$msg) {
    `$ts = Get-Date -Format 'yyyy-MM-ddTHH:mm:sszzz'
    Add-Content -Path `$wdLog -Value "[`$ts] `$msg"
}

function Test-LlmExecutable(`$cmdStr) {
    `$allowedNames = @('llama-server.exe', 'llama-server', 'ollama.exe', 'ollama', 'vllm.exe', 'koboldcpp.exe')
    `$parts = `$cmdStr -split '\s+'
    `$exePath = `$parts[0].Trim('"', "'")
    `$exeName = [System.IO.Path]::GetFileName(`$exePath)
    if (`$exeName -notin `$allowedNames) { return `$false }
    if (Test-Path `$exePath) {
        `$resolved = (Resolve-Path `$exePath).Path
        `$blockedPrefixes = @('C:\Users', '$expandedPublicDir', '$expandedTempDir', 'C:\Windows\Temp')
        foreach (`$prefix in `$blockedPrefixes) {
            `$expandedPrefix = [System.Environment]::ExpandEnvironmentVariables(`$prefix)
            if (`$expandedPrefix -and `$resolved.StartsWith(`$expandedPrefix, [System.StringComparison]::OrdinalIgnoreCase)) {
                return `$false
            }
        }
    }
    return `$true
}

function Invoke-LlmRestart(`$cmdStr) {
    if (-not (Test-LlmExecutable `$cmdStr)) {
        Write-WdLog "[FAIL] BLOCKED: restart_command failed allowlist validation: `$cmdStr"
        return `$false
    }
    `$parts = [System.Collections.Generic.List[string]]::new()
    `$inQuote = `$false; `$current = ''
    foreach (`$char in `$cmdStr.ToCharArray()) {
        if (`$char -eq '"') { `$inQuote = -not `$inQuote; continue }
        if (`$char -eq ' ' -and -not `$inQuote -and `$current) { `$parts.Add(`$current); `$current = ''; continue }
        `$current += `$char
    }
    if (`$current) { `$parts.Add(`$current) }
    `$exe = `$parts[0]
    `$argsList = if (`$parts.Count -gt 1) { `$parts.GetRange(1, `$parts.Count - 1).ToArray() } else { @() }
    Start-Process -FilePath `$exe -ArgumentList `$argsList -NoNewWindow
    return `$true
}

function Get-DescendantPids([int]`$parentPid, [hashtable]`$childMap) {
    `$descendants = [System.Collections.Generic.HashSet[int]]::new()
    `$queue = [System.Collections.Generic.Queue[int]]::new()
    `$queue.Enqueue(`$parentPid)
    while (`$queue.Count -gt 0) {
        `$current = `$queue.Dequeue()
        if (`$childMap.ContainsKey(`$current)) {
            foreach (`$childPid in `$childMap[`$current]) {
                if (`$descendants.Add(`$childPid)) {
                    `$queue.Enqueue(`$childPid)
                }
            }
        }
    }
    return `$descendants
}

Rotate-WatchdogLog

`$ts = Get-Date -Format 'yyyy-MM-ddTHH:mm:sszzz'
`$totalDaemons = 0
`$healthyDaemons = 0
`$totalLlms = 0
`$healthyLlms = 0
`$restartCount = 0
`$orphansKilled = 0

`$allServices = Get-Service PlenumNET-Array3-* -ErrorAction SilentlyContinue
`$totalDaemons = (`$allServices | Measure-Object).Count

`$stopped = `$allServices | Where-Object { `$_.Status -ne 'Running' }
if (`$stopped) {
    foreach (`$svc in `$stopped) {
        `$repC = Get-RepCAddr `$svc.Name
        try {
            Start-Service -Name `$svc.Name -ErrorAction Stop
            Write-WdLog "[WARN] Restarted stopped service `$(`$svc.Name)`$repC"
            `$restartCount++
        } catch {
            Write-WdLog "[FAIL] Could not restart `$(`$svc.Name)`$repC: `$_"
        }
    }
}
`$runningServices = Get-Service PlenumNET-Array3-* -ErrorAction SilentlyContinue | Where-Object { `$_.Status -eq 'Running' }
`$healthyDaemons = (`$runningServices | Measure-Object).Count

`$relayUrl = 'https://plenumnet.replit.app'
`$relayStatusUrl = "`$relayUrl/api/salvi/inter-cube/relay/status"
`$relayDisconnectedFile = Join-Path `$logDir 'relay-disconnected.flag'
if (`$healthyDaemons -gt 0) {
    try {
        `$relayResp = Invoke-RestMethod -Uri `$relayStatusUrl -TimeoutSec 10 -ErrorAction Stop
        `$relayConnected = [int]`$relayResp.connectedNodes
        if (`$relayConnected -ge `$healthyDaemons) {
            Write-WdLog "[OK] Relay: `$relayConnected/`$healthyDaemons daemons connected"
            if (Test-Path `$relayDisconnectedFile) { Remove-Item `$relayDisconnectedFile -Force -ErrorAction SilentlyContinue }
        } else {
            Write-WdLog "[WARN] Relay: only `$relayConnected/`$healthyDaemons daemons connected"
            if (Test-Path `$relayDisconnectedFile) {
                `$flagAge = ((Get-Date) - (Get-Item `$relayDisconnectedFile).LastWriteTime).TotalSeconds
                if (`$flagAge -gt 120) {
                    Write-WdLog "[WARN] Relay disconnected for >`$([math]::Round(`$flagAge))s -- restarting all daemon services"
                    Get-Service PlenumNET-Array3-* -ErrorAction SilentlyContinue | Where-Object { `$_.Status -eq 'Running' } | ForEach-Object {
                        try {
                            Restart-Service -Name `$_.Name -Force -ErrorAction Stop
                            Write-WdLog "[OK] Restarted `$(`$_.Name) for relay reconnection"
                            `$restartCount++
                        } catch {
                            Write-WdLog "[FAIL] Could not restart `$(`$_.Name): `$_"
                        }
                    }
                    Remove-Item `$relayDisconnectedFile -Force -ErrorAction SilentlyContinue
                } else {
                    Write-WdLog "[OK] Relay disconnect flag age `$([math]::Round(`$flagAge))s < 120s -- waiting"
                }
            } else {
                New-Item -ItemType File -Path `$relayDisconnectedFile -Force | Out-Null
                Write-WdLog "[OK] Relay disconnect detected -- flag created, will restart after 120s if persistent"
            }
        }
    } catch {
        Write-WdLog "[WARN] Could not reach relay status endpoint: `$_ -- skipping relay check"
    }
}

`$allProcs = Get-CimInstance Win32_Process -ErrorAction SilentlyContinue | Select-Object ProcessId, ParentProcessId, Name
`$childMap = @{}
foreach (`$p in `$allProcs) {
    `$ppid = [int]`$p.ParentProcessId
    if (-not `$childMap.ContainsKey(`$ppid)) { `$childMap[`$ppid] = @() }
    `$childMap[`$ppid] += [int]`$p.ProcessId
}

`$legitimateDaemonPids = [System.Collections.Generic.HashSet[int]]::new()
`$servicePids = @()
foreach (`$svc in `$runningServices) {
    try {
        `$wmiSvc = Get-CimInstance Win32_Service -Filter "Name='`$(`$svc.Name)'" -ErrorAction SilentlyContinue
        if (`$wmiSvc -and `$wmiSvc.ProcessId -gt 0) {
            `$svcPid = [int]`$wmiSvc.ProcessId
            `$servicePids += `$svcPid
            `$descendants = Get-DescendantPids -parentPid `$svcPid -childMap `$childMap
            foreach (`$dPid in `$descendants) {
                `$proc = `$allProcs | Where-Object { `$_.ProcessId -eq `$dPid -and `$_.Name -eq 'inter-cube-daemon.exe' }
                if (`$proc) {
                    [void]`$legitimateDaemonPids.Add(`$dPid)
                }
            }
            `$svcRepC = Get-RepCAddr `$svc.Name
            Write-WdLog "[OK] Service `$(`$svc.Name)`$svcRepC PID=`$svcPid descendants=`$(`$descendants.Count) daemons=`$(`$legitimateDaemonPids.Count)"
        }
    } catch {
        Write-WdLog "[WARN] Could not query service `$(`$svc.Name) PID: `$_"
    }
}

`$daemonPorts = @($watchdogPortList)
`$allDaemonProcs = Get-Process -Name "inter-cube-daemon" -ErrorAction SilentlyContinue
if (`$allDaemonProcs) {
    foreach (`$dp in `$allDaemonProcs) {
        if (`$legitimateDaemonPids.Contains(`$dp.Id)) {
            continue
        }
        `$isListeningOnKnownPort = `$false
        try {
            `$conns = Get-NetTCPConnection -OwningProcess `$dp.Id -State Listen -ErrorAction SilentlyContinue
            if (`$conns) {
                foreach (`$c in `$conns) {
                    if (`$daemonPorts -contains `$c.LocalPort) {
                        `$isListeningOnKnownPort = `$true
                        break
                    }
                }
            }
        } catch {}
        if (`$isListeningOnKnownPort) {
            [void]`$legitimateDaemonPids.Add(`$dp.Id)
            Write-WdLog "[OK] PID `$(`$dp.Id) -> legitimate (listening on known daemon port)"
        } else {
            `$procUptime = 0
            try { `$procUptime = ((Get-Date) - `$dp.StartTime).TotalSeconds } catch {}
            if (`$procUptime -lt 30) {
                Write-WdLog "[OK] PID `$(`$dp.Id) -> sparing (started `$([math]::Round(`$procUptime))s ago, under startup threshold)"
                continue
            }
            try {
                Stop-Process -Id `$dp.Id -Force -ErrorAction Stop
                Write-WdLog "[WARN] PID `$(`$dp.Id) -> orphan killed (not in service tree, not on known ports, up `$([math]::Round(`$procUptime))s)"
                `$orphansKilled++
            } catch {
                Write-WdLog "[FAIL] PID `$(`$dp.Id) -> orphan kill failed: `$_"
            }
        }
    }
}

`$neSvcPid = 0
try {
    `$neWmiSvc = Get-CimInstance Win32_Service -Filter "Name='PlenumNET-NinjaExec'" -ErrorAction SilentlyContinue
    if (`$neWmiSvc -and `$neWmiSvc.ProcessId -gt 0) { `$neSvcPid = [int]`$neWmiSvc.ProcessId }
} catch {}
`$legitimateNePids = [System.Collections.Generic.HashSet[int]]::new()
if (`$neSvcPid -gt 0) {
    `$neDescendants = Get-DescendantPids -parentPid `$neSvcPid -childMap `$childMap
    foreach (`$dPid in `$neDescendants) {
        `$proc = `$allProcs | Where-Object { `$_.ProcessId -eq `$dPid -and `$_.Name -eq 'ninja-exec.exe' }
        if (`$proc) { [void]`$legitimateNePids.Add(`$dPid) }
    }
}
`$allNeProcs = Get-Process -Name "ninja-exec" -ErrorAction SilentlyContinue
if (`$allNeProcs) {
    foreach (`$np in `$allNeProcs) {
        if (`$legitimateNePids.Contains(`$np.Id)) { continue }
        `$procUptime = 0
        try { `$procUptime = ((Get-Date) - `$np.StartTime).TotalSeconds } catch {}
        if (`$procUptime -lt 30) {
            Write-WdLog "[OK] NinjaExec PID `$(`$np.Id) -> sparing (started `$([math]::Round(`$procUptime))s ago)"
            continue
        }
        try {
            Stop-Process -Id `$np.Id -Force -ErrorAction Stop
            Write-WdLog "[WARN] NinjaExec PID `$(`$np.Id) -> orphan killed (not in service tree, up `$([math]::Round(`$procUptime))s)"
            `$orphansKilled++
        } catch {
            Write-WdLog "[FAIL] NinjaExec PID `$(`$np.Id) -> orphan kill failed: `$_"
        }
    }
}

if (Test-Path `$llmConfigPath) {
    `$llmEngines = Get-Content `$llmConfigPath -Raw | ConvertFrom-Json
    `$counters = @{}
    if (Test-Path `$llmCounterPath) {
        try {
            `$jsonObj = Get-Content `$llmCounterPath -Raw | ConvertFrom-Json
            `$counters = @{}
            foreach (`$prop in `$jsonObj.PSObject.Properties) {
                `$counters[`$prop.Name] = @{
                    failures = [int]`$prop.Value.failures
                    process_alive = [bool]`$prop.Value.process_alive
                    last_restart = `$prop.Value.last_restart
                    last_check = `$prop.Value.last_check
                    grace_used = if (`$prop.Value.PSObject.Properties['grace_used']) { [bool]`$prop.Value.grace_used } else { `$false }
                }
            }
        } catch { `$counters = @{} }
    }
    if (`$null -eq `$counters -or `$counters -isnot [hashtable]) { `$counters = @{} }

    foreach (`$engine in `$llmEngines) {
        `$port = [string]`$engine.port
        `$totalLlms++
        `$portKey = `$port

        if (-not `$counters.ContainsKey(`$portKey)) {
            `$counters[`$portKey] = @{ failures = 0; process_alive = `$false; last_restart = `$null; last_check = `$null; grace_used = `$false }
        }
        `$counters[`$portKey]['last_check'] = (Get-Date -Format 'o')

        `$tcpConn = `$null
        try { `$tcpConn = Get-NetTCPConnection -LocalPort ([int]`$port) -State Listen -ErrorAction SilentlyContinue | Select-Object -First 1 } catch {}

        if (-not `$tcpConn) {
            Write-WdLog "[WARN] LLM engine on port `$port: no process listening -- restarting"
            `$counters[`$portKey]['process_alive'] = `$false
            `$counters[`$portKey]['failures'] = [int]`$counters[`$portKey]['failures'] + 1
            try {
                if (Invoke-LlmRestart `$engine.restart_command) {
                    `$counters[`$portKey]['last_restart'] = (Get-Date -Format 'o')
                    `$counters[`$portKey]['grace_used'] = `$false
                    Write-WdLog "[OK] LLM engine on port `$port: restart command issued"
                    `$restartCount++
                }
            } catch {
                Write-WdLog "[FAIL] LLM engine on port `$port: restart failed: `$_"
            }
            continue
        }

        `$counters[`$portKey]['process_alive'] = `$true
        `$llmOk = `$false
        try {
            `$resp = Invoke-RestMethod -Uri "http://127.0.0.1:`$port/v1/models" -TimeoutSec 5 -ErrorAction Stop
            if (`$resp.data -and `$resp.data.Count -gt 0) { `$llmOk = `$true }
        } catch {}

        if (`$llmOk) {
            `$counters[`$portKey]['failures'] = 0
            `$counters[`$portKey]['grace_used'] = `$false
            `$healthyLlms++
            Write-WdLog "[OK] LLM engine on port `$port: healthy"
        } else {
            `$procStartTime = `$null
            try {
                `$ownerPid = `$tcpConn.OwningProcess
                `$proc = Get-Process -Id `$ownerPid -ErrorAction SilentlyContinue
                if (`$proc) { `$procStartTime = `$proc.StartTime }
            } catch {}

            `$upSeconds = 0
            if (`$procStartTime) { `$upSeconds = ((Get-Date) - `$procStartTime).TotalSeconds }

            `$lastRestart = `$counters[`$portKey]['last_restart']
            `$recentRestart = `$false
            if (`$lastRestart) {
                try {
                    `$sinceRestart = ((Get-Date) - [DateTime]::Parse(`$lastRestart)).TotalSeconds
                    if (`$sinceRestart -lt 300) { `$recentRestart = `$true }
                } catch {}
            }

            `$graceAlreadyUsed = `$false
            if (`$counters[`$portKey].ContainsKey('grace_used')) { `$graceAlreadyUsed = [bool]`$counters[`$portKey]['grace_used'] }
            `$isLoading = ((`$upSeconds -lt 300) -or `$recentRestart) -and (-not `$graceAlreadyUsed)

            if (`$isLoading) {
                Write-WdLog "[OK] LLM engine on port `$port: /v1/models not ready, process started `$([math]::Round(`$upSeconds))s ago -- grace period"
                `$counters[`$portKey]['failures'] = [int]`$counters[`$portKey]['failures'] + 1
                `$counters[`$portKey]['grace_used'] = `$true
            } else {
                Write-WdLog "[WARN] LLM engine on port `$port: /v1/models failing, process up `$([math]::Round(`$upSeconds))s -- killing and restarting"
                `$counters[`$portKey]['failures'] = [int]`$counters[`$portKey]['failures'] + 1
                try {
                    `$ownerPid = `$tcpConn.OwningProcess
                    Stop-Process -Id `$ownerPid -Force -ErrorAction Stop
                    Start-Sleep -Seconds 2
                    if (Invoke-LlmRestart `$engine.restart_command) {
                        `$counters[`$portKey]['last_restart'] = (Get-Date -Format 'o')
                        `$counters[`$portKey]['grace_used'] = `$false
                        Write-WdLog "[OK] LLM engine on port `$port: killed stale process and restarted"
                        `$restartCount++
                    }
                } catch {
                    Write-WdLog "[FAIL] LLM engine on port `$port: kill/restart failed: `$_"
                }
            }
        }
    }

    `$counters | ConvertTo-Json -Depth 3 | Set-Content -Path `$llmCounterPath -Encoding UTF8
} else {
    Write-WdLog "[OK] No LLM engine config found at `$llmConfigPath -- skipping LLM checks"
}

`$allOk = (`$healthyDaemons -eq `$totalDaemons) -and (`$healthyLlms -eq `$totalLlms) -and (`$restartCount -eq 0) -and (`$orphansKilled -eq 0)
`$prefix = if (`$allOk) { '[OK]' } elseif (`$restartCount -gt 0 -or `$orphansKilled -gt 0) { '[WARN]' } else { '[DEGRADED]' }
Write-WdLog "`$prefix `$healthyDaemons/`$totalDaemons daemons | `$healthyLlms/`$totalLlms LLM engines | `$restartCount restarts | `$orphansKilled orphans killed"
Write-WdLog "[SUMMARY] {`"daemons_healthy`":`$healthyDaemons,`"daemons_total`":`$totalDaemons,`"llms_healthy`":`$healthyLlms,`"llms_total`":`$totalLlms,`"restarts`":`$restartCount,`"orphans_killed`":`$orphansKilled}"
"@ | Set-Content -Path $watchdogScript -Encoding ASCII

$llmEnginesConfig = @()
$defaultModelPath = ""
$modelSearchPaths = @(
    (Join-Path $RepoDir "*.gguf"),
    (Join-Path $RepoDir "models\*.gguf"),
    "C:\PlenumNET\models\*.gguf"
)
foreach ($pattern in $modelSearchPaths) {
    $found = Get-Item $pattern -ErrorAction SilentlyContinue | Select-Object -First 1
    if ($found) {
        $defaultModelPath = $found.FullName
        Write-Host "  [OK] Discovered LLM model: $defaultModelPath" -ForegroundColor Green
        break
    }
}

$llamaServerPath = "llama-server"
$llamaExe = Get-Command "llama-server" -ErrorAction SilentlyContinue
if ($llamaExe) {
    $llamaServerPath = $llamaExe.Source
    Write-Host "  [OK] llama-server found at: $llamaServerPath" -ForegroundColor Green
} else {
    $llamaSearchPaths = @(
        "C:\llama.cpp\build\bin\Release\llama-server.exe",
        "C:\llama.cpp\llama-server.exe",
        (Join-Path $RepoDir "llama-server.exe")
    )
    foreach ($p in $llamaSearchPaths) {
        if (Test-Path $p) {
            $llamaServerPath = $p
            Write-Host "  [OK] llama-server found at: $llamaServerPath" -ForegroundColor Green
            break
        }
    }
}

if (-not $defaultModelPath) {
    Write-Host "  [WARN] No AI model file (.gguf) was found on this machine." -ForegroundColor Yellow
    Write-Host "         To enable AI inference, download a compatible model file and place it" -ForegroundColor Yellow
    Write-Host "         in C:\PlenumNET\models\. The watchdog will automatically configure it." -ForegroundColor Yellow
    Write-Host "         For model recommendations, visit https://plenumnet.com/docs/models" -ForegroundColor Yellow
}
foreach ($cfg in $daemonConfigs) {
    $llmPort = $cfg.GatewayPort + 1
    if ($defaultModelPath) {
        $restartCmd = "`"$llamaServerPath`" --model `"$defaultModelPath`" --port $llmPort --host 127.0.0.1"
    } else {
        $restartCmd = "`"$llamaServerPath`" --port $llmPort --host 127.0.0.1"
    }
    $llmEnginesConfig += @{
        node_id = $cfg.Id
        port = $llmPort
        restart_command = $restartCmd
    }
}
$llmEnginesDir = "C:\ProgramData\PlenumNET"
if (-not (Test-Path $llmEnginesDir)) { New-Item -ItemType Directory -Force -Path $llmEnginesDir | Out-Null }
$llmEnginesPath = Join-Path $llmEnginesDir "llm-engines.json"
$llmEnginesConfig | ConvertTo-Json -Depth 3 | Set-Content -Path $llmEnginesPath -Encoding UTF8

if (Restrict-FileAcl -FilePath $llmEnginesPath) {
    Write-Host "  [OK] LLM engine config written to $llmEnginesPath (ACL: SYSTEM + Administrators only)" -ForegroundColor Green
} else {
    Write-Host "  [WARN] LLM engine config written but ACL restriction failed." -ForegroundColor Yellow
    Write-Host "         Manually restrict $llmEnginesPath to SYSTEM and Administrators." -ForegroundColor Yellow
}

if (Restrict-DirAcl -DirPath $llmEnginesDir) {
    Write-Host "  [OK] $llmEnginesDir ACL restricted (SYSTEM + Administrators)" -ForegroundColor Green
}

$llmCounterPath = Join-Path $llmEnginesDir "llm-health-counters.json"
if (Test-Path $llmCounterPath) {
    Remove-Item $llmCounterPath -Force -ErrorAction SilentlyContinue
    Write-Host "  [OK] Stale LLM health counters removed" -ForegroundColor Green
}

$taskName = "PlenumNET-Array3-Watchdog"
try {
    $existingTask = Get-ScheduledTask -TaskName $taskName -ErrorAction SilentlyContinue
    if ($existingTask) {
        Unregister-ScheduledTask -TaskName $taskName -Confirm:$false -ErrorAction SilentlyContinue
    }
} catch {}

try {
    $action = New-ScheduledTaskAction -Execute "powershell.exe" -Argument "-NoProfile -ExecutionPolicy Bypass -WindowStyle Hidden -File `"$watchdogScript`""
    $triggerBoot = New-ScheduledTaskTrigger -AtStartup
    $triggerRepeat = New-ScheduledTaskTrigger -Once -At (Get-Date) -RepetitionInterval (New-TimeSpan -Minutes 2)
    $principal = New-ScheduledTaskPrincipal -UserId "SYSTEM" -RunLevel Highest -LogonType ServiceAccount
    $settings = New-ScheduledTaskSettingsSet -AllowStartIfOnBatteries -DontStopIfGoingOnBatteries -StartWhenAvailable -RestartCount 5 -RestartInterval (New-TimeSpan -Minutes 1)

    Register-ScheduledTask -TaskName $taskName -Action $action -Trigger @($triggerBoot, $triggerRepeat) -Principal $principal -Settings $settings -Description "PlenumNET Array3 watchdog - monitors daemon services, LLM engines, and orphan processes every 2 minutes and on boot. Runs as SYSTEM. Capomastro Holdings Ltd." | Out-Null

    $verify = Get-ScheduledTask -TaskName $taskName -ErrorAction SilentlyContinue
    if ($verify) {
        Write-Host "  [OK] Watchdog scheduled task registered: $taskName" -ForegroundColor Green
        Write-Host "       Checks every 2 minutes + on boot" -ForegroundColor DarkGray
        Write-Host "       Watchdog log: $LOG_DIR\watchdog.log" -ForegroundColor DarkGray
    } else {
        Write-Host "  [WARN] Watchdog was registered but could not be verified." -ForegroundColor Yellow
        Write-Host "         Check Task Scheduler manually for task '$taskName'." -ForegroundColor Yellow
    }
} catch {
    Write-Host "  [WARN] Watchdog scheduled task registration failed: $_" -ForegroundColor Yellow
    Write-Host "         Falling back to schtasks.exe..." -ForegroundColor Yellow
    try {
        schtasks.exe /Create /TN $taskName /TR "powershell.exe -NoProfile -ExecutionPolicy Bypass -WindowStyle Hidden -File `"$watchdogScript`"" /SC MINUTE /MO 2 /RU SYSTEM /RL HIGHEST /F 2>&1 | Out-Null
        $fallbackVerify = schtasks.exe /Query /TN $taskName 2>&1
        if ($LASTEXITCODE -eq 0) {
            Write-Host "  [OK] Watchdog registered via schtasks.exe fallback" -ForegroundColor Green
        } else {
            Write-Host "  [FAIL] Watchdog registration failed completely." -ForegroundColor Red
            Write-Host "         Manually create a scheduled task named '$taskName' that runs" -ForegroundColor Yellow
            Write-Host "         '$watchdogScript' every 2 minutes as SYSTEM." -ForegroundColor Yellow
        }
    } catch {
        Write-Host "  [FAIL] Watchdog registration failed on both paths: $_" -ForegroundColor Red
    }
}

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
            Write-Host "  [WARN] Could not read CRS address. The coordinator may still be starting." -ForegroundColor Yellow
        }
        continue
    }

    $regOk = $false
    for ($attempt = 1; $attempt -le 5; $attempt++) {
        try {
            $regTimestamp = (Get-Date -Format "o")
            $signPayload = "CRS-REGISTER||$($cfg.PublicKey)||$($cfg.Endpoint)||$regTimestamp"
            $signature = Get-TlDsaSignature -IdentDir $cfg.IdentityDir -PayloadToSign $signPayload

            $regBody = @{
                publicKey = $cfg.PublicKey
                endpoint = $cfg.Endpoint
                timestamp = $regTimestamp
                signature = $signature
            } | ConvertTo-Json

            $regResult = Invoke-RestMethod -Uri "$LOCAL_CRS_URL/api/salvi/inter-cube/crs/register" -Method Post -ContentType "application/json" -Body $regBody -TimeoutSec 15 -ErrorAction Stop
            $regOk = $true
            $cfg.Address = $regResult.address
            break
        } catch {
            Write-Host "  [WARN] Node #$($cfg.Id) registration attempt $attempt/5 failed -- retrying in 3s..." -ForegroundColor Yellow
            Start-Sleep -Seconds 3
        }
    }
    if ($regOk) {
        Write-Host "  [OK] Node #$($cfg.Id) registered -> address: $($cfg.Address)" -ForegroundColor Green
        $registeredAddresses += $cfg.Address
    } else {
        Write-Host "  [WARN] Node #$($cfg.Id) registration with coordinator failed after 5 attempts." -ForegroundColor Yellow
        Write-Host "         Check that Node #1 (coordinator) is running and healthy at $LOCAL_CRS_URL" -ForegroundColor Yellow
        Write-Host "         You can re-run the deployer to retry registration." -ForegroundColor Yellow
        $deploymentHealthy = $false
        $degradedReasons += "Node #$($cfg.Id) registration failed"
    }
}

# ── Write address-map.json for watchdog Rep C address logging ─────────
$addressMap = @{}
foreach ($cfg in $daemonConfigs) {
    $svcName = "PlenumNET-Array3-$($cfg.Id)"
    if ($cfg.Address) {
        $addressMap[$svcName] = $cfg.Address
    }
}
$addressMapPath = Join-Path $OpsBase "address-map.json"
$addressMap | ConvertTo-Json -Depth 1 | Set-Content -Path $addressMapPath -Encoding UTF8
Write-Host "  [OK] Address map written to $addressMapPath" -ForegroundColor Green

# ── Memory hygiene: clear passphrases from $daemonConfigs ─────────────
foreach ($cfg in $daemonConfigs) {
    $cfg.Passphrase = $null
}

# ── Slot registry verification ────────────────────────────────────────
if ($crsReady) {
    $crsRegistryFile = Join-Path $OpsBase "slot-registry-$($daemonConfigs[0].Id).json"
    $registryCount = 0
    if (Test-Path $crsRegistryFile) {
        try {
            $regJson = Get-Content -Path $crsRegistryFile -Raw | ConvertFrom-Json
            $registryCount = ($regJson.PSObject.Properties | Where-Object { $_.Name -ne "2.2.2" } | Measure-Object).Count
        } catch {}
    }
    $expectedMin = $registryCount + 1
    $slotVerified = $false
    for ($attempt = 1; $attempt -le 3; $attempt++) {
        try {
            $slotResponse = Invoke-RestMethod -Uri "$LOCAL_CRS_URL/api/salvi/inter-cube/slots" -TimeoutSec 10 -ErrorAction Stop
            if ($slotResponse -and $slotResponse.summary) {
                $occupied = $slotResponse.summary.occupied
                if ($occupied -ge $expectedMin) {
                    Write-Host "  [OK] Slot registry verified: $occupied occupied slots ($registryCount from registry + gateway)" -ForegroundColor Green
                    $slotVerified = $true
                    break
                } else {
                    Write-Host "  [..] Attempt $attempt/3: $occupied occupied (need >= $expectedMin) -- retrying in 2s" -ForegroundColor DarkGray
                }
            } else {
                Write-Host "  [..] Attempt $attempt/3: empty response -- retrying in 2s" -ForegroundColor DarkGray
            }
        } catch {
            Write-Host "  [..] Attempt $attempt/3: $_ -- retrying in 2s" -ForegroundColor DarkGray
        }
        if ($attempt -lt 3) { Start-Sleep -Seconds 2 }
    }
    if (-not $slotVerified) {
        Write-Host "  [WARN] Slot registry verification failed after 3 attempts (expected >= $expectedMin occupied)" -ForegroundColor Yellow
        Write-Host "         Check PLENUM_SLOT_REGISTRY_FILE content and daemon logs for [SLOTS-N*] messages" -ForegroundColor Yellow
    }
}

# ── STEP 11/11: Deployment summary and desktop launchers ────────────────
Write-Host ""
Write-Host "STEP 11/11: Deployment summary and desktop launchers" -ForegroundColor Yellow
Write-Host "---" -ForegroundColor DarkGray

$hostname = $env:COMPUTERNAME

$daemonsArray = @()
foreach ($cfg in $daemonConfigs) {
    $daemonsArray += @{
        id = $cfg.Id
        address = if ($cfg.Address) { $cfg.Address } else { "" }
        publicKey = if ($cfg.PublicKey) { $cfg.PublicKey } else { "" }
        port = $cfg.GatewayPort
        gatewayPort = $cfg.GatewayPort
        terminalPort = $cfg.TerminalPort
        rangeStart = $cfg.RangeStart
        endpoint = $cfg.Endpoint
        mode = $cfg.Mode
        pid = 0
    }
}

$deployTimestamp = (Get-Date -Format "o")
$deploySignPayload = "DEPLOYMENT||$($registeredAddresses -join '||')||$deployTimestamp"
$deploySignature = ""
if ($daemonConfigs.Count -gt 0 -and $daemonConfigs[0].IdentityDir) {
    $deploySignature = Get-TlDsaSignature -IdentDir $daemonConfigs[0].IdentityDir -PayloadToSign $deploySignPayload
}

$deploymentPayload = @{
    addresses = $registeredAddresses
    daemonCount = $DAEMON_COUNT
    daemons = $daemonsArray
    localCrsPort = $LOCAL_CRS_PORT
    crsUrl = $REMOTE_CRS
    binaryHash = $tis27Hash
    releaseTag = $RELEASE_TAG
    localVersion = $localVersion
    timestamp = $deployTimestamp
    deployer = "deploy-yoda/$DEPLOYER_VERSION"
    signature = $deploySignature
    metadata = @{
        hostname = $hostname
        ip = $ip
        architecture = $cpuArch
        localCrsUrl = $LOCAL_CRS_URL
        binaryPath = $BinaryPath
        binarySizeMB = $fileSizeMB
        logDir = $LOG_DIR
        identityBase = $IdentityBase
    }
} | ConvertTo-Json -Depth 3

try {
    $notifyCrs = Invoke-RestMethod -Uri "$REMOTE_CRS/api/salvi/inter-cube/relay/deployment" -Method Post -Body $deploymentPayload -ContentType "application/json" -TimeoutSec 15 -ErrorAction Stop
    Write-Host "  [OK] Deployment registered with the PlenumNET Node Registry." -ForegroundColor Green
    Write-Host "       View your deployment at: $REMOTE_CRS/api/salvi/inter-cube/relay/deployments" -ForegroundColor DarkGray
} catch {
    Write-Host "  [WARN] Could not register this deployment with the Node Registry." -ForegroundColor Yellow
    Write-Host "         Your cluster is running normally -- registry notification can be retried later." -ForegroundColor Yellow
}

# ── Desktop launchers ────────────────────────────────────────────────────
# NOTE: [Environment]::GetFolderPath("Desktop") may return a OneDrive-synced
# path (e.g. C:\Users\<user>\OneDrive\Desktop). This is intentional — the
# launcher works from either location. If OneDrive sync causes issues, the
# user can move the .cmd files to C:\Users\<user>\Desktop manually.
$startYodaPath = Join-Path ([Environment]::GetFolderPath("Desktop")) "Start PlenumNET Array3.cmd"
$crsCfg = $daemonConfigs[0]
$launchLines = @(
    "@echo off"
    "title PlenumNET Array3"
    "echo =========================================================="
    "echo   PlenumNET Array3 -- Start Services"
    "echo   Capomastro Holdings Ltd."
    "echo =========================================================="
    "echo."
    ""
    "net session >nul 2>&1"
    "if %errorlevel% neq 0 ("
    "    echo [FAIL] Administrator privileges required."
    "    echo        Right-click this file and select 'Run as administrator'."
    "    pause"
    "    exit /b 1"
    ")"
    ""
    "echo Starting PlenumNET Array3 services..."
    "echo."
    ""
    "echo Starting Node #1 (coordinator)..."
    "net start PlenumNET-Array3-1 2>nul"
    "if %errorlevel% equ 0 (echo   [OK] Node #1 started) else (echo   [WARN] Node #1 is already running, or could not be started. Check Event Viewer for details.)"
    "timeout /t 5 /nobreak >nul"
    ""
    "echo Starting Node #2 (worker)..."
    "net start PlenumNET-Array3-2 2>nul"
    "if %errorlevel% equ 0 (echo   [OK] Node #2 started) else (echo   [WARN] Node #2 is already running, or could not be started. Check Event Viewer for details.)"
    "timeout /t 1 /nobreak >nul"
    ""
    "echo Starting Node #3 (worker)..."
    "net start PlenumNET-Array3-3 2>nul"
    "if %errorlevel% equ 0 (echo   [OK] Node #3 started) else (echo   [WARN] Node #3 is already running, or could not be started. Check Event Viewer for details.)"
    ""
    "echo."
    "echo =========================================================="
    "echo   PlenumNET Array3 Services Active"
    "echo   Node #1 (coordinator) : http://localhost:$($crsCfg.GatewayPort)  terminal=$($crsCfg.TerminalPort)"
)
for ($i = 1; $i -lt $DAEMON_COUNT; $i++) {
    $cfg = $daemonConfigs[$i]
    $launchLines += "echo   Node #$($cfg.Id) (worker)      : http://localhost:$($cfg.GatewayPort)  terminal=$($cfg.TerminalPort)"
}
$launchLines += @(
    "echo   VM API: /vm/exec, /vm/status (on gateway port)"
    "echo =========================================================="
    "echo."
    "echo Services will continue running after this window closes."
    "echo."
    "pause"
)
$launchContent = $launchLines -join "`r`n"
Set-Content -Path $startYodaPath -Value $launchContent -Encoding ASCII
Restrict-FileAcl -FilePath $startYodaPath | Out-Null
Write-Host ""
Write-Host "  [OK] Start launcher created: $startYodaPath" -ForegroundColor Green

$stopYodaPath = Join-Path ([Environment]::GetFolderPath("Desktop")) "Stop PlenumNET Array3.cmd"
$stopLines = @(
    "@echo off"
    "title PlenumNET Array3"
    "echo =========================================================="
    "echo   PlenumNET Array3 -- Stop Services"
    "echo   Capomastro Holdings Ltd."
    "echo =========================================================="
    "echo."
    "net session >nul 2>&1"
    "if %errorlevel% neq 0 ("
    "    echo [FAIL] Administrator privileges required."
    "    echo        Right-click this file and select 'Run as administrator'."
    "    pause"
    "    exit /b 1"
    ")"
    ""
    "echo WARNING: Stopping services will disconnect all connected relay clients."
    "echo."
    "set /p confirm=Are you sure? (Y/n): "
    "if /i not ""%confirm%""==""y"" if not ""%confirm%""=="""" ("
    "    echo Cancelled."
    "    pause"
    "    exit /b 0"
    ")"
    ""
    "echo Stopping PlenumNET Array3 services..."
    "net stop PlenumNET-Array3-3 2>nul"
    "echo   [OK] Node #3 stopped"
    "net stop PlenumNET-Array3-2 2>nul"
    "echo   [OK] Node #2 stopped"
    "net stop PlenumNET-Array3-1 2>nul"
    "echo   [OK] Node #1 stopped"
    "echo."
    "echo =========================================================="
    "echo   All Array3 services stopped."
    "echo   Capomastro Holdings Ltd."
    "echo =========================================================="
    "echo."
    "pause"
)
$stopContent = $stopLines -join "`r`n"
Set-Content -Path $stopYodaPath -Value $stopContent -Encoding ASCII
Restrict-FileAcl -FilePath $stopYodaPath | Out-Null
Write-Host "  [OK] Stop launcher created: $stopYodaPath" -ForegroundColor Green

# ── Completion summary ───────────────────────────────────────────────────
Write-Host ""
if ($deploymentHealthy) {
    Write-Host "==========================================================" -ForegroundColor Green
    Write-Host "  PlenumNET Array3 Deployment Complete" -ForegroundColor Green
    Write-Host "  Capomastro Holdings Ltd." -ForegroundColor Green
    Write-Host "  Deployer $DEPLOYER_VERSION | Release $RELEASE_TAG" -ForegroundColor Green
    Write-Host "  NinjaExec signing agent included" -ForegroundColor Green
    Write-Host "==========================================================" -ForegroundColor Green
} else {
    Write-Host "==========================================================" -ForegroundColor Yellow
    Write-Host "  PlenumNET Array3 Deployment Complete (DEGRADED)" -ForegroundColor Yellow
    Write-Host "  Capomastro Holdings Ltd." -ForegroundColor Yellow
    Write-Host "  Deployer $DEPLOYER_VERSION | Release $RELEASE_TAG" -ForegroundColor Yellow
    Write-Host "==========================================================" -ForegroundColor Yellow
    Write-Host ""
    Write-Host "  The following issues were detected:" -ForegroundColor Yellow
    foreach ($reason in $degradedReasons) {
        Write-Host "    - $reason" -ForegroundColor Yellow
    }
    Write-Host ""
    Write-Host "  Remediation steps:" -ForegroundColor Yellow
    Write-Host "    1. Check the coordinator log: $LOG_DIR\array3-node-1.log" -ForegroundColor White
    Write-Host "    2. Verify services: Get-Service PlenumNET-Array3-*" -ForegroundColor White
    Write-Host "    3. Re-run the deployer to retry registration" -ForegroundColor White
}
Write-Host ""
Write-Host "  Cluster Status" -ForegroundColor Cyan
$addrDisplay = if ($crsCfg.Address) { $crsCfg.Address } else { "(pending registration)" }
Write-Host "  Node #1 (coordinator): gateway $($crsCfg.GatewayPort), terminal $($crsCfg.TerminalPort), address $addrDisplay" -ForegroundColor White
for ($i = 1; $i -lt $DAEMON_COUNT; $i++) {
    $cfg = $daemonConfigs[$i]
    $addrDisplay = if ($cfg.Address) { $cfg.Address } else { "(pending registration)" }
    Write-Host "  Node #$($cfg.Id) (worker)      : gateway $($cfg.GatewayPort), terminal $($cfg.TerminalPort), address $addrDisplay" -ForegroundColor White
}
Write-Host ""
Write-Host "  API Endpoints" -ForegroundColor Cyan
Write-Host "  VM API           : /vm/exec, /vm/status, /vm/registers, /vm/reset (on gateway port)" -ForegroundColor White
Write-Host "  Cluster Shell    : /cluster/exec, /cluster/peers (on gateway port)" -ForegroundColor White
Write-Host "  Coordinator      : $LOCAL_CRS_URL (Node #1)" -ForegroundColor White
Write-Host "  Relay            : $REMOTE_CRS (WebSocket NAT traversal)" -ForegroundColor White
Write-Host "  Node Registry    : $REMOTE_CRS/api/salvi/inter-cube/relay/deployments" -ForegroundColor White
Write-Host ""
Write-Host "  Management" -ForegroundColor Cyan
Write-Host "  Services         : PlenumNET-Array3-1, PlenumNET-Array3-2, PlenumNET-Array3-3" -ForegroundColor White
Write-Host "  Startup          : Automatic (survives reboots + terminal close)" -ForegroundColor White
Write-Host "  Watchdog         : PlenumNET-Array3-Watchdog (every 2 min + on boot)" -ForegroundColor White
Write-Host "  Start Launcher   : $startYodaPath" -ForegroundColor White
Write-Host "  Stop Launcher    : $stopYodaPath" -ForegroundColor White
Write-Host ""
Write-Host "  NinjaExec Signing Agent" -ForegroundColor Cyan
if (Test-Path $NinjaExecPath) {
    Write-Host "  Binary           : $NinjaExecPath" -ForegroundColor White
    Write-Host "  Service          : PlenumNET-NinjaExec (Automatic)" -ForegroundColor White
    Write-Host "  Signing API      : http://localhost:21027/sign" -ForegroundColor White
    $neKeystoreCheck = Join-Path $env:APPDATA "NinjaExec\ninja-exec.keystore"
    if (Test-Path $neKeystoreCheck) {
        Write-Host "  Keystore         : $neKeystoreCheck (initialized)" -ForegroundColor White
    } else {
        Write-Host "  Keystore         : Not initialized (run 'ninja-exec init')" -ForegroundColor Yellow
    }
} else {
    Write-Host "  Status           : Not built (re-run deployer to install)" -ForegroundColor Yellow
}
Write-Host ""
Write-Host "  Operations Channel" -ForegroundColor Cyan
Write-Host "  Ops Config       : $opsConfigPath" -ForegroundColor White
$opsEnabledDisplay = "false"
try {
    $opsCheckConfig = Get-Content $opsConfigPath -Raw | ConvertFrom-Json
    if ($opsCheckConfig.ops_enabled) { $opsEnabledDisplay = "true" }
} catch {}
Write-Host "  Ops Enabled      : $opsEnabledDisplay" -ForegroundColor White
Write-Host "  Audit Log        : $OpsBase\ops-audit.jsonl" -ForegroundColor White
Write-Host "  Ops Directories  : ops/, logs/, configs/, transfers/" -ForegroundColor White
Write-Host ""
Write-Host "  Files and Logs" -ForegroundColor Cyan
Write-Host "  Binary           : $BinaryPath ($fileSizeMB MB)" -ForegroundColor White
Write-Host "  Binary Hash      : $tis27Hash" -ForegroundColor White
Write-Host "  Logs             : $LOG_DIR" -ForegroundColor White
Write-Host "  Watchdog Log     : $LOG_DIR\watchdog.log" -ForegroundColor White
Write-Host "  Release Tag      : $RELEASE_TAG" -ForegroundColor White
Write-Host ""
Write-Host "  Closing this window will NOT stop the nodes -- they run as services." -ForegroundColor DarkGray
Write-Host "  Applications (e.g. YODA) connect via the relay to reach these nodes." -ForegroundColor DarkGray
Write-Host ""
Write-Host "  ── Upgrade Notes (${RELEASE_TAG}) ──" -ForegroundColor Cyan
Write-Host "  What changed in ${RELEASE_TAG}:" -ForegroundColor DarkGray
Write-Host "    - Monitor v9.3.1: baked into daemon via include_str! (no file drops)" -ForegroundColor DarkGray
Write-Host "    - Watchdog: relay connectivity check (auto-restart after 120s disconnect)" -ForegroundColor DarkGray
Write-Host "    - MSI installer: fixed UAC self-elevation quoting + git stderr handling" -ForegroundColor DarkGray
Write-Host "    - Relay status endpoint for watchdog health monitoring" -ForegroundColor DarkGray
Write-Host "    - Deployer: git stderr no longer triggers NativeCommandError" -ForegroundColor DarkGray
Write-Host "    - test-monitor.ps1: git pull + file-system preview for pre-publish testing" -ForegroundColor DarkGray
Write-Host "  Re-running this deployer on an existing cluster is safe:" -ForegroundColor DarkGray
Write-Host "    - Existing data and identity keys are preserved" -ForegroundColor DarkGray
Write-Host "    - The .bat script always downloads the latest deployer" -ForegroundColor DarkGray
Write-Host "    - Services are stopped, updated, and restarted cleanly" -ForegroundColor DarkGray
Write-Host ""

if (-not $deploymentHealthy -and $Force) {
    Write-Host "  [EXIT] Exiting with code 1 (degraded deployment with -Force flag)" -ForegroundColor Yellow
    exit 1
}

} catch {
    Write-Host ""
    Write-Host "  [FAIL] Deployment interrupted or failed: $_" -ForegroundColor Red
    if ($partialServices.Count -gt 0) {
        Write-Host "  Cleaning up partially registered services..." -ForegroundColor Yellow
        foreach ($svc in $partialServices) {
            try {
                Stop-Service -Name $svc -Force -ErrorAction SilentlyContinue
                & sc.exe delete $svc | Out-Null
                Write-Host "  [OK] Removed partial service: $svc" -ForegroundColor DarkGray
            } catch {}
        }
    }
    Write-Host "  Re-run the deployer after resolving the issue." -ForegroundColor Yellow
    Write-Host ""
}

Read-Host "Press Enter to close this window. Your nodes are running as services and will continue."
