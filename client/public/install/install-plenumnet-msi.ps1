<#
.SYNOPSIS
    PlenumNET MSI Installer — One-Click Setup
    Downloads, builds, and installs PlenumNET products via the
    plenum-pack MSI framework (Task #56).

.DESCRIPTION
    Served from https://plenumnet.replit.app/api/install-msi
    Run with:  irm https://plenumnet.replit.app/api/install-msi | iex
    Or download the .bat wrapper from the Distribution page.

    This script:
      1. Installs prerequisites (Git, Rust, MSVC, LLVM/clang, .NET SDK, WiX v4)
      2. Clones/updates the PlenumNET repository
      3. Builds plenum-pack (the MSI build tool)
      4. Builds all PlenumNET product binaries
      5. Runs plenum-pack build for each product to generate MSIs
      6. Installs the generated MSIs

.NOTES
    Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
    Applied Physics Division
#>

$ErrorActionPreference = "Stop"
$REMOTE_CRS     = "https://plenumnet.replit.app"
$RepoDir        = "C:\PlenumNET"
$RepoUrl        = "https://github.com/SigmaWolf-8/Ternary.git"
$OutputDir      = Join-Path $RepoDir "installer-output"
$LogFile        = Join-Path $env:TEMP "PlenumNET_MSI_Install.log"

$Products = @(
    @{ Name = "PlenumNET-Launcher"; Crate = "plenum-launcher"; ManifestDir = "tools/plenum-launcher"; ExtraCrates = @("plenum-launcher-elevate") },
    @{ Name = "InterCubeDaemon"; Crate = "inter-cube"; ManifestDir = "services/inter-cube"; ExtraCrates = @() },
    @{ Name = "NinjaExec"; Crate = "ninja-exec"; ManifestDir = "ninja-exec"; ExtraCrates = @() }
)

function Write-Log {
    param([string]$Message, [string]$Color = "White")
    $ts = Get-Date -Format "HH:mm:ss"
    $logLine = "[$ts] $Message"
    Add-Content -Path $LogFile -Value $logLine -ErrorAction SilentlyContinue
    Write-Host $Message -ForegroundColor $Color
}

function Test-Admin {
    $p = New-Object Security.Principal.WindowsPrincipal([Security.Principal.WindowsIdentity]::GetCurrent())
    return $p.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
}

function Test-Command($cmd) {
    try { Get-Command $cmd -ErrorAction Stop | Out-Null; return $true }
    catch { return $false }
}

# ── Banner ────────────────────────────────────────────────────────────────────
Write-Host ""
Write-Host "==========================================================" -ForegroundColor Cyan
Write-Host "  PlenumNET MSI Installer" -ForegroundColor Cyan
Write-Host "  Capomastro Holdings Ltd. — Applied Physics Division" -ForegroundColor Cyan
Write-Host "==========================================================" -ForegroundColor Cyan
Write-Host ""
Write-Host "  This script builds and installs PlenumNET products" -ForegroundColor White
Write-Host "  using the plenum-pack MSI framework." -ForegroundColor White
Write-Host ""
Write-Host "  Products: Launcher, Inter-Cube Daemon, NinjaExec" -ForegroundColor White
Write-Host "  Log file: $LogFile" -ForegroundColor DarkGray
Write-Host ""

if (-not (Test-Admin)) {
    Write-Host "  Administrator privileges required for MSI installation." -ForegroundColor Yellow
    Write-Host "  Elevating..." -ForegroundColor DarkGray
    $scriptPath = $MyInvocation.MyCommand.Definition
    if ($scriptPath) {
        Start-Process powershell.exe -Verb RunAs -ArgumentList "-NoProfile -ExecutionPolicy Bypass -File `"$scriptPath`""
        Write-Host "  Re-launched as Administrator. This window can be closed." -ForegroundColor Green
        exit 0
    } else {
        $tempScript = Join-Path $env:TEMP "install-plenumnet-msi-elevated.ps1"
        Invoke-WebRequest -Uri "$REMOTE_CRS/api/install-msi" -OutFile $tempScript -UseBasicParsing
        Start-Process powershell.exe -Verb RunAs -ArgumentList "-NoProfile -ExecutionPolicy Bypass -File `"$tempScript`""
        Write-Host "  Re-launched as Administrator. This window can be closed." -ForegroundColor Green
        exit 0
    }
}

# ── Detect architecture ──────────────────────────────────────────────────────
try {
    $cpuArch = [System.Runtime.InteropServices.RuntimeInformation]::OSArchitecture.ToString()
} catch {
    $cpuArch = $env:PROCESSOR_ARCHITECTURE
    if (-not $cpuArch) { $cpuArch = "AMD64" }
}

$rustTarget = if ($cpuArch -eq "Arm64") { "aarch64-pc-windows-msvc" } else { "x86_64-pc-windows-msvc" }
$archFlag = if ($cpuArch -eq "Arm64") { "aarch64" } else { "x86_64" }
Write-Log "  Architecture: $cpuArch (Rust target: $rustTarget)" "White"

# ── STEP 1: Prerequisites ────────────────────────────────────────────────────
$totalSteps = 8
$step = 1
Write-Host ""
Write-Log "STEP $step/$totalSteps`: Checking prerequisites" "Yellow"
Write-Host "---"

# Git
if (-not (Test-Command "git")) {
    Write-Log "  ERROR: git is not installed. Install from https://git-scm.com/download/win" "Red"
    Read-Host "Press Enter to close"
    exit 1
}
Write-Log "  [OK] git" "Green"

# Rust
if (-not (Test-Command "cargo")) {
    Write-Log "  -> Rust not found — installing rustup..." "Yellow"
    $rustupExe = Join-Path $env:TEMP "rustup-init.exe"
    Invoke-WebRequest -Uri "https://win.rustup.rs/x86_64" -OutFile $rustupExe -UseBasicParsing
    Start-Process -FilePath $rustupExe -ArgumentList "-y" -Wait -NoNewWindow
    Remove-Item $rustupExe -Force -ErrorAction SilentlyContinue
    $cargoBin = Join-Path (Join-Path $env:USERPROFILE ".cargo") "bin"
    $env:PATH += ";$cargoBin"
    if (-not (Test-Command "cargo")) {
        Write-Log "  ERROR: cargo not found after install — restart terminal and re-run." "Red"
        Read-Host "Press Enter to close"
        exit 1
    }
}
Write-Log "  [OK] cargo" "Green"

# MSVC build tools
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
            Write-Log "  [OK] MSVC environment activated ($vcvarsName)" "Green"
        }
    }
} else {
    Write-Log "  WARN: Visual Studio / MSVC Build Tools not found." "Yellow"
    Write-Log "        Install 'Desktop development with C++' from https://visualstudio.microsoft.com/downloads/" "Yellow"
    Write-Log "        Continuing — build may fail without MSVC linker." "Yellow"
}

