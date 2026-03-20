# PlenumLAN Cube Node Installer / Re-run Script
# Connects to the PlenumNET CRS at https://plenumnet.replit.app
$ErrorActionPreference = "Stop"
trap {
  Write-Host ""
  Write-Host "=== INSTALLER ERROR ===" -ForegroundColor Red
  Write-Host $_.Exception.Message -ForegroundColor Red
  Write-Host ""
  Read-Host "Press Enter to close"
  break
}

$CRS_URL       = "https://plenumnet.replit.app"
$PLENUMNET_DIR = "C:\PlenumNET"
$IDENTITY_DIR  = Join-Path (Join-Path $env:USERPROFILE ".plenumnet") "identity"
$LOG_DIR       = Join-Path $PLENUMNET_DIR "logs"

Write-Host "=== PlenumLAN Cube Node Installer ===" -ForegroundColor Cyan
Write-Host "CRS  : $CRS_URL"
Write-Host ""

# -- 1. Detect local IP -----------------------------------------------------
$ip = (Get-NetIPAddress -AddressFamily IPv4 |
  Where-Object { $_.IPAddress -notmatch '^127\.' -and $_.IPAddress -notmatch '^169\.254' -and $_.PrefixOrigin -ne 'WellKnown' } |
  Sort-Object @{ Expression = { switch -Wildcard ($_.InterfaceAlias) { 'Wi-Fi*' { 0 } 'Ethernet*' { 1 } default { 2 } } } } |
  Select-Object -First 1).IPAddress
if (-not $ip) { $ip = "0.0.0.0" }
$CUBE_ENDPOINT = "${ip}:51820"
Write-Host "Local endpoint : $CUBE_ENDPOINT"
if ($ip -eq "0.0.0.0") {
  Write-Host "  WARN Could not detect local IP -- routing may fail." -ForegroundColor Yellow
}

# -- 2. Check Rust/Cargo ----------------------------------------------------
Write-Host ""
Write-Host "Checking Rust/Cargo..."
if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
  Write-Host "  -> Rust not found -- installing rustup..."
  $rustupExe = Join-Path $env:TEMP "rustup-init.exe"
  Invoke-WebRequest -Uri "https://win.rustup.rs/x86_64" -OutFile $rustupExe -UseBasicParsing
  Start-Process -FilePath $rustupExe -ArgumentList "-y" -Wait -NoNewWindow
  Remove-Item $rustupExe -Force -ErrorAction SilentlyContinue
  $cargoBin = Join-Path (Join-Path $env:USERPROFILE ".cargo") "bin"
  $env:PATH += ";$cargoBin"
  Write-Host "  OK Rust installed"
} else {
  Write-Host "  OK Cargo already installed: $(cargo --version 2>$null)"
}

# -- 3. Check Git ------------------------------------------------------------
if (-not (Get-Command git -ErrorAction SilentlyContinue)) {
  throw "Git is not installed. Install it from https://git-scm.com/download/win then re-run this installer."
}

# -- 4. Find Visual Studio and ARM64 build tools -----------------------------
Write-Host ""
Write-Host "Setting up build environment for the ring crypto crate..."
$cpuArch = [System.Runtime.InteropServices.RuntimeInformation]::OSArchitecture.ToString()
Write-Host "  -> Architecture: $cpuArch"

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

$vsPath = $null
if ($vsWhere) {
  $vsPath = & $vsWhere -latest -products * -property installationPath 2>$null
  if ($vsPath) {
    $vcvarsName = if ($cpuArch -eq "Arm64") { "vcvarsarm64.bat" } else { "vcvars64.bat" }
    $vcvars = Join-Path (Join-Path (Join-Path $vsPath "VC") "Auxiliary\Build") $vcvarsName
    if (Test-Path -LiteralPath $vcvars) {
      Write-Host "  -> Activating MSVC environment ($vcvarsName)..."
      $envLines = cmd.exe /c ('"' + $vcvars + '" > nul 2>&1 && set')
      foreach ($line in $envLines) {
        if ($line -match '^([^=\r\n]+)=(.*)$') {
          [System.Environment]::SetEnvironmentVariable($Matches[1], $Matches[2], "Process")
        }
      }
      Write-Host "  OK MSVC environment activated (VCINSTALLDIR = $env:VCINSTALLDIR)"
    } else {
      Write-Host "  WARN $vcvarsName not found in $vsPath -- MSVC ARM64 tools may not be installed." -ForegroundColor Yellow
    }
  } else {
    Write-Host "  WARN No VS installation found via vswhere." -ForegroundColor Yellow
  }
} else {
  Write-Host "  WARN vswhere.exe not found -- cannot activate MSVC environment." -ForegroundColor Yellow
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
  Write-Host "  OK clang: $($hasClang.Source)"
  $env:CC = "clang"
  $env:AR = "llvm-ar"
} else {
  Write-Host "  -> clang not found -- installing LLVM via winget (~300 MB)..." -ForegroundColor Yellow
  $hasWinget = Get-Command winget -ErrorAction SilentlyContinue
  if ($hasWinget) {
    winget install --id LLVM.LLVM --silent --accept-package-agreements --accept-source-agreements
  } else {
    Write-Host "  -> winget not available -- downloading LLVM installer directly..."
    $llvmRelease = (Invoke-RestMethod "https://api.github.com/repos/llvm/llvm-project/releases/latest").tag_name
    $llvmVer = $llvmRelease -replace "llvmorg-",""
    $llvmUrl = "https://github.com/llvm/llvm-project/releases/download/$llvmRelease/LLVM-$llvmVer-win64.exe"
    $llvmInstaller = Join-Path $env:TEMP "llvm-installer.exe"
    Write-Host "  -> Downloading LLVM $llvmVer..."
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
    Write-Host "  OK clang installed"
    $env:CC = "clang"
    $env:AR = "llvm-ar"
  } else {
    throw "clang not found after LLVM install -- please restart this script in a new terminal."
  }
}

