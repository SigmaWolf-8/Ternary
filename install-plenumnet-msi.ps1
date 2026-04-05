<#
.SYNOPSIS
    PlenumNET Installer -- Developer / Early Access
    Downloads, builds, and installs PlenumNET products via the
    plenum-pack MSI framework.

.DESCRIPTION
    Served from https://plenumnet.replit.app/api/install-msi
    Run with:  irm https://plenumnet.replit.app/api/install-msi | iex
    Or download the .bat wrapper from the Distribution page.

    This script:
      1. Checks prerequisites (Git, Rust, MSVC, LLVM/clang, .NET SDK, WiX v4, WebView2)
      2. Clones/updates the PlenumNET repository
      3. Builds plenum-pack (the MSI build tool)
      4. Builds all PlenumNET product binaries
      5. Validates product manifests
      6. Generates MSI installers via plenum-pack
      7. Installs MSI packages (Inter-Cube Daemon first, then Launcher and NinjaExec)
      8. Displays installation summary

.NOTES
    Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
    Applied Physics Division
#>

$ErrorActionPreference = "Continue"
$REMOTE_CRS     = "https://plenumnet.replit.app"
$RepoDir        = "C:\PlenumNET"
$RepoUrl        = "https://github.com/SigmaWolf-8/Ternary.git"
$OutputDir      = Join-Path $RepoDir "installer-output"
$LogFile        = Join-Path $env:TEMP "PlenumNET_MSI_Install.log"
$installStart   = Get-Date

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

function Write-StepTime {
    param([string]$StepName)
    $elapsed = (Get-Date) - $installStart
    $mins = [math]::Floor($elapsed.TotalMinutes)
    $secs = $elapsed.Seconds
    Write-Log "  Elapsed: ${mins}m ${secs}s" "DarkGray"
}

function Test-Admin {
    $p = New-Object Security.Principal.WindowsPrincipal([Security.Principal.WindowsIdentity]::GetCurrent())
    return $p.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
}

function Test-Command($cmd) {
    try { Get-Command $cmd -ErrorAction Stop | Out-Null; return $true }
    catch { return $false }
}

# == Brand Banner =============================================================
Write-Host ""
Write-Host "==========================================================" -ForegroundColor Cyan
Write-Host "       ____  " -ForegroundColor Cyan
Write-Host "      |  _ \ " -ForegroundColor Cyan
Write-Host "      | |_) |" -ForegroundColor Cyan
Write-Host "      |  __/ " -ForegroundColor Cyan
Write-Host "      |_|    " -ForegroundColor Cyan
Write-Host "" -ForegroundColor Cyan
Write-Host "  PlenumNET Installer" -ForegroundColor Cyan
Write-Host "  Post-Quantum Internet Infrastructure" -ForegroundColor Cyan
Write-Host "  Capomastro Holdings Ltd." -ForegroundColor Cyan
Write-Host "==========================================================" -ForegroundColor Cyan
Write-Host ""
Write-Host "  This installer sets up the PlenumNET suite:" -ForegroundColor White
Write-Host "    -- Launcher (application hub)" -ForegroundColor White
Write-Host "    -- NinjaExec (signing agent)" -ForegroundColor White
Write-Host "    -- Inter-Cube Daemon (network service)" -ForegroundColor White
Write-Host ""
Write-Host "  Learn more: https://plenumnet.replit.app" -ForegroundColor DarkGray
Write-Host ""
Write-Host "  NOTE: This installer compiles PlenumNET from source" -ForegroundColor Yellow
Write-Host "  code. It requires developer tools (Git, Rust, Visual" -ForegroundColor Yellow
Write-Host "  Studio) and may take 30-60 minutes depending on your" -ForegroundColor Yellow
Write-Host "  system." -ForegroundColor Yellow
Write-Host ""
Write-Host "  Console colors: Cyan = brand, Green = success," -ForegroundColor DarkGray
Write-Host "  Yellow = warning, Red = error, Gray = debug" -ForegroundColor DarkGray
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

# == Pre-install summary with confirmation ====================================
Write-Host "==========================================================" -ForegroundColor Cyan
Write-Host "  Pre-Install Summary" -ForegroundColor Cyan
Write-Host "==========================================================" -ForegroundColor Cyan
Write-Host ""

$programFiles = $env:ProgramFiles
$appData = $env:APPDATA