# LLVM/clang
$hasClang = Get-Command clang -ErrorAction SilentlyContinue
if (-not $hasClang) {
    $llvmBin = "C:\Program Files\LLVM\bin"
    if (Test-Path -LiteralPath (Join-Path $llvmBin "clang.exe")) {
        $env:PATH += ";$llvmBin"
        $hasClang = Get-Command clang -ErrorAction SilentlyContinue
    }
}
if ($hasClang) {
    Write-Log "  [OK] clang" "Green"
    $env:CC = "clang"
    $env:AR = "llvm-ar"
} else {
    Write-Log "  -> clang not found — installing LLVM via winget..." "Yellow"
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
        Write-Log "  [OK] clang installed" "Green"
        $env:CC = "clang"
        $env:AR = "llvm-ar"
    } else {
        Write-Log "  ERROR: clang not found after LLVM install. Restart terminal and re-run." "Red"
        Read-Host "Press Enter to close"
        exit 1
    }
}

# .NET SDK (required for WiX v4)
if (-not (Test-Command "dotnet")) {
    Write-Log "  -> .NET SDK not found — installing..." "Yellow"
    $hasWinget = Get-Command winget -ErrorAction SilentlyContinue
    if ($hasWinget) {
        winget install --id Microsoft.DotNet.SDK.8 --silent --accept-package-agreements --accept-source-agreements
    } else {
        $dotnetInstaller = Join-Path $env:TEMP "dotnet-install.ps1"
        Invoke-WebRequest -Uri "https://dot.net/v1/dotnet-install.ps1" -OutFile $dotnetInstaller -UseBasicParsing
        & $dotnetInstaller -Channel 8.0 -InstallDir "$env:ProgramFiles\dotnet"
        Remove-Item $dotnetInstaller -Force -ErrorAction SilentlyContinue
    }
    $env:PATH = [System.Environment]::GetEnvironmentVariable("PATH","Machine") + ";" + [System.Environment]::GetEnvironmentVariable("PATH","User")
    $dotnetPath = "$env:ProgramFiles\dotnet"
    if (Test-Path (Join-Path $dotnetPath "dotnet.exe")) { $env:PATH += ";$dotnetPath" }
    if (-not (Test-Command "dotnet")) {
        Write-Log "  ERROR: .NET SDK not found after install. Restart terminal and re-run." "Red"
        Read-Host "Press Enter to close"
        exit 1
    }
}
Write-Log "  [OK] .NET SDK" "Green"