# -- 5. Clone/update PlenumNET and build daemon ------------------------------
Write-Host ""
Write-Host "Installing PlenumNET (inter-cube)..."
New-Item -ItemType Directory -Force -Path $LOG_DIR | Out-Null
function Invoke-ClonePlenumNET {
  Write-Host "  -> Cloning PlenumNET..."
  git clone --depth 1 https://github.com/SigmaWolf-8/Ternary $PLENUMNET_DIR
}
if (Test-Path (Join-Path $PLENUMNET_DIR ".git")) {
  Write-Host "  -> Repository exists, updating..."
  try {
    $pullOutput = git -C $PLENUMNET_DIR pull --ff-only 2>&1
    Write-Host "  -> $pullOutput"
  } catch {
    Write-Host "  -> git pull failed (network issue?) -- continuing with existing checkout" -ForegroundColor Yellow
  }
  $icToml = Join-Path (Join-Path (Join-Path $PLENUMNET_DIR "services") "inter-cube") "Cargo.toml"
  if (-not (Test-Path $icToml)) {
    Write-Host "  -> Existing checkout is incomplete -- re-cloning..."
    Remove-Item -Recurse -Force $PLENUMNET_DIR
    Invoke-ClonePlenumNET
  }
} elseif (Test-Path $PLENUMNET_DIR) {
  Write-Host "  -> $PLENUMNET_DIR exists but is not a git repo -- using existing source"
} else {
  Invoke-ClonePlenumNET
}

Write-Host "  -> Building inter-cube daemon (first build takes a few minutes)..."
Write-Host "  Cargo warnings are normal -- only errors matter." -ForegroundColor Gray
Push-Location $PLENUMNET_DIR
$ErrorActionPreference = "Continue"
cargo build --release --package inter-cube 2>&1 | ForEach-Object { Write-Host $_ }
$cargoBuildExit = $LASTEXITCODE
$ErrorActionPreference = "Stop"
Pop-Location
if ($cargoBuildExit -ne 0) {
  throw "cargo build failed with exit code $cargoBuildExit -- check the output above for errors."
}
$relDir = Join-Path (Join-Path $PLENUMNET_DIR "target") "release"
$daemonBin = Get-ChildItem -Path $relDir -Filter "inter-cube*.exe" -ErrorAction SilentlyContinue |
  Where-Object { $_.Name -notlike "*.d" } | Select-Object -First 1
if (-not $daemonBin) {
  throw "Build completed but no inter-cube binary found in $relDir -- check cargo output above for errors."
}
$DAEMON_PATH = $daemonBin.FullName
Write-Host "  OK Daemon built: $DAEMON_PATH" -ForegroundColor Green

# -- 6. Identity passphrase + PT26-DSA keygen --------------------------------
Write-Host ""
Write-Host "Setting up PlenumNET identity..."
New-Item -ItemType Directory -Force -Path $IDENTITY_DIR | Out-Null
$PASSPHRASE_FILE = Join-Path $IDENTITY_DIR ".passphrase"

