<#
.SYNOPSIS
    PlenumNET MSI Installer - One-Click Setup
    Downloads, builds, and installs PlenumNET products via the
    plenum-pack MSI framework.

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

$ErrorActionPreference = "Continue"
$REMOTE_CRS     = "https://plenumnet.replit.app"
$RepoDir        = "C:\PlenumNET"
$RepoUrl        = "https://github.com/SigmaWolf-8/Ternary.git"
$OutputDir      = Join-Path $RepoDir "installer-output"
$LogFile        = Join-Path $env:TEMP "PlenumNET_MSI_Install.log"

$Products = @(
    @{ Name = "PlenumNET-Launcher"; Crate = "plenum-launcher"; ManifestDir = "tools/plenum-launcher"; ExtraCrates = @("plenum-launcher-elevate") }
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

function Test-Command {
    param([string]$Name)
    $cmd = Get-Command $Name -ErrorAction SilentlyContinue
    return ($null -ne $cmd)
}

Write-Host ""
Write-Host "==========================================================" -ForegroundColor Cyan
Write-Host "  PlenumNET MSI Installer" -ForegroundColor Cyan
Write-Host "  Capomastro Holdings Ltd. - Applied Physics Division" -ForegroundColor Cyan
Write-Host "==========================================================" -ForegroundColor Cyan
Write-Host ""
Write-Host "  This script builds and installs PlenumNET products" -ForegroundColor White
Write-Host "  using the plenum-pack MSI framework." -ForegroundColor White
Write-Host ""
Write-Host "  Products: Launcher" -ForegroundColor White
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

# == Fix PATH for Admin elevation =============================================
# When running as Admin, the shell may not inherit user-level PATH entries.
# We prepend the user's cargo, dotnet tools, and LLVM paths so the correct
# versions of rustc, cargo, wix, and clang are found.
$userCargoBin = Join-Path $env:USERPROFILE ".cargo\bin"
if (Test-Path $userCargoBin) {
    $env:PATH = $userCargoBin + ";" + $env:PATH
}
$env:RUSTUP_HOME = Join-Path $env:USERPROFILE ".rustup"
$env:CARGO_HOME = Join-Path $env:USERPROFILE ".cargo"
$userDotnetTools = Join-Path $env:USERPROFILE ".dotnet\tools"
if (Test-Path $userDotnetTools) {
    $env:PATH = $userDotnetTools + ";" + $env:PATH
}

# == Detect architecture =======================================================
$cpuArch = $env:PROCESSOR_ARCHITECTURE
if (-not $cpuArch) { $cpuArch = "AMD64" }

$rustTarget = "x86_64-pc-windows-msvc"
$archFlag = "x86_64"
if ($cpuArch -eq "ARM64") {
    $rustTarget = "aarch64-pc-windows-msvc"
    $archFlag = "aarch64"
}
Write-Log "  Architecture: $cpuArch (Rust target: $rustTarget)" "White"

# == STEP 1: Prerequisites =====================================================
Write-Host ""
Write-Log "STEP 1/8: Checking prerequisites" "Yellow"
Write-Host "---"

# Git
if (-not (Test-Command "git")) {
    Write-Log "  ERROR: git is not installed. Install from https://git-scm.com/download/win" "Red"
    Read-Host "Press Enter to close"
    exit 1
}
Write-Log "  [OK] git" "Green"

# Rust — find it, update it, verify the version is new enough
if (-not (Test-Command "cargo")) {
    Write-Log "  -> Rust not found - installing rustup..." "Yellow"
    $rustupExe = Join-Path $env:TEMP "rustup-init.exe"
    Invoke-WebRequest -Uri "https://win.rustup.rs/x86_64" -OutFile $rustupExe -UseBasicParsing
    Start-Process -FilePath $rustupExe -ArgumentList "-y" -Wait -NoNewWindow
    Remove-Item $rustupExe -Force -ErrorAction SilentlyContinue
    $cargoBin = Join-Path $env:USERPROFILE ".cargo\bin"
    $env:PATH = $cargoBin + ";" + $env:PATH
    if (-not (Test-Command "cargo")) {
        Write-Log "  ERROR: cargo not found after install - restart terminal and re-run." "Red"
        Read-Host "Press Enter to close"
        exit 1
    }
}

Write-Log "  Updating Rust toolchain to latest stable..." "White"
$rustupBin = Join-Path $userCargoBin "rustup.exe"
if (Test-Path $rustupBin) {
    & $rustupBin update stable 2>&1 | Out-Null
    & $rustupBin default stable 2>&1 | Out-Null
} else {
    & rustup update stable 2>&1 | Out-Null
    & rustup default stable 2>&1 | Out-Null
}

$cargoExe = Join-Path $userCargoBin "cargo.exe"
if (-not (Test-Path $cargoExe)) {
    $cargoExe = "cargo"
}
$rustcExe = Join-Path $userCargoBin "rustc.exe"
if (-not (Test-Path $rustcExe)) {
    $rustcExe = "rustc"
}

$rustcVer = & $rustcExe --version 2>&1
Write-Log "  [OK] $rustcVer" "Green"
$cargoVer = & $cargoExe --version 2>&1
Write-Log "  [OK] $cargoVer" "Green"
Write-Log "  cargo path: $cargoExe" "DarkGray"

# Verify rustc is new enough (need >= 1.82)
$verMatch = [regex]::Match("$rustcVer", "(\d+)\.(\d+)")
if ($verMatch.Success) {
    $major = [int]$verMatch.Groups[1].Value
    $minor = [int]$verMatch.Groups[2].Value
    if ($major -eq 1 -and $minor -lt 82) {
        Write-Log "  ERROR: rustc $major.$minor is too old. Need 1.82+. Update failed." "Red"
        Write-Log "  Try running: rustup update stable" "Yellow"
        Read-Host "Press Enter to close"
        exit 1
    }
}

# MSVC build tools
$vsWhere = $null
$progX86 = [System.Environment]::GetFolderPath("ProgramFilesX86")
if (-not $progX86) { $progX86 = "C:\Program Files (x86)" }
$progFiles = $env:ProgramFiles
if (-not $progFiles) { $progFiles = "C:\Program Files" }

$searchPaths = @(
    (Join-Path $progX86 "Microsoft Visual Studio\Installer\vswhere.exe"),
    (Join-Path $progFiles "Microsoft Visual Studio\Installer\vswhere.exe"),
    "C:\Program Files (x86)\Microsoft Visual Studio\Installer\vswhere.exe",
    "C:\Program Files\Microsoft Visual Studio\Installer\vswhere.exe"
)
foreach ($p in $searchPaths) {
    if (Test-Path -LiteralPath $p) { $vsWhere = $p; break }
}
if ($vsWhere) {
    try {
        $vsPath = & $vsWhere -latest -products * -property installationPath 2>$null
        if ($vsPath) {
            $vcvarsName = "vcvars64.bat"
            if ($cpuArch -eq "ARM64") { $vcvarsName = "vcvarsarm64.bat" }
            $vcvars = Join-Path $vsPath "VC\Auxiliary\Build"
            $vcvars = Join-Path $vcvars $vcvarsName
            if (Test-Path -LiteralPath $vcvars) {
                $cmdCall = "`"$vcvars`" > nul 2>&1 && set"
                $envLines = cmd.exe /c $cmdCall
                foreach ($line in $envLines) {
                    if ($line -match '^([^=]+)=(.*)$') {
                        [System.Environment]::SetEnvironmentVariable($Matches[1], $Matches[2], "Process")
                    }
                }
                Write-Log "  [OK] MSVC environment activated" "Green"
            }
        }
    } catch {
        Write-Log "  WARN: Could not activate MSVC environment." "Yellow"
    }
} else {
    Write-Log "  WARN: Visual Studio / MSVC Build Tools not found." "Yellow"
    Write-Log "        Install Desktop development with C++ workload." "Yellow"
    Write-Log "        Continuing - build may fail without MSVC linker." "Yellow"
}

# LLVM/clang
$hasClang = Test-Command "clang"
if (-not $hasClang) {
    $llvmBin = "C:\Program Files\LLVM\bin"
    $llvmClang = Join-Path $llvmBin "clang.exe"
    if (Test-Path -LiteralPath $llvmClang) {
        $env:PATH = $env:PATH + ";" + $llvmBin
        $hasClang = Test-Command "clang"
    }
}
if ($hasClang) {
    Write-Log "  [OK] clang" "Green"
    $env:CC = "clang"
    $env:AR = "llvm-ar"
} else {
    Write-Log "  -> clang not found - installing LLVM..." "Yellow"
    $hasWinget = Test-Command "winget"
    if ($hasWinget) {
        try {
            winget install --id LLVM.LLVM --silent --accept-package-agreements --accept-source-agreements 2>&1 | Out-Null
        } catch {
            Write-Log "  WARN: winget install failed, trying direct download..." "Yellow"
        }
    }
    $hasClang = Test-Command "clang"
    if (-not $hasClang) {
        $llvmBin = "C:\Program Files\LLVM\bin"
        $llvmClang = Join-Path $llvmBin "clang.exe"
        if (Test-Path -LiteralPath $llvmClang) {
            $env:PATH = $env:PATH + ";" + $llvmBin
            $hasClang = Test-Command "clang"
        }
    }
    if (-not $hasClang) {
        $machinePath = [System.Environment]::GetEnvironmentVariable("PATH", "Machine")
        $userPath = [System.Environment]::GetEnvironmentVariable("PATH", "User")
        $env:PATH = $machinePath + ";" + $userPath
        $llvmBin = "C:\Program Files\LLVM\bin"
        $llvmClang = Join-Path $llvmBin "clang.exe"
        if (Test-Path -LiteralPath $llvmClang) {
            $env:PATH = $env:PATH + ";" + $llvmBin
        }
        $hasClang = Test-Command "clang"
    }
    if ($hasClang) {
        Write-Log "  [OK] clang installed" "Green"
        $env:CC = "clang"
        $env:AR = "llvm-ar"
    } else {
        Write-Log "  ERROR: clang not found. Install LLVM from https://releases.llvm.org and re-run." "Red"
        Read-Host "Press Enter to close"
        exit 1
    }
}

# .NET SDK (required for WiX v4)
if (-not (Test-Command "dotnet")) {
    Write-Log "  -> .NET SDK not found - installing..." "Yellow"
    $hasWinget = Test-Command "winget"
    if ($hasWinget) {
        try {
            winget install --id Microsoft.DotNet.SDK.8 --silent --accept-package-agreements --accept-source-agreements 2>&1 | Out-Null
        } catch {
            Write-Log "  WARN: winget install of .NET failed." "Yellow"
        }
    }
    if (-not (Test-Command "dotnet")) {
        $dotnetInstaller = Join-Path $env:TEMP "dotnet-install.ps1"
        try {
            Invoke-WebRequest -Uri "https://dot.net/v1/dotnet-install.ps1" -OutFile $dotnetInstaller -UseBasicParsing
            & $dotnetInstaller -Channel 8.0 -InstallDir (Join-Path $env:ProgramFiles "dotnet")
            Remove-Item $dotnetInstaller -Force -ErrorAction SilentlyContinue
        } catch {
            Write-Log "  WARN: .NET installer download failed." "Yellow"
        }
    }
    $machinePath = [System.Environment]::GetEnvironmentVariable("PATH", "Machine")
    $userPath = [System.Environment]::GetEnvironmentVariable("PATH", "User")
    $env:PATH = $machinePath + ";" + $userPath
    $dotnetDir = Join-Path $env:ProgramFiles "dotnet"
    $dotnetExe = Join-Path $dotnetDir "dotnet.exe"
    if (Test-Path $dotnetExe) {
        $env:PATH = $env:PATH + ";" + $dotnetDir
    }
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
    $wixList = & dotnet tool list --global 2>$null
    if ($wixList) {
        foreach ($wixLine in $wixList) {
            if ($wixLine -match "^wix\s") { $wixAvailable = $true; break }
        }
    }
} catch {}

$dotnetToolsPath = Join-Path $env:USERPROFILE ".dotnet\tools"
if (Test-Path $dotnetToolsPath) {
    $env:PATH = $env:PATH + ";" + $dotnetToolsPath
}

if (-not $wixAvailable) {
    Write-Log "  -> WiX Toolset v4 not found - installing..." "Yellow"
    try {
        & dotnet tool install --global wix 2>&1 | Out-Null
    } catch {
        Write-Log "  WARN: WiX install command failed." "Yellow"
    }
    if (Test-Path $dotnetToolsPath) {
        $env:PATH = $env:PATH + ";" + $dotnetToolsPath
    }
}

if (Test-Command "wix") {
    $wixVerRaw = (& wix --version 2>$null) | Out-String
    $wixVerRaw = $wixVerRaw.Trim()
    if ($wixVerRaw -match "^(\d+\.\d+\.\d+)") {
        $wixVerClean = $Matches[1]
    } else {
        $wixVerClean = "6.0.2"
    }
    if ($wixVerRaw -match "^(\d+)\.") {
        $wixMajor = $Matches[1]
    } else {
        $wixMajor = "6"
    }
    Write-Log "  [OK] WiX v$wixMajor ($wixVerClean)" "Green"

    $wixExtensions = @("WixToolset.UI.wixext", "WixToolset.Util.wixext")
    foreach ($ext in $wixExtensions) {
        & wix extension remove --global $ext 2>&1 | Out-Null
        $extVersioned = "$ext/$wixVerClean"
        Write-Log "  Installing $extVersioned..." "DarkGray"
        $extOut = & wix extension add --global $extVersioned 2>&1
        $extStr = ($extOut | Out-String).Trim()
        if ($LASTEXITCODE -ne 0) {
            if ($extStr -match "already") {
                Write-Log "  [OK] $ext (already installed)" "Green"
            } else {
                Write-Log "  WARN: $ext install failed: $extStr" "Yellow"
            }
        } else {
            Write-Log "  [OK] $ext v$wixVerClean installed" "Green"
        }
    }
} else {
    Write-Log "  WARN: WiX not on PATH - MSI generation may produce .wxs only." "Yellow"
    Write-Log "        To fix: dotnet tool install --global wix" "Yellow"
    Write-Log "        Then add $dotnetToolsPath to your PATH." "Yellow"
    Write-Log "        Continuing anyway..." "Yellow"
}

# == STEP 2: Clone/Update Source ===============================================
Write-Host ""
Write-Log "STEP 2/8: Source code" "Yellow"
Write-Host "---"

if (-not (Test-Path $RepoDir)) {
    Write-Log "  Cloning PlenumNET repository..." "White"
    $cloneOut = & git clone $RepoUrl $RepoDir 2>&1
    if ($LASTEXITCODE -ne 0) {
        Write-Log "  ERROR: git clone failed." "Red"
        $cloneOut | ForEach-Object { Write-Log "    $_" "DarkGray" }
        Read-Host "Press Enter to close"
        exit 1
    }
} elseif (-not (Test-Path (Join-Path $RepoDir ".git"))) {
    Write-Log "  Directory exists but is not a git repo. Initializing..." "Yellow"
    Push-Location $RepoDir
    & git init 2>&1 | Out-Null
    & git remote add origin $RepoUrl 2>&1 | Out-Null
    & git fetch origin main 2>&1 | Out-Null
    if ($LASTEXITCODE -ne 0) {
        Write-Log "  ERROR: git fetch failed." "Red"
        Pop-Location
        Read-Host "Press Enter to close"
        exit 1
    }
    & git checkout -B main origin/main --force 2>&1 | Out-Null
    & git reset --hard origin/main 2>&1 | Out-Null
    Pop-Location
} else {
    Write-Log "  Updating source..." "White"
    Push-Location $RepoDir
    $shallowFile = Join-Path $RepoDir ".git\shallow"
    if (Test-Path $shallowFile) {
        Write-Log "  Detected shallow clone - unshallowing..." "Yellow"
        & git fetch --unshallow origin 2>&1 | Out-Null
    }
    & git fetch origin main 2>&1 | Out-Null
    & git checkout main 2>&1 | Out-Null
    & git reset --hard origin/main 2>&1 | Out-Null
    Pop-Location
}
Write-Log "  [OK] Source ready at $RepoDir" "Green"

$plenumPackToml = Join-Path $RepoDir "tools\plenum-pack\Cargo.toml"
if (-not (Test-Path $plenumPackToml)) {
    Write-Log "  tools\plenum-pack not found - repo may be stale. Re-cloning..." "Yellow"
    $backupDir = $RepoDir + "-backup-" + (Get-Date -Format "yyyyMMdd-HHmmss")
    Write-Log "  Backing up existing repo to $backupDir" "White"
    Rename-Item -Path $RepoDir -NewName $backupDir -Force
    & git clone $RepoUrl $RepoDir 2>&1 | Out-Null
    if ($LASTEXITCODE -ne 0) {
        Write-Log "  ERROR: Re-clone failed." "Red"
        Read-Host "Press Enter to close"
        exit 1
    }
    $plenumPackToml = Join-Path $RepoDir "tools\plenum-pack\Cargo.toml"
    if (-not (Test-Path $plenumPackToml)) {
        Write-Log "  ERROR: tools\plenum-pack still not found after fresh clone." "Red"
        Write-Log "  The GitHub repo may not contain this package yet." "Red"
        Read-Host "Press Enter to close"
        exit 1
    }
    Write-Log "  [OK] Fresh clone has tools\plenum-pack" "Green"
}

# == STEP 3: Build plenum-pack ================================================
Write-Host ""
Write-Log "STEP 3/8: Building plenum-pack (MSI build tool)" "Yellow"
Write-Host "---"

Push-Location $RepoDir
Write-Log "  Updating dependency lock file (this may take a few minutes)..." "White"
& $cargoExe update 2>&1 | ForEach-Object {
    $line = $_.ToString()
    if ($line -match "Updating|Adding|Removing") { Write-Log "    $line" "DarkGray" }
}
$env:CARGO_BUILD_JOBS = "1"
Write-Log "  Compiling plenum-pack (release)..." "White"
$buildOutput = & $cargoExe build --release -p plenum-pack 2>&1
$buildExit = $LASTEXITCODE
$buildOutput | ForEach-Object {
    $line = $_.ToString()
    if ($line -match "error") { Write-Log "  $line" "Red" }
    elseif ($line -match "Compiling|Finished") { Write-Log "  $line" "DarkGray" }
}
Pop-Location

$plenumPackBin = Join-Path $RepoDir "target\release\plenum-pack.exe"
if (($buildExit -ne 0) -or (-not (Test-Path $plenumPackBin))) {
    Write-Log "  ERROR: plenum-pack build failed." "Red"
    Read-Host "Press Enter to close"
    exit 1
}
Write-Log "  [OK] plenum-pack built" "Green"

# == STEP 4: Build product binaries ============================================
Write-Host ""
Write-Log "STEP 4/8: Building product binaries" "Yellow"
Write-Host "---"

$allCrates = @("plenum-launcher", "plenum-launcher-elevate")
foreach ($crate in $allCrates) {
    Write-Log "  Building $crate..." "White"
    Push-Location $RepoDir
    $crateOutput = & $cargoExe build --release -p $crate 2>&1
    $crateBuildExit = $LASTEXITCODE
    if ($crateBuildExit -ne 0) {
        $crateOutput | ForEach-Object { Write-Log "  $_" "Red" }
    } else {
        $crateOutput | ForEach-Object {
            $line = $_.ToString()
            if ($line -match "Compiling|Finished") { Write-Log "  $line" "DarkGray" }
        }
    }
    Pop-Location
    if ($crateBuildExit -ne 0) {
        Write-Log "  WARN: $crate build failed - skipping this product." "Yellow"
    } else {
        Write-Log "  [OK] $crate built" "Green"
    }
}

# == STEP 5: Validate manifests ================================================
Write-Host ""
Write-Log "STEP 5/8: Validating product manifests" "Yellow"
Write-Host "---"

foreach ($product in $Products) {
    $manifestDir = Join-Path $RepoDir $product.ManifestDir
    $manifestFile = Join-Path $manifestDir "plenum-app.toml"
    if (-not (Test-Path $manifestFile)) {
        $pName = $product.Name
        Write-Log "  WARN: Manifest not found - skipping $pName" "Yellow"
        continue
    }
    $pName = $product.Name
    Write-Log "  Validating $pName..." "White"
    try {
        $validateOutput = & $plenumPackBin validate --manifest-dir $manifestDir 2>&1
        if ($LASTEXITCODE -eq 0) {
            Write-Log "  [OK] $pName manifest valid" "Green"
        } else {
            Write-Log "  WARN: $pName validation issues" "Yellow"
            $validateOutput | ForEach-Object { Write-Log "    $_" "DarkGray" }
        }
    } catch {
        Write-Log "  WARN: Could not validate $pName" "Yellow"
    }
}

# == STEP 6: Generate MSI installers ==========================================
Write-Host ""
Write-Log "STEP 6/8: Generating MSI installers" "Yellow"
Write-Host "---"

New-Item -ItemType Directory -Force -Path $OutputDir | Out-Null
$generatedMSIs = @()
$releaseDir = Join-Path $RepoDir "target\release"
$wixOnPath = Test-Command "wix"
if (-not $wixOnPath) {
    Write-Log "  WiX not on PATH - will generate .wxs source only (--dry-run)" "Yellow"
}

foreach ($product in $Products) {
    $manifestDir = Join-Path $RepoDir $product.ManifestDir
    $manifestFile = Join-Path $manifestDir "plenum-app.toml"
    if (-not (Test-Path $manifestFile)) { continue }

    $binaryName = ""
    switch ($product.Crate) {
        "plenum-launcher" { $binaryName = "plenum-launcher.exe" }
        "inter-cube" { $binaryName = "inter-cube-daemon.exe" }
        default { $binaryName = $product.Crate + ".exe" }
    }
    $binaryPath = Join-Path $releaseDir $binaryName
    if (-not (Test-Path $binaryPath)) {
        $pName = $product.Name
        Write-Log "  WARN: Binary $binaryName not found - skipping $pName" "Yellow"
        continue
    }

    $pName = $product.Name
    $dryRunFlag = ""
    if (-not $wixOnPath) { $dryRunFlag = "--dry-run" }
    Write-Log "  Building MSI for $pName ($archFlag)..." "White"
    try {
        $packArgs = @("build", "--arch", $archFlag, "--manifest-dir", $manifestDir, "--binary-dir", $releaseDir)
        if ($dryRunFlag) { $packArgs += $dryRunFlag }
        $packOutput = & $plenumPackBin @packArgs 2>&1
        if ($LASTEXITCODE -eq 0) {
            $packOutputDir = Join-Path $manifestDir "plenum-pack-output"
            $msiFiles = Get-ChildItem -Path $packOutputDir -Filter "*.msi" -ErrorAction SilentlyContinue
            if ($msiFiles) {
                foreach ($msi in $msiFiles) {
                    $destMsi = Join-Path $OutputDir $msi.Name
                    Copy-Item -Path $msi.FullName -Destination $destMsi -Force
                    $generatedMSIs += $destMsi
                    $sizeMB = [math]::Round($msi.Length / 1MB, 1)
                    $msiName = $msi.Name
                    Write-Log "  [OK] $msiName ($sizeMB MB)" "Green"
                }
            } else {
                Write-Log "  WARN: No .msi produced for $pName (WiX may have generated .wxs only)." "Yellow"
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
            Write-Log "  WARN: plenum-pack build failed for $pName" "Yellow"
            $packOutput | ForEach-Object { Write-Log "    $_" "DarkGray" }
        }
    } catch {
        $errMsg = $_.Exception.Message
        Write-Log "  WARN: Exception building MSI for $pName - $errMsg" "Yellow"
    }
}

# == STEP 7: Install MSIs =====================================================
Write-Host ""
Write-Log "STEP 7/8: Installing PlenumNET products" "Yellow"
Write-Host "---"

if ($generatedMSIs.Count -eq 0) {
    Write-Log "  No MSI files were generated. Skipping installation." "Yellow"
    Write-Log "  This may mean WiX could not compile the .wxs files." "Yellow"
    Write-Log "  Fix: dotnet tool install --global wix" "Yellow"
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
            $exitCode = $proc.ExitCode
            Write-Log "  WARN: Launcher install exited with code $exitCode - see $msiLog" "Yellow"
        }
    }

    foreach ($msi in $otherMsis) {
        $msiBaseName = [System.IO.Path]::GetFileNameWithoutExtension($msi)
        Write-Log "  Installing $msiBaseName..." "White"
        $msiLog = Join-Path $env:TEMP ("PlenumNET_" + $msiBaseName + "_install.log")
        $proc = Start-Process -FilePath "msiexec.exe" -ArgumentList "/i `"$msi`" /qb /l*v `"$msiLog`"" -Wait -PassThru
        if ($proc.ExitCode -eq 0) {
            Write-Log "  [OK] $msiBaseName installed" "Green"
        } else {
            $exitCode = $proc.ExitCode
            Write-Log "  WARN: $msiBaseName install exited with code $exitCode - see $msiLog" "Yellow"
        }
    }
}

# == STEP 8: Summary ==========================================================
Write-Host ""
Write-Log "STEP 8/8: Installation Summary" "Yellow"
Write-Host "==========================================================" -ForegroundColor Cyan

Write-Host ""
Write-Log "  Architecture   : $cpuArch" "White"
Write-Log "  Source          : $RepoDir" "White"
Write-Log "  MSI output      : $OutputDir" "White"
Write-Log "  Install log     : $LogFile" "White"
Write-Host ""

$msiCount = $generatedMSIs.Count
if ($msiCount -gt 0) {
    Write-Log "  Installed MSIs:" "Green"
    foreach ($msi in $generatedMSIs) {
        $msiFileName = [System.IO.Path]::GetFileName($msi)
        Write-Log "    - $msiFileName" "Green"
    }
    Write-Host ""
    Write-Log "  PlenumNET products installed under:" "White"
    Write-Log "    %ProgramFiles%\Capomastro\" "White"
    Write-Host ""
    Write-Log "  Data directories (preserved on uninstall):" "White"
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
    Write-Host ""
    Write-Log "  To complete MSI generation, ensure WiX v4 is on PATH:" "White"
    Write-Log "    dotnet tool install --global wix" "White"
    Write-Log "  Then re-run this installer." "White"
}

Write-Host ""
Write-Host "==========================================================" -ForegroundColor Cyan
Write-Host "  PlenumNET MSI Installation Complete" -ForegroundColor Cyan
Write-Host "  Capomastro Holdings Ltd. - Applied Physics Division" -ForegroundColor Cyan
Write-Host "==========================================================" -ForegroundColor Cyan
Write-Host ""
Read-Host "Press Enter to close"