# WiX v4 (dotnet tool)
$wixAvailable = $false
try {
    $wixCheck = & dotnet tool list --global 2>$null | Select-String "wix"
    if ($wixCheck) { $wixAvailable = $true }
} catch {}
if (-not $wixAvailable) {
    Write-Log "  -> WiX Toolset v4 not found — installing..." "Yellow"
    & dotnet tool install --global wix 2>&1 | Out-Null
    $dotnetToolsPath = Join-Path $env:USERPROFILE ".dotnet\tools"
    if (Test-Path $dotnetToolsPath) { $env:PATH += ";$dotnetToolsPath" }
}
if (Test-Command "wix") {
    Write-Log "  [OK] WiX v4" "Green"
} else {
    $dotnetToolsPath = Join-Path $env:USERPROFILE ".dotnet\tools"
    if (Test-Path $dotnetToolsPath) { $env:PATH += ";$dotnetToolsPath" }
    if (Test-Command "wix") {
        Write-Log "  [OK] WiX v4" "Green"
    } else {
        Write-Log "  WARN: WiX not on PATH — MSI generation may fail. Add $dotnetToolsPath to PATH." "Yellow"
    }
}

# ── STEP 2: Clone/Update Source ──────────────────────────────────────────────
$step++
Write-Host ""
Write-Log "STEP $step/$totalSteps`: Source code" "Yellow"
Write-Host "---"

if (-not (Test-Path $RepoDir)) {
    Write-Log "  Cloning PlenumNET repository..." "White"
    $null = & git clone --depth 1 $RepoUrl $RepoDir 2>&1
    if ($LASTEXITCODE -ne 0) {
        Write-Log "  ERROR: git clone failed." "Red"
        Read-Host "Press Enter to close"
        exit 1
    }
} elseif (-not (Test-Path (Join-Path $RepoDir ".git"))) {
    Write-Log "  Converting to git repo..." "Yellow"
    Push-Location $RepoDir
    $null = & git init 2>&1
    $null = & git remote add origin $RepoUrl 2>&1
    $null = & git fetch origin main 2>&1
    $null = & git reset --hard origin/main 2>&1
    Pop-Location
} else {
    Write-Log "  Updating source..." "White"
    Push-Location $RepoDir
    $null = & git pull origin main --ff-only 2>&1
    Pop-Location
}
Write-Log "  [OK] Source ready at $RepoDir" "Green"

# ── STEP 3: Build plenum-pack ────────────────────────────────────────────────
$step++
Write-Host ""
Write-Log "STEP $step/$totalSteps`: Building plenum-pack (MSI build tool)" "Yellow"
Write-Host "---"

Push-Location $RepoDir
$env:CARGO_BUILD_JOBS = "1"
& cargo build --release -p plenum-pack 2>&1 | ForEach-Object {
    $line = $_.ToString()
    if ($line -match "error") { Write-Log "  $line" "Red" }
    elseif ($line -match "Compiling|Finished") { Write-Log "  $line" "DarkGray" }
}
$buildExit = $LASTEXITCODE
Pop-Location

$plenumPackBin = Join-Path $RepoDir "target\release\plenum-pack.exe"
if ($buildExit -ne 0 -or -not (Test-Path $plenumPackBin)) {
    Write-Log "  ERROR: plenum-pack build failed." "Red"
    Read-Host "Press Enter to close"
    exit 1
}
Write-Log "  [OK] plenum-pack built" "Green"

# ── STEP 4: Build product binaries ──────────────────────────────────────────
$step++
Write-Host ""
Write-Log "STEP $step/$totalSteps`: Building product binaries" "Yellow"
Write-Host "---"