Write-Host "  The following products will be installed:" -ForegroundColor White
Write-Host ""
Write-Host "  1. PlenumNET Launcher" -ForegroundColor Green
Write-Host "     Manage all your PlenumNET applications from one" -ForegroundColor White
Write-Host "     panel in your taskbar (auto-starts with Windows)" -ForegroundColor White
Write-Host "     Install: $programFiles\Capomastro\PlenumNET-Launcher" -ForegroundColor DarkGray
Write-Host "     Data:    $appData\PlenumNET-Launcher" -ForegroundColor DarkGray
Write-Host ""
Write-Host "  2. NinjaExec" -ForegroundColor Green
Write-Host "     Securely signs and authenticates your PlenumNET" -ForegroundColor White
Write-Host "     operations (auto-starts with Windows)" -ForegroundColor White
Write-Host "     Install: $programFiles\Capomastro\NinjaExec" -ForegroundColor DarkGray
Write-Host "     Data:    $appData\NinjaExec" -ForegroundColor DarkGray
Write-Host ""
Write-Host "  3. Inter-Cube Daemon" -ForegroundColor Green
Write-Host "     Connects your machine to the PlenumNET network" -ForegroundColor White
Write-Host "     (3 service instances)" -ForegroundColor White
Write-Host "     Install: $programFiles\Capomastro\InterCubeDaemon" -ForegroundColor DarkGray
Write-Host "     Data:    $appData\InterCubeDaemon" -ForegroundColor DarkGray
Write-Host ""
Write-Host "  Source:  $RepoDir" -ForegroundColor DarkGray
Write-Host "  Output:  $OutputDir" -ForegroundColor DarkGray
Write-Host ""

$confirm = Read-Host "  Continue? (Y/N)"
if ($confirm -ne 'Y' -and $confirm -ne 'y') {
    Write-Host ""
    Write-Host "  Installation cancelled." -ForegroundColor Yellow
    exit 0
}

Write-Host ""

# == Detect architecture =======================================================
try {
    $cpuArch = [System.Runtime.InteropServices.RuntimeInformation]::OSArchitecture.ToString()
} catch {
    $cpuArch = $env:PROCESSOR_ARCHITECTURE
    if (-not $cpuArch) { $cpuArch = "AMD64" }
}

$rustTarget = "x86_64-pc-windows-msvc"
$archFlag = "x86_64"
if ($cpuArch -eq "Arm64") {
    $rustTarget = "aarch64-pc-windows-msvc"
    $archFlag = "aarch64"
}
Write-Log "  Architecture: $cpuArch (Rust target: $rustTarget)" "White"

# == STEP 1/8: Prerequisites (1-5 minutes) ====================================
Write-Host ""
Write-Log "STEP 1/8: Checking prerequisites (1-5 minutes)" "Yellow"
Write-Host "---"

# Git
if (-not (Test-Command "git")) {
    Write-Log "  Error: Git is not installed." "Red"
    Write-Log "  Install from https://git-scm.com/download/win and re-run." "Red"
    Read-Host "Press Enter to close"
    exit 1
}
Write-Log "  [OK] git" "Green"