if (Test-Path $PASSPHRASE_FILE) {
  $CUBE_PASSPHRASE = (Get-Content $PASSPHRASE_FILE -Raw).Trim()
  Write-Host "  -> Loaded existing identity passphrase"
} else {
  $rng   = [System.Security.Cryptography.RNGCryptoServiceProvider]::new()
  $bytes = [byte[]]::new(24)
  $rng.GetBytes($bytes)
  $CUBE_PASSPHRASE = ($bytes | ForEach-Object { $_.ToString("x2") }) -join ""
  $rng.Dispose()
  $CUBE_PASSPHRASE | Set-Content -Path $PASSPHRASE_FILE -NoNewline
  $acl = Get-Acl $PASSPHRASE_FILE
  $acl.SetAccessRuleProtection($true, $false)
  $userRule = New-Object System.Security.AccessControl.FileSystemAccessRule(
    [System.Security.Principal.WindowsIdentity]::GetCurrent().Name,
    "FullControl", "Allow"
  )
  $acl.SetAccessRule($userRule)
  Set-Acl $PASSPHRASE_FILE $acl
  Write-Host "  OK Generated and saved identity passphrase"
}
$env:CUBE_IDENTITY_PASSPHRASE = $CUBE_PASSPHRASE

Write-Host "  -> Generating PT26-DSA identity keypair..."
$env:CUBE_MODE = "keygen"
$keygenLog = Join-Path $LOG_DIR "keygen.log"
$ErrorActionPreference = "Continue"
$keygenOutput = & $DAEMON_PATH 2>$keygenLog
$ErrorActionPreference = "Stop"
$env:CUBE_MODE = $null
$pkLine = $keygenOutput | Where-Object { $_ -match "PT26-DSA Public Key" } | Select-Object -First 1
if ($pkLine -match ':\s*([0-9a-fA-F]+)\s*$') {
  $PUB_KEY = $matches[1]
} else {
  $PUB_KEY = ""
}
if (-not $PUB_KEY) {
  Write-Host "  Keygen output:" -ForegroundColor Yellow
  $keygenOutput | ForEach-Object { Write-Host "    $_" }
  throw "Daemon keygen produced no public key -- check $keygenLog for details."
}
Write-Host "  OK Public key: $($PUB_KEY.Substring(0, [Math]::Min(32, $PUB_KEY.Length)))..." -ForegroundColor Green

# -- 7. Start cube daemon connecting to Replit CRS ---------------------------
Write-Host ""
Write-Host "Starting PlenumNET cube daemon..."
Write-Host "  CRS  : $CRS_URL"
Write-Host "  Node : $CUBE_ENDPOINT"
Get-Process | Where-Object { $_.Name -like "inter-cube*" } | Stop-Process -Force -ErrorAction SilentlyContinue
Start-Sleep -Milliseconds 500

$env:CUBE_MODE     = "cube"
$env:CUBE_CRS_URL  = $CRS_URL
$env:CUBE_ENDPOINT = $CUBE_ENDPOINT
$env:CUBE_API_PORT = "8080"
$daemonOutLog = Join-Path $LOG_DIR "intercube-cube-out.log"
$daemonErrLog = Join-Path $LOG_DIR "intercube-cube-err.log"
$daemonProc = Start-Process -FilePath $DAEMON_PATH -NoNewWindow -PassThru -RedirectStandardOutput $daemonOutLog -RedirectStandardError $daemonErrLog
Write-Host "  OK Daemon started (PID $($daemonProc.Id))" -ForegroundColor Green
Start-Sleep -Seconds 5

# -- 8. Verify registration -------------------------------------------------
Write-Host ""
Write-Host "Verifying CRS registration..."
try {
  $health = Invoke-RestMethod -Uri "$CRS_URL/health/crs" -TimeoutSec 10 -ErrorAction Stop
  Write-Host "  OK CRS reachable: $($health.service) v$($health.version)" -ForegroundColor Green
} catch {
  Write-Host "  WARN CRS health check failed: $_ " -ForegroundColor Yellow
  Write-Host "  Check log: $daemonOutLog" -ForegroundColor Yellow
}

# -- 9. Summary --------------------------------------------------------------
Write-Host ""
Write-Host "========================================" -ForegroundColor Green
Write-Host "  PlenumLAN Cube Node is LIVE" -ForegroundColor Green
Write-Host "  Binary   : $DAEMON_PATH" -ForegroundColor Green
Write-Host "  PID      : $($daemonProc.Id)" -ForegroundColor Green
Write-Host "  CRS      : $CRS_URL" -ForegroundColor Green
Write-Host "  Endpoint : $CUBE_ENDPOINT" -ForegroundColor Green
Write-Host "  PubKey   : $($PUB_KEY.Substring(0, [Math]::Min(32, $PUB_KEY.Length)))..." -ForegroundColor Green
Write-Host "  Logs     : $LOG_DIR" -ForegroundColor Green
Write-Host "========================================" -ForegroundColor Green
Write-Host ""
Write-Host "Daemon runs in the background." -ForegroundColor Yellow
Write-Host "To stop it: Stop-Process -Id $($daemonProc.Id)" -ForegroundColor Yellow
Read-Host "Press Enter to close (daemon keeps running)"