$allCrates = @("plenum-launcher", "plenum-launcher-elevate", "inter-cube", "ninja-exec")
foreach ($crate in $allCrates) {
    Write-Log "  Building $crate..." "White"
    Push-Location $RepoDir
    & cargo build --release -p $crate 2>&1 | ForEach-Object {
        $line = $_.ToString()
        if ($line -match "error") { Write-Log "  $line" "Red" }
        elseif ($line -match "Compiling|Finished") { Write-Log "  $line" "DarkGray" }
    }
    $crateBuildExit = $LASTEXITCODE
    Pop-Location
    if ($crateBuildExit -ne 0) {
        Write-Log "  WARN: $crate build failed (exit $crateBuildExit) — skipping this product." "Yellow"
    } else {
        Write-Log "  [OK] $crate built" "Green"
    }
}

# ── STEP 5: Validate manifests ──────────────────────────────────────────────
$step++
Write-Host ""
Write-Log "STEP $step/$totalSteps`: Validating product manifests" "Yellow"
Write-Host "---"

foreach ($product in $Products) {
    $manifestDir = Join-Path $RepoDir $product.ManifestDir
    $manifestFile = Join-Path $manifestDir "plenum-app.toml"
    if (-not (Test-Path $manifestFile)) {
        Write-Log "  WARN: Manifest not found at $manifestFile — skipping $($product.Name)" "Yellow"
        continue
    }
    Write-Log "  Validating $($product.Name)..." "White"
    $validateOutput = & $plenumPackBin validate --manifest-dir $manifestDir 2>&1
    if ($LASTEXITCODE -eq 0) {
        Write-Log "  [OK] $($product.Name) manifest valid" "Green"
    } else {
        Write-Log "  WARN: $($product.Name) validation issues:" "Yellow"
        $validateOutput | ForEach-Object { Write-Log "    $_" "DarkGray" }
    }
}

# ── STEP 6: Generate MSI installers ─────────────────────────────────────────
$step++
Write-Host ""
Write-Log "STEP $step/$totalSteps`: Generating MSI installers" "Yellow"
Write-Host "---"

New-Item -ItemType Directory -Force -Path $OutputDir | Out-Null
$generatedMSIs = @()
$releaseDir = Join-Path $RepoDir "target\release"

foreach ($product in $Products) {
    $manifestDir = Join-Path $RepoDir $product.ManifestDir
    $manifestFile = Join-Path $manifestDir "plenum-app.toml"
    if (-not (Test-Path $manifestFile)) { continue }

    $binaryName = switch ($product.Crate) {
        "plenum-launcher" { "plenum-launcher.exe" }
        "inter-cube" { "inter-cube-daemon.exe" }
        "ninja-exec" { "ninja-exec.exe" }
        default { "$($product.Crate).exe" }
    }
    $binaryPath = Join-Path $releaseDir $binaryName
    if (-not (Test-Path $binaryPath)) {
        Write-Log "  WARN: Binary $binaryName not found — skipping $($product.Name)" "Yellow"
        continue
    }

    Write-Log "  Building MSI for $($product.Name) ($archFlag)..." "White"
    $packOutput = & $plenumPackBin build --arch $archFlag --manifest-dir $manifestDir --binary-dir $releaseDir 2>&1
    if ($LASTEXITCODE -eq 0) {
        $msiFiles = Get-ChildItem -Path (Join-Path $manifestDir "plenum-pack-output") -Filter "*.msi" -ErrorAction SilentlyContinue
        if ($msiFiles) {
            foreach ($msi in $msiFiles) {
                $destMsi = Join-Path $OutputDir $msi.Name
                Copy-Item -Path $msi.FullName -Destination $destMsi -Force
                $generatedMSIs += $destMsi
                $sizeMB = [math]::Round($msi.Length / 1MB, 1)
                Write-Log "  [OK] $($msi.Name) ($sizeMB MB)" "Green"
            }
        } else {
            Write-Log "  WARN: No MSI produced for $($product.Name). WiX may have generated only .wxs files." "Yellow"
            $wxsFiles = Get-ChildItem -Path (Join-Path $manifestDir "plenum-pack-output") -Filter "*.wxs" -ErrorAction SilentlyContinue
            if ($wxsFiles) {
                Write-Log "  Generated WiX source files (MSI compilation requires WiX on PATH):" "DarkGray"
                foreach ($wxs in $wxsFiles) {
                    Write-Log "    $($wxs.FullName)" "DarkGray"
                }
            }
        }
    } else {
        Write-Log "  WARN: plenum-pack build failed for $($product.Name):" "Yellow"
        $packOutput | ForEach-Object { Write-Log "    $_" "DarkGray" }
    }
}