# Rust
if (-not (Test-Command "cargo")) {
    Write-Log "  -> Rust not found -- installing rustup..." "Yellow"
    $rustupExe = Join-Path $env:TEMP "rustup-init.exe"
    Invoke-WebRequest -Uri "https://win.rustup.rs/x86_64" -OutFile $rustupExe -UseBasicParsing
    Start-Process -FilePath $rustupExe -ArgumentList "-y" -Wait -NoNewWindow
    Remove-Item $rustupExe -Force -ErrorAction SilentlyContinue
    $cargoBin = Join-Path $env:USERPROFILE ".cargo\bin"
    $env:PATH += ";$cargoBin"
    if (-not (Test-Command "cargo")) {
        Write-Log "  Error: Rust installation could not be completed." "Red"
        Write-Log "  Please restart your terminal and re-run the installer." "Red"
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
        $vcvarsName = "vcvars64.bat"
        if ($cpuArch -eq "Arm64") { $vcvarsName = "vcvarsarm64.bat" }
        $vcvars = Join-Path $vsPath "VC\Auxiliary\Build\$vcvarsName"
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
    Write-Log "  Warning: Visual Studio / MSVC Build Tools not found." "Yellow"
    Write-Log "  Install 'Desktop development with C++' from:" "Yellow"
    Write-Log "  https://visualstudio.microsoft.com/downloads/" "Yellow"
    Write-Log "  Continuing -- build may fail without MSVC linker." "Yellow"
}

# LLVM/clang
$hasClang = Get-Command clang -ErrorAction SilentlyContinue
if (-not $hasClang) {
    $llvmBin = "C:\Program Files\LLVM\bin"
    $llvmClang = Join-Path $llvmBin "clang.exe"
    if (Test-Path -LiteralPath $llvmClang) {
        $env:PATH += ";$llvmBin"
        $hasClang = Get-Command clang -ErrorAction SilentlyContinue
    }
}
if ($hasClang) {
    Write-Log "  [OK] clang" "Green"
    $env:CC = "clang"
    $env:AR = "llvm-ar"
} else {
    Write-Log "  -> clang not found -- installing LLVM via winget..." "Yellow"
    $hasWinget = Get-Command winget -ErrorAction SilentlyContinue
    if ($hasWinget) {
        winget install --id LLVM.LLVM --silent --accept-package-agreements --accept-source-agreements
    } else {
        $llvmRelease = (Invoke-RestMethod "https://api.github.com/repos/llvm/llvm-project/releases/latest").tag_name
        $llvmVer = $llvmRelease -replace "llvmorg-",""
        $llvmUrl = "https://github.com/llvm/llvm-project/releases/download/$llvmRelease/LLVM-$llvmVer-win64.exe"
        $llvmInstaller = Join-Path $env:TEMP "llvm-installer.exe"
        $hasCurl = Get-Command curl.exe -ErrorAction SilentlyContinue
        if ($hasCurl) {
            curl.exe -fL $llvmUrl -o $llvmInstaller
        } else {
            Invoke-WebRequest -Uri $llvmUrl -OutFile $llvmInstaller -UseBasicParsing
        }
        Start-Process -FilePath $llvmInstaller -ArgumentList "/S" -Wait -NoNewWindow
        Remove-Item $llvmInstaller -Force -ErrorAction SilentlyContinue
    }
    $machinePath = [System.Environment]::GetEnvironmentVariable("PATH", "Machine")
    $userPath = [System.Environment]::GetEnvironmentVariable("PATH", "User")
    $env:PATH = $machinePath + ";" + $userPath
    $llvmBin = "C:\Program Files\LLVM\bin"
    $llvmClang = Join-Path $llvmBin "clang.exe"
    if (Test-Path -LiteralPath $llvmClang) {
        $env:PATH += ";$llvmBin"
    }
    $hasClang = Get-Command clang -ErrorAction SilentlyContinue
    if ($hasClang) {
        Write-Log "  [OK] clang installed" "Green"
        $env:CC = "clang"
        $env:AR = "llvm-ar"
    } else {
        Write-Log "  Error: LLVM/clang installation could not be completed." "Red"
        Write-Log "  Please restart your terminal and re-run the installer." "Red"
        Read-Host "Press Enter to close"
        exit 1
    }
}

# .NET SDK (required for WiX v4)
if (-not (Test-Command "dotnet")) {
    Write-Log "  -> .NET SDK not found -- installing..." "Yellow"
    $hasWinget = Get-Command winget -ErrorAction SilentlyContinue
    if ($hasWinget) {
        winget install --id Microsoft.DotNet.SDK.8 --silent --accept-package-agreements --accept-source-agreements
    } else {
        $dotnetInstaller = Join-Path $env:TEMP "dotnet-install.ps1"
        Invoke-WebRequest -Uri "https://dot.net/v1/dotnet-install.ps1" -OutFile $dotnetInstaller -UseBasicParsing
        & $dotnetInstaller -Channel 8.0 -InstallDir "$env:ProgramFiles\dotnet"
        Remove-Item $dotnetInstaller -Force -ErrorAction SilentlyContinue
    }
    $machinePath = [System.Environment]::GetEnvironmentVariable("PATH", "Machine")
    $userPath = [System.Environment]::GetEnvironmentVariable("PATH", "User")
    $env:PATH = $machinePath + ";" + $userPath
    $dotnetPath = "$env:ProgramFiles\dotnet"
    $dotnetExe = Join-Path $dotnetPath "dotnet.exe"
    if (Test-Path $dotnetExe) {
        $env:PATH += ";$dotnetPath"
    }
    if (-not (Test-Command "dotnet")) {
        Write-Log "  Error: .NET SDK installation could not be completed." "Red"
        Write-Log "  Please restart your terminal and re-run the installer." "Red"
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
    Write-Log "  -> WiX Toolset v4 not found -- installing..." "Yellow"
    & dotnet tool install --global wix 2>&1 | Out-Null
    $dotnetToolsPath = Join-Path $env:USERPROFILE ".dotnet\tools"
    if (Test-Path $dotnetToolsPath) {
        $env:PATH += ";$dotnetToolsPath"
    }
}
$dotnetToolsPath = Join-Path $env:USERPROFILE ".dotnet\tools"
if (Test-Path $dotnetToolsPath) {
    $env:PATH += ";$dotnetToolsPath"
}
if (Test-Command "wix") {
    Write-Log "  [OK] WiX v4" "Green"
} else {
    Write-Log "  Warning: WiX not on PATH -- MSI generation may fail." "Yellow"
    Write-Log "  Add $dotnetToolsPath to PATH." "Yellow"
}

# WebView2 Evergreen Runtime (required for Launcher popup)
$webview2Installed = $false
try {
    $wv2Key = Get-ItemProperty -Path "HKLM:\SOFTWARE\WOW6432Node\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}" -ErrorAction SilentlyContinue
    if ($wv2Key) { $webview2Installed = $true }
} catch {}
if (-not $webview2Installed) {
    try {
        $wv2Appx = Get-AppxPackage "Microsoft.WebView2Runtime" -ErrorAction SilentlyContinue
        if ($wv2Appx) { $webview2Installed = $true }
    } catch {}
}
if ($webview2Installed) {
    Write-Log "  [OK] WebView2 Runtime" "Green"
} else {
    Write-Log "  -> WebView2 Runtime not found -- installing..." "Yellow"
    $wv2Bootstrapper = Join-Path $env:TEMP "MicrosoftEdgeWebview2Setup.exe"
    try {
        Invoke-WebRequest -Uri "https://go.microsoft.com/fwlink/p/?LinkId=2124703" -OutFile $wv2Bootstrapper -UseBasicParsing
        Start-Process -FilePath $wv2Bootstrapper -ArgumentList "/silent /install" -Wait -NoNewWindow
        Remove-Item $wv2Bootstrapper -Force -ErrorAction SilentlyContinue
        Write-Log "  [OK] WebView2 Runtime installed" "Green"
    } catch {
        Write-Log "  Warning: WebView2 Runtime install failed." "Yellow"
        Write-Log "  The Launcher popup may not work without it." "Yellow"
    }
}

Write-StepTime "Prerequisites"

# == STEP 2/8: Clone/Update Source (2-10 minutes) =============================
Write-Host ""
Write-Log "STEP 2/8: Source code (2-10 minutes)" "Yellow"
Write-Host "---"

if (-not (Test-Path $RepoDir)) {
    Write-Log "  Cloning PlenumNET repository..." "White"
    $null = & git clone --depth 1 $RepoUrl $RepoDir 2>&1
    if ($LASTEXITCODE -ne 0) {
        Write-Log "  Error: Repository could not be downloaded." "Red"
        Write-Log "  Check your internet connection and try again." "Red"
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
Write-StepTime "Source code"

# == STEP 3/8: Build plenum-pack (10-20 minutes) ==============================
Write-Host ""
Write-Log "STEP 3/8: Building plenum-pack (10-20 minutes)" "Yellow"
Write-Host "---"

Push-Location $RepoDir
$env:CARGO_BUILD_JOBS = "1"
$buildLog = Join-Path $env:TEMP "PlenumNET_build_plenum-pack.log"
& cargo build --release -p plenum-pack 2>&1 | ForEach-Object {
    $line = $_.ToString()
    Add-Content -Path $buildLog -Value $line -ErrorAction SilentlyContinue
    if ($line -match "Compiling|Finished") { Write-Log "  $line" "DarkGray" }
}
$buildExit = $LASTEXITCODE
Pop-Location

$plenumPackBin = Join-Path $RepoDir "target\release\plenum-pack.exe"
if (($buildExit -ne 0) -or (-not (Test-Path $plenumPackBin))) {
    Write-Log "  Error: Building plenum-pack could not be completed." "Red"
    Write-Log "  This may be caused by missing development tools or" "Red"
    Write-Log "  an incompatible Rust version. The log file at:" "Red"
    Write-Log "    $buildLog" "Red"
    Write-Log "  contains technical details." "Red"
    Write-Log "  No products have been installed. You can retry by" "Red"
    Write-Log "  running this installer again." "Red"
    Write-Log "  If this persists, visit:" "Red"
    Write-Log "    https://plenumnet.replit.app/support" "Red"
    Read-Host "Press Enter to close"
    exit 1
}
Write-Log "  [OK] plenum-pack built" "Green"
Write-StepTime "plenum-pack"

# == STEP 4/8: Build ALL product binaries (10-20 minutes) =====================
# Two-phase: build ALL first, then install ALL. If any build fails, nothing
# is installed.
Write-Host ""
Write-Log "STEP 4/8: Building product binaries (10-20 minutes)" "Yellow"
Write-Host "---"

$allCrates = @("plenum-launcher", "plenum-launcher-elevate", "inter-cube", "ninja-exec")
$buildFailed = $false
$failedCrate = ""

foreach ($crate in $allCrates) {
    Write-Log "  Building $crate..." "White"
    $crateBuildLog = Join-Path $env:TEMP "PlenumNET_build_${crate}.log"
    Push-Location $RepoDir
    & cargo build --release -p $crate 2>&1 | ForEach-Object {
        $line = $_.ToString()
        Add-Content -Path $crateBuildLog -Value $line -ErrorAction SilentlyContinue
        if ($line -match "Compiling|Finished") { Write-Log "  $line" "DarkGray" }
    }
    $crateBuildExit = $LASTEXITCODE
    Pop-Location
    if ($crateBuildExit -ne 0) {
        $buildFailed = $true
        $failedCrate = $crate
        break
    }
    Write-Log "  [OK] $crate built" "Green"
}

if ($buildFailed) {
    $crateBuildLog = Join-Path $env:TEMP "PlenumNET_build_${failedCrate}.log"
    Write-Host ""
    Write-Log "  Error: Building $failedCrate could not be completed." "Red"
    Write-Log "  This may be caused by missing development tools or" "Red"
    Write-Log "  an incompatible Rust version. The log file at:" "Red"
    Write-Log "    $crateBuildLog" "Red"
    Write-Log "  contains technical details." "Red"
    Write-Host ""
    Write-Log "  No products have been installed. You can retry by" "Red"
    Write-Log "  running this installer again." "Red"
    Write-Log "  If this persists, visit:" "Red"
    Write-Log "    https://plenumnet.replit.app/support" "Red"
    Read-Host "Press Enter to close"
    exit 1
}

Write-StepTime "Product binaries"

# == STEP 5/8: Validate manifests (< 1 minute) ================================
Write-Host ""
Write-Log "STEP 5/8: Validating product manifests (< 1 minute)" "Yellow"
Write-Host "---"

foreach ($product in $Products) {
    $manifestDir = Join-Path $RepoDir $product.ManifestDir
    $manifestFile = Join-Path $manifestDir "plenum-app.toml"
    if (-not (Test-Path $manifestFile)) {
        $pName = $product.Name
        Write-Log "  Warning: Manifest not found at $manifestFile" "Yellow"
        Write-Log "  Skipping $pName" "Yellow"
        continue
    }
    $pName = $product.Name
    Write-Log "  Validating $pName..." "White"
    $validateOutput = & $plenumPackBin validate --manifest-dir $manifestDir 2>&1
    if ($LASTEXITCODE -eq 0) {
        Write-Log "  [OK] $pName manifest valid" "Green"
    } else {
        Write-Log "  Warning: $pName validation issues:" "Yellow"
        $validateOutput | ForEach-Object { Write-Log "    $_" "DarkGray" }
    }
}

Write-StepTime "Manifest validation"

# == STEP 6/8: Generate MSI installers (2-5 minutes) ==========================
Write-Host ""
Write-Log "STEP 6/8: Generating MSI installers (2-5 minutes)" "Yellow"
Write-Host "---"

New-Item -ItemType Directory -Force -Path $OutputDir | Out-Null
$generatedMSIs = @()
$releaseDir = Join-Path $RepoDir "target\release"

foreach ($product in $Products) {
    $manifestDir = Join-Path $RepoDir $product.ManifestDir
    $manifestFile = Join-Path $manifestDir "plenum-app.toml"
    if (-not (Test-Path $manifestFile)) { continue }

    $binaryName = ""
    switch ($product.Crate) {
        "plenum-launcher" { $binaryName = "plenum-launcher.exe" }
        "inter-cube" { $binaryName = "inter-cube-daemon.exe" }
        "ninja-exec" { $binaryName = "ninja-exec.exe" }
        default { $binaryName = $product.Crate + ".exe" }
    }
    $binaryPath = Join-Path $releaseDir $binaryName
    if (-not (Test-Path $binaryPath)) {
        $pName = $product.Name
        Write-Log "  Warning: Binary $binaryName not found -- skipping $pName" "Yellow"
        continue
    }

    $pName = $product.Name
    Write-Log "  Building MSI for $pName ($archFlag)..." "White"
    $packOutput = & $plenumPackBin build --arch $archFlag --manifest-dir $manifestDir --binary-dir $releaseDir 2>&1
    if ($LASTEXITCODE -eq 0) {
        $packOutputDir = Join-Path $manifestDir "plenum-pack-output"
        $msiFiles = Get-ChildItem -Path $packOutputDir -Filter "*.msi" -ErrorAction SilentlyContinue
        if ($msiFiles) {
            foreach ($msi in $msiFiles) {
                $destMsi = Join-Path $OutputDir $msi.Name
                Copy-Item -Path $msi.FullName -Destination $destMsi -Force
                $generatedMSIs += @{ Path = $destMsi; Product = $pName; FileName = $msi.Name }
                $sizeMB = [math]::Round($msi.Length / 1MB, 1)
                $msiName = $msi.Name
                Write-Log "  [OK] $msiName ($sizeMB MB)" "Green"
            }
        } else {
            Write-Log "  Warning: No MSI produced for $pName." "Yellow"
            Write-Log "  WiX may have generated only .wxs files." "Yellow"
            $wxsFiles = Get-ChildItem -Path $packOutputDir -Filter "*.wxs" -ErrorAction SilentlyContinue
            if ($wxsFiles) {
                Write-Log "  Generated WiX source files:" "DarkGray"
                foreach ($wxs in $wxsFiles) {
                    $wxsPath = $wxs.FullName
                    Write-Log "    $wxsPath" "DarkGray"
                }
            }
        }
    } else {
        Write-Log "  Warning: MSI generation failed for $pName." "Yellow"
        $packOutput | ForEach-Object { Write-Log "    $_" "DarkGray" }
    }
}

Write-StepTime "MSI generation"

# == NinjaExec Passphrase Setup ================================================
# Prompt for NinjaExec passphrase using secure temp file pattern
$passphraseTempFile = $null
$ninjaExecMsi = $generatedMSIs | Where-Object { $_.Product -eq "NinjaExec" } | Select-Object -First 1

if ($ninjaExecMsi) {
    Write-Host ""
    Write-Host "==========================================================" -ForegroundColor Cyan
    Write-Host "  NinjaExec Passphrase Setup" -ForegroundColor Cyan
    Write-Host "==========================================================" -ForegroundColor Cyan
    Write-Host ""
    Write-Host "  Create a passphrase to protect your signing key." -ForegroundColor White
    Write-Host ""
    Write-Host "  This passphrase encrypts your NinjaExec identity." -ForegroundColor White
    Write-Host "  You will need it to authorize signing operations." -ForegroundColor White
    Write-Host ""
    Write-Host "  Choose something memorable -- at least 12 characters." -ForegroundColor Yellow
    Write-Host ""

    $passphraseAccepted = $false
    while (-not $passphraseAccepted) {
        $secPass = Read-Host "  Enter passphrase" -AsSecureString
        $bstr = [System.Runtime.InteropServices.Marshal]::SecureStringToBSTR($secPass)
        $plainPass = [System.Runtime.InteropServices.Marshal]::PtrToStringAuto($bstr)
        [System.Runtime.InteropServices.Marshal]::ZeroFreeBSTR($bstr)

        if ($plainPass.Length -lt 12) {
            Write-Host "  Passphrase must be at least 12 characters. You entered $($plainPass.Length)." -ForegroundColor Red
            $plainPass = ""
            continue
        }

        $secConfirm = Read-Host "  Confirm passphrase" -AsSecureString
        $bstrC = [System.Runtime.InteropServices.Marshal]::SecureStringToBSTR($secConfirm)
        $confirmPass = [System.Runtime.InteropServices.Marshal]::PtrToStringAuto($bstrC)
        [System.Runtime.InteropServices.Marshal]::ZeroFreeBSTR($bstrC)

        if ($plainPass -ne $confirmPass) {
            Write-Host "  Passphrases do not match. Please try again." -ForegroundColor Red
            $plainPass = ""
            $confirmPass = ""
            continue
        }
        $confirmPass = ""

        $passphraseAccepted = $true
        Write-Host "  Passphrase accepted -- $($plainPass.Length) characters." -ForegroundColor Green
        Write-Host ""

        $passphraseTempFile = Join-Path $env:TEMP "plenumnet-passphrase-$([guid]::NewGuid().ToString('N')).tmp"
        try {
            [System.IO.File]::WriteAllText($passphraseTempFile, $plainPass)
            $acl = Get-Acl $passphraseTempFile
            $acl.SetAccessRuleProtection($true, $false)
            $rule = New-Object System.Security.AccessControl.FileSystemAccessRule(
                [System.Security.Principal.WindowsIdentity]::GetCurrent().Name,
                "FullControl",
                "Allow"
            )
            $acl.AddAccessRule($rule)
            Set-Acl -Path $passphraseTempFile -AclObject $acl
        } catch {
            Write-Log "  Warning: Could not set ACL on passphrase file." "Yellow"
        }
        $plainPass = ""
    }
}

# == STEP 7/8: Install MSI packages (1-3 minutes) =============================
Write-Host ""
Write-Log "STEP 7/8: Installing PlenumNET products (1-3 minutes)" "Yellow"
Write-Host "---"

if ($generatedMSIs.Count -eq 0) {
    Write-Log "  No MSI files were generated. Skipping installation." "Yellow"
    Write-Log "  This may mean WiX could not compile the .wxs files." "Yellow"
    Write-Log "  Check that wix is on your PATH: dotnet tool install --global wix" "Yellow"
} else {
    # Install order: Inter-Cube Daemon FIRST (Launcher depends on it)
    $daemonMsi = $generatedMSIs | Where-Object { $_.Product -eq "InterCubeDaemon" } | Select-Object -First 1
    $launcherMsi = $generatedMSIs | Where-Object { $_.Product -eq "PlenumNET-Launcher" } | Select-Object -First 1
    $otherMsis = $generatedMSIs | Where-Object { $_.Product -ne "InterCubeDaemon" -and $_.Product -ne "PlenumNET-Launcher" }

    # Check if daemon is already running
    $daemonAlreadyRunning = $false
    try {
        $tcpTest = Test-NetConnection -ComputerName localhost -Port 11124 -WarningAction SilentlyContinue -ErrorAction SilentlyContinue
        if ($tcpTest.TcpTestSucceeded) {
            $daemonAlreadyRunning = $true
            Write-Log "  Existing Inter-Cube Daemon detected on port 11124." "Green"
            Write-Log "  Skipping daemon installation -- Launcher will connect" "Green"
            Write-Log "  to the existing cluster." "Green"
        }
    } catch {}

    # 1. Install Inter-Cube Daemon first (if not already running)
    if ($daemonMsi -and -not $daemonAlreadyRunning) {
        $msiPath = $daemonMsi.Path
        Write-Log "  Installing Inter-Cube Daemon..." "White"
        $msiLog = Join-Path $env:TEMP "PlenumNET_InterCubeDaemon_install.log"
        $proc = Start-Process -FilePath "msiexec.exe" -ArgumentList "/i `"$msiPath`" /qb /l*v `"$msiLog`"" -Wait -PassThru
        if ($proc.ExitCode -eq 0) {
            Write-Log "  [OK] Inter-Cube Daemon installed" "Green"

            # Start the daemon service and wait for it to be ready
            Write-Log "  Starting Inter-Cube Daemon service..." "White"
            try {
                & sc.exe start InterCubeDaemon 2>&1 | Out-Null
            } catch {}

            $daemonReady = $false
            $waitStart = Get-Date
            while (((Get-Date) - $waitStart).TotalSeconds -lt 30) {
                Start-Sleep -Seconds 2
                try {
                    $tcpTest = Test-NetConnection -ComputerName localhost -Port 11124 -WarningAction SilentlyContinue -ErrorAction SilentlyContinue
                    if ($tcpTest.TcpTestSucceeded) {
                        $daemonReady = $true
                        break
                    }
                } catch {}
            }

            if ($daemonReady) {
                Write-Log "  [OK] Inter-Cube Daemon is running on port 11124" "Green"
            } else {
                Write-Log "  Warning: Inter-Cube Daemon did not start within 30s." "Yellow"
                Write-Log "  The Launcher may show 'Connecting to daemon...' until" "Yellow"
                Write-Log "  the service starts. You can start it manually:" "Yellow"
                Write-Log "    sc.exe start InterCubeDaemon" "Yellow"
            }
        } else {
            $exitCode = $proc.ExitCode
            Write-Log "  Warning: Inter-Cube Daemon install exited with code $exitCode." "Yellow"
            Write-Log "  Log: $msiLog" "Yellow"
        }
    }

    # 2. Install PlenumNET Launcher
    if ($launcherMsi) {
        $msiPath = $launcherMsi.Path
        Write-Log "  Installing PlenumNET Launcher..." "White"
        $msiLog = Join-Path $env:TEMP "PlenumNET_Launcher_install.log"
        $proc = Start-Process -FilePath "msiexec.exe" -ArgumentList "/i `"$msiPath`" /qb /l*v `"$msiLog`"" -Wait -PassThru
        if ($proc.ExitCode -eq 0) {
            Write-Log "  [OK] PlenumNET Launcher installed" "Green"
        } else {
            $exitCode = $proc.ExitCode
            Write-Log "  Warning: Launcher install exited with code $exitCode." "Yellow"
            Write-Log "  Log: $msiLog" "Yellow"
        }
    }

    # 3. Install NinjaExec (with passphrase file if available)
    foreach ($msiEntry in $otherMsis) {
        $msiPath = $msiEntry.Path
        $msiBaseName = $msiEntry.Product
        Write-Log "  Installing $msiBaseName..." "White"
        $msiLog = Join-Path $env:TEMP "PlenumNET_${msiBaseName}_install.log"
        $msiArgs = "/i `"$msiPath`" /qb /l*v `"$msiLog`""
        if ($msiBaseName -eq "NinjaExec" -and $passphraseTempFile -and (Test-Path $passphraseTempFile)) {
            $msiArgs = "/i `"$msiPath`" /qb /l*v `"$msiLog`" PASSPHRASE_FILE=`"$passphraseTempFile`""
        }
        $proc = Start-Process -FilePath "msiexec.exe" -ArgumentList $msiArgs -Wait -PassThru
        if ($proc.ExitCode -eq 0) {
            Write-Log "  [OK] $msiBaseName installed" "Green"
        } else {
            $exitCode = $proc.ExitCode
            Write-Log "  Warning: $msiBaseName install exited with code $exitCode." "Yellow"
            Write-Log "  Log: $msiLog" "Yellow"
        }
    }
}

# Clean up passphrase temp file
if ($passphraseTempFile -and (Test-Path $passphraseTempFile)) {
    try {
        Remove-Item -Path $passphraseTempFile -Force -ErrorAction SilentlyContinue
    } catch {}
}

Write-StepTime "Installation"

# == STEP 8/8: Summary ========================================================
Write-Host ""
Write-Log "STEP 8/8: Summary" "Yellow"
Write-Host "==========================================================" -ForegroundColor Cyan
Write-Host ""

$elapsed = (Get-Date) - $installStart
$totalMins = [math]::Floor($elapsed.TotalMinutes)
$totalSecs = $elapsed.Seconds

$msiCount = $generatedMSIs.Count
if ($msiCount -gt 0) {
    Write-Host "  Installation complete. PlenumNET is ready." -ForegroundColor Green
    Write-Host ""
    Write-Host "  Your applications will appear in the system tray" -ForegroundColor White
    Write-Host "  momentarily." -ForegroundColor White
    Write-Host ""
    Write-Host "  Installed products:" -ForegroundColor Green

    foreach ($msiEntry in $generatedMSIs) {
        $msiFileName = $msiEntry.FileName
        Write-Host "    -- $msiFileName" -ForegroundColor Green
    }

    Write-Host ""
    Write-Host "  Install locations:" -ForegroundColor White
    Write-Host "    PlenumNET Launcher:  $programFiles\Capomastro\PlenumNET-Launcher" -ForegroundColor White
    Write-Host "    NinjaExec:           $programFiles\Capomastro\NinjaExec" -ForegroundColor White
    Write-Host "    Inter-Cube Daemon:   $programFiles\Capomastro\InterCubeDaemon" -ForegroundColor White
    Write-Host ""
    Write-Host "  Data directories (preserved on uninstall):" -ForegroundColor White
    Write-Host "    $appData\PlenumNET-Launcher" -ForegroundColor White
    Write-Host "    $appData\NinjaExec" -ForegroundColor White
    Write-Host "    $appData\InterCubeDaemon" -ForegroundColor White
    Write-Host ""
    Write-Host "  To get started:" -ForegroundColor White
    Write-Host "    https://plenumnet.replit.app/getting-started" -ForegroundColor Cyan
    Write-Host ""
    Write-Host "  To uninstall:" -ForegroundColor White
    Write-Host "    Settings > Apps > search 'PlenumNET' or 'Capomastro'" -ForegroundColor White
    Write-Host "    Uninstall order: NinjaExec first, then Launcher, then" -ForegroundColor DarkGray
    Write-Host "    Inter-Cube Daemon last. Daemon services are stopped" -ForegroundColor DarkGray
    Write-Host "    automatically. Data directories are preserved by" -ForegroundColor DarkGray
    Write-Host "    default for reinstallation." -ForegroundColor DarkGray
} else {
    Write-Log "  No MSIs were installed." "Yellow"
    Write-Log "  The WiX source files (.wxs) have been generated at:" "White"
    foreach ($product in $Products) {
        $wxsDir = Join-Path (Join-Path $RepoDir $product.ManifestDir) "plenum-pack-output"
        if (Test-Path $wxsDir) {
            Write-Log "    $wxsDir" "White"
        }
    }
    Write-Host ""
    Write-Log "  To complete MSI generation, ensure WiX v4 is on PATH:" "White"
    Write-Log "    dotnet tool install --global wix" "White"
    Write-Log "  Then re-run this installer." "White"
}

Write-Host ""
Write-Host "  Architecture:  $cpuArch" -ForegroundColor DarkGray
Write-Host "  Source:        $RepoDir" -ForegroundColor DarkGray
Write-Host "  MSI output:    $OutputDir" -ForegroundColor DarkGray
Write-Host "  Log file:      $LogFile" -ForegroundColor DarkGray
Write-Host "  Total time:    ${totalMins}m ${totalSecs}s" -ForegroundColor DarkGray
Write-Host ""
Write-Host "==========================================================" -ForegroundColor Cyan
Write-Host "  Thank you for choosing Capomastro." -ForegroundColor Cyan
Write-Host "  Capomastro Holdings Ltd. -- Applied Physics Division" -ForegroundColor Cyan
Write-Host "==========================================================" -ForegroundColor Cyan
Write-Host ""
Read-Host "Press Enter to close"