# ── STEP 7: Install MSIs ────────────────────────────────────────────────────
$step++
Write-Host ""
Write-Log "STEP $step/$totalSteps`: Installing PlenumNET products" "Yellow"
Write-Host "---"

if ($generatedMSIs.Count -eq 0) {
    Write-Log "  No MSI files were generated. Skipping installation." "Yellow"
    Write-Log "  This may mean WiX could not compile the .wxs files." "Yellow"
    Write-Log "  Check that 'wix' is on your PATH: dotnet tool install --global wix" "Yellow"
} else {
    $launcherMsi = $generatedMSIs | Where-Object { $_ -match "Launcher" } | Select-Object -First 1
    $otherMsis = $generatedMSIs | Where-Object { $_ -notmatch "Launcher" }

    if ($launcherMsi) {
        Write-Log "  Installing PlenumNET Launcher..." "White"
        $msiLog = Join-Path $env:TEMP "PlenumNET_Launcher_install.log"
        $proc = Start-Process -FilePath "msiexec.exe" -ArgumentList "/i `"$launcherMsi`" /qb /l*v `"$msiLog`"" -Wait -PassThru
        if ($proc.ExitCode -eq 0) {
            Write-Log "  [OK] PlenumNET Launcher installed" "Green"
        } else {
            Write-Log "  WARN: Launcher install exited with code $($proc.ExitCode). Log: $msiLog" "Yellow"
        }
    }

    foreach ($msi in $otherMsis) {
        $msiName = [System.IO.Path]::GetFileNameWithoutExtension($msi)
        Write-Log "  Installing $msiName..." "White"
        $msiLog = Join-Path $env:TEMP "PlenumNET_${msiName}_install.log"
        $proc = Start-Process -FilePath "msiexec.exe" -ArgumentList "/i `"$msi`" /qb /l*v `"$msiLog`"" -Wait -PassThru
        if ($proc.ExitCode -eq 0) {
            Write-Log "  [OK] $msiName installed" "Green"
        } else {
            Write-Log "  WARN: $msiName install exited with code $($proc.ExitCode). Log: $msiLog" "Yellow"
        }
    }
}

# ── STEP 8: Summary ─────────────────────────────────────────────────────────
$step++
Write-Host ""
Write-Log "STEP $step/$totalSteps`: Installation Summary" "Yellow"
Write-Host "==========================================================" -ForegroundColor Cyan

Write-Host ""
Write-Log "  Architecture   : $cpuArch" "White"
Write-Log "  Source          : $RepoDir" "White"
Write-Log "  MSI output      : $OutputDir" "White"
Write-Log "  Install log     : $LogFile" "White"
Write-Host ""

if ($generatedMSIs.Count -gt 0) {
    Write-Log "  Installed MSIs:" "Green"
    foreach ($msi in $generatedMSIs) {
        Write-Log "    - $([System.IO.Path]::GetFileName($msi))" "Green"
    }
    Write-Host ""
    Write-Log "  PlenumNET products are now installed under:" "White"
    Write-Log "    %ProgramFiles%\Capomastro\" "White"
    Write-Host ""
    Write-Log "  Data directories (preserved on uninstall):" "White"
    Write-Log "    %APPDATA%\NinjaExec\" "White"
    Write-Log "    %APPDATA%\InterCubeDaemon\" "White"
    Write-Log "    %APPDATA%\PlenumNET-Launcher\" "White"
} else {
    Write-Log "  No MSIs were installed." "Yellow"
    Write-Log "  The WiX source files (.wxs) have been generated at:" "White"
    foreach ($product in $Products) {
        $wxsDir = Join-Path (Join-Path $RepoDir $product.ManifestDir) "plenum-pack-output"
        if (Test-Path $wxsDir) {
            Write-Log "    $wxsDir" "White"
        }
    }
    Write-Log "" "White"
    Write-Log "  To complete MSI generation, ensure WiX v4 is on PATH:" "White"
    Write-Log "    dotnet tool install --global wix" "White"
    Write-Log "  Then re-run this installer." "White"
}

Write-Host ""
Write-Host "==========================================================" -ForegroundColor Cyan
Write-Host "  PlenumNET MSI Installation Complete" -ForegroundColor Cyan
Write-Host "  Capomastro Holdings Ltd. — Applied Physics Division" -ForegroundColor Cyan
Write-Host "==========================================================" -ForegroundColor Cyan
Write-Host ""
Read-Host "Press Enter to close"
