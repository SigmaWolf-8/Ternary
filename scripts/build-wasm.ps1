#Requires -Version 5.1
<#
.SYNOPSIS
    PlenumNET WASM Build Script — ternary-math crate
    Copyright (c) 2025-2026 Capomastro Holdings Ltd.

.DESCRIPTION
    Validates all build dependencies, installs missing ones, and produces
    the WASM package (pkg/ternary_math.js + ternary_math_bg.wasm).

    Dependencies checked:
      1. Rust toolchain (via rustup)
      2. wasm32-unknown-unknown target
      3. C/C++ compiler (clang or MSVC cl.exe via VS Build Tools)
      4. wasm-pack
      5. Node.js (for smoke test)

    Run from repo root:  .\scripts\build-wasm.ps1
    Run from ternary-math:  ..\scripts\build-wasm.ps1

.PARAMETER SkipTest
    Skip the Node.js smoke test after building.

.PARAMETER Force
    Reinstall dependencies even if already present.
#>
param(
    [switch]$SkipTest,
    [switch]$Force
)

$ErrorActionPreference = "Stop"
$ProgressPreference = "SilentlyContinue"

$WASM_PACK_MIN = "0.13.1"
$RUST_MIN = "1.77.0"

function Write-Step  { param($n,$msg) Write-Host "[$n] $msg" -ForegroundColor Yellow }
function Write-OK    { param($msg) Write-Host "  OK  $msg" -ForegroundColor Green }
function Write-Warn  { param($msg) Write-Host "  WARN $msg" -ForegroundColor DarkYellow }
function Write-Fail  { param($msg) Write-Host "  FAIL $msg" -ForegroundColor Red }
function Write-Info  { param($msg) Write-Host "  ... $msg" -ForegroundColor Gray }

function Compare-SemVer {
    param([string]$Current, [string]$Minimum)
    $c = $Current.Split('.') | ForEach-Object { [int]$_ }
    $m = $Minimum.Split('.') | ForEach-Object { [int]$_ }
    for ($i = 0; $i -lt 3; $i++) {
        if ($c[$i] -gt $m[$i]) { return 1 }
        if ($c[$i] -lt $m[$i]) { return -1 }
    }
    return 0
}

function Refresh-Path {
    $machPath = [Environment]::GetEnvironmentVariable("Path", "Machine")
    $userPath = [Environment]::GetEnvironmentVariable("Path", "User")
    $env:Path = "$userPath;$machPath"
}

function Find-RepoRoot {
    $dir = Get-Location
    while ($dir) {
        if (Test-Path (Join-Path $dir "Cargo.toml")) {
            $content = Get-Content (Join-Path $dir "Cargo.toml") -Raw -ErrorAction SilentlyContinue
            if ($content -match '\[workspace\]') { return $dir.ToString() }
        }
        $parent = Split-Path $dir -Parent
        if ($parent -eq $dir) { break }
        $dir = $parent
    }
    return $null
}

Write-Host ""
Write-Host "==========================================" -ForegroundColor Cyan
Write-Host " PlenumNET WASM Build System" -ForegroundColor Cyan
Write-Host " Capomastro Holdings Ltd." -ForegroundColor Cyan
Write-Host "==========================================" -ForegroundColor Cyan
Write-Host ""

$repoRoot = Find-RepoRoot
if (-not $repoRoot) {
    $repoRoot = $PSScriptRoot | Split-Path -Parent
}
$crateDir = Join-Path $repoRoot "ternary-math"
if (-not (Test-Path (Join-Path $crateDir "Cargo.toml"))) {
    Write-Fail "Cannot find ternary-math/Cargo.toml from $repoRoot"
    exit 1
}
Write-Host "Repo root:    $repoRoot"
Write-Host "Crate dir:    $crateDir"
Write-Host ""

$installCount = 0
$errorCount = 0

Write-Step "1/5" "Checking Rust toolchain..."
$rustup = Get-Command rustup -ErrorAction SilentlyContinue
if (-not $rustup) {
    Write-Info "rustup not found — installing Rust toolchain..."
    $rustupInit = Join-Path $env:TEMP "rustup-init.exe"
    try {
        Invoke-WebRequest -Uri "https://win.rustup.rs/aarch64" -OutFile $rustupInit -UseBasicParsing
        if (-not (Test-Path $rustupInit)) {
            Invoke-WebRequest -Uri "https://win.rustup.rs/x86_64" -OutFile $rustupInit -UseBasicParsing
        }
        & $rustupInit -y --default-toolchain stable 2>&1 | Out-Null
        Refresh-Path
        $installCount++
    } catch {
        Write-Fail "Failed to install Rust: $_"
        $errorCount++
    }
}
$rustup = Get-Command rustup -ErrorAction SilentlyContinue
if ($rustup) {
    $rustVer = (rustc --version 2>&1) -replace '.*?(\d+\.\d+\.\d+).*','$1'
    $cmp = Compare-SemVer $rustVer $RUST_MIN
    if ($cmp -lt 0) {
        Write-Warn "Rust $rustVer below minimum $RUST_MIN — updating..."
        rustup update stable 2>&1 | Out-Null
        Refresh-Path
        $rustVer = (rustc --version 2>&1) -replace '.*?(\d+\.\d+\.\d+).*','$1'
    }
    Write-OK "Rust $rustVer (minimum: $RUST_MIN)"
} else {
    Write-Fail "Rust toolchain not available after install attempt"
    $errorCount++
}

Write-Step "2/5" "Checking wasm32-unknown-unknown target..."
$targets = rustup target list --installed 2>&1
if ($targets -notmatch 'wasm32-unknown-unknown') {
    Write-Info "Adding wasm32-unknown-unknown target..."
    rustup target add wasm32-unknown-unknown 2>&1 | Out-Null
    $installCount++
}
Write-OK "wasm32-unknown-unknown target installed"

Write-Step "3/5" "Checking C/C++ compiler..."
$hasCompiler = $false
$clang = Get-Command clang -ErrorAction SilentlyContinue
if ($clang) {
    $clangVer = (clang --version 2>&1 | Select-Object -First 1) -replace '.*?(\d+\.\d+\.\d+).*','$1'
    Write-OK "clang $clangVer"
    $hasCompiler = $true
}

if (-not $hasCompiler) {
    $cl = Get-Command cl -ErrorAction SilentlyContinue
    if ($cl) {
        Write-OK "MSVC cl.exe found"
        $hasCompiler = $true
    }
}

if (-not $hasCompiler) {
    $vsWhere = "${env:ProgramFiles(x86)}\Microsoft Visual Studio\Installer\vswhere.exe"
    if (Test-Path $vsWhere) {
        $vsPath = & $vsWhere -latest -products * -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath 2>$null
        if ($vsPath) {
            $vcvars = Get-ChildItem -Path $vsPath -Recurse -Filter "vcvarsall.bat" -ErrorAction SilentlyContinue | Select-Object -First 1
            if ($vcvars) {
                Write-OK "VS Build Tools found at $vsPath (vcvarsall.bat available)"
                Write-Warn "cl.exe not in PATH — run from Developer Command Prompt or execute:"
                Write-Warn "  cmd /c `"$($vcvars.FullName)`" amd64 `"&&`" powershell -File $($MyInvocation.MyCommand.Path)"
                $hasCompiler = $true
            }
        }
    }
}

if (-not $hasCompiler) {
    Write-Info "No C compiler found — installing LLVM/Clang via winget..."
    $winget = Get-Command winget -ErrorAction SilentlyContinue
    if ($winget) {
        try {
            winget install LLVM.LLVM --accept-package-agreements --accept-source-agreements 2>&1 | Out-Null
            Refresh-Path
            $clangCheck = Get-Command clang -ErrorAction SilentlyContinue
            if ($clangCheck) {
                Write-OK "LLVM/Clang installed successfully"
                $hasCompiler = $true
                $installCount++
            } else {
                $llvmBin = "C:\Program Files\LLVM\bin"
                if (Test-Path $llvmBin) {
                    $env:Path = "$llvmBin;$env:Path"
                    Write-OK "LLVM installed at $llvmBin (added to session PATH)"
                    $hasCompiler = $true
                    $installCount++
                } else {
                    Write-Fail "LLVM installed but clang not found in PATH"
                    Write-Info "Restart your terminal and re-run this script"
                    $errorCount++
                }
            }
        } catch {
            Write-Fail "winget install failed: $_"
            $errorCount++
        }
    } else {
        Write-Fail "No C compiler and winget not available"
        Write-Info "Install one of:"
        Write-Info "  1. LLVM/Clang:  winget install LLVM.LLVM"
        Write-Info "  2. VS Build Tools: winget install Microsoft.VisualStudio.2022.BuildTools --add Microsoft.VisualStudio.Component.VC.Tools.x86.x64"
        $errorCount++
    }
}

Write-Step "4/5" "Checking wasm-pack..."
$wasmPack = Get-Command wasm-pack -ErrorAction SilentlyContinue
if ($wasmPack -and -not $Force) {
    $wpVer = (wasm-pack --version 2>&1) -replace '.*?(\d+\.\d+\.\d+).*','$1'
    $cmp = Compare-SemVer $wpVer $WASM_PACK_MIN
    if ($cmp -lt 0) {
        Write-Warn "wasm-pack $wpVer below minimum $WASM_PACK_MIN — reinstalling..."
        $wasmPack = $null
    } else {
        Write-OK "wasm-pack $wpVer (minimum: $WASM_PACK_MIN)"
    }
}
if (-not $wasmPack -or $Force) {
    Write-Info "Installing wasm-pack via cargo..."
    cargo install wasm-pack 2>&1 | ForEach-Object { if ($_ -match 'error') { Write-Host $_ -ForegroundColor Red } }
    Refresh-Path
    $wasmPack = Get-Command wasm-pack -ErrorAction SilentlyContinue
    if ($wasmPack) {
        $wpVer = (wasm-pack --version 2>&1) -replace '.*?(\d+\.\d+\.\d+).*','$1'
        Write-OK "wasm-pack $wpVer installed"
        $installCount++
    } else {
        Write-Fail "wasm-pack installation failed"
        $errorCount++
    }
}

Write-Step "5/5" "Checking Node.js..."
$node = Get-Command node -ErrorAction SilentlyContinue
if ($node) {
    $nodeVer = (node --version 2>&1) -replace 'v',''
    Write-OK "Node.js $nodeVer"
} else {
    if (-not $SkipTest) {
        Write-Info "Node.js not found — installing via winget..."
        $winget = Get-Command winget -ErrorAction SilentlyContinue
        if ($winget) {
            try {
                winget install OpenJS.NodeJS.LTS --accept-package-agreements --accept-source-agreements 2>&1 | Out-Null
                Refresh-Path
                $node = Get-Command node -ErrorAction SilentlyContinue
                if ($node) {
                    Write-OK "Node.js installed: $(node --version)"
                    $installCount++
                } else {
                    $nodePaths = @(
                        "$env:ProgramFiles\nodejs",
                        "${env:ProgramFiles(x86)}\nodejs",
                        "$env:LOCALAPPDATA\Programs\nodejs"
                    )
                    foreach ($np in $nodePaths) {
                        if (Test-Path (Join-Path $np "node.exe")) {
                            $env:Path = "$np;$env:Path"
                            Write-OK "Node.js found at $np (added to session PATH)"
                            $node = Get-Command node -ErrorAction SilentlyContinue
                            $installCount++
                            break
                        }
                    }
                    if (-not $node) {
                        Write-Warn "Node.js installed but not in PATH — restart terminal, or use -SkipTest"
                    }
                }
            } catch {
                Write-Warn "Node.js install failed — smoke test will be skipped"
            }
        } else {
            Write-Warn "Node.js not found and winget unavailable — smoke test will be skipped"
            Write-Info "Install manually: https://nodejs.org/en/download/"
        }
    } else {
        Write-Info "Node.js not found (smoke test skipped via -SkipTest)"
    }
}

Write-Host ""

if ($errorCount -gt 0) {
    Write-Fail "Cannot proceed — $errorCount dependency check(s) failed"
    Write-Info "Fix the issues above and re-run this script"
    exit 1
}

if ($installCount -gt 0) {
    Write-Host ""
    Write-Host "  Installed $installCount dependency(ies)" -ForegroundColor Cyan
    Write-Host ""
}

Write-Host "==========================================" -ForegroundColor Cyan
Write-Host " Building WASM (ternary-math)" -ForegroundColor Cyan
Write-Host "==========================================" -ForegroundColor Cyan
Write-Host ""

Push-Location $crateDir
try {
    Write-Info "wasm-pack build --target nodejs --no-default-features --release"
    Write-Host ""
    wasm-pack build --target nodejs --no-default-features --release
    if ($LASTEXITCODE -ne 0) {
        Write-Fail "wasm-pack build failed (exit code $LASTEXITCODE)"
        exit 1
    }

    Write-Host ""
    Write-OK "WASM build complete"

    $pkgDir = Join-Path $crateDir "pkg"
    $jsFile = Join-Path $pkgDir "ternary_math.js"
    $wasmFile = Join-Path $pkgDir "ternary_math_bg.wasm"
    $dtsFile = Join-Path $pkgDir "ternary_math.d.ts"

    if (Test-Path $jsFile)   { Write-OK "  $jsFile ($([math]::Round((Get-Item $jsFile).Length / 1KB, 1)) KB)" }
    if (Test-Path $wasmFile) { Write-OK "  $wasmFile ($([math]::Round((Get-Item $wasmFile).Length / 1KB, 1)) KB)" }
    if (Test-Path $dtsFile)  { Write-OK "  $dtsFile ($([math]::Round((Get-Item $dtsFile).Length / 1KB, 1)) KB)" }

    $node = Get-Command node -ErrorAction SilentlyContinue
    if ($node -and -not $SkipTest) {
        Write-Host ""
        Write-Host "==========================================" -ForegroundColor Cyan
        Write-Host " Smoke Test" -ForegroundColor Cyan
        Write-Host "==========================================" -ForegroundColor Cyan
        Write-Host ""

        $testScript = @"
const tm = require('./pkg/ternary_math.js');
let pass = 0;
let fail = 0;

function check(name, actual, expected) {
    if (actual === expected) {
        console.log('  PASS  ' + name + ' = ' + actual);
        pass++;
    } else {
        console.log('  FAIL  ' + name + ' = ' + actual + ' (expected ' + expected + ')');
        fail++;
    }
}

console.log('--- Integer Constants ---');
check('QUAD_PRODUCT (R6)',   tm.quad_product(),      364);
check('ROOT_X1 (pi)',        tm.root_x1(),           14);
check('FIBONACCI_PI',        tm.fibonacci_pi(),      377);
check('FIBONACCI_12',        tm.fibonacci_12(),      144);
check('CIRCUMFERENCE',       tm.circumference(),     540);
check('CYCLIC_ORDER',        tm.cyclic_order(),      13);
check('CENTER',              tm.center(),            7);

console.log('--- Vacuum Bias ---');
check('VACUUM_BIAS_NUM',     tm.vacuum_bias_num(),   193);
check('VACUUM_BIAS_DEN',     tm.vacuum_bias_den(),   100000);

console.log('--- Sponge Hash ---');
const h = tm.sponge_hash(Buffer.from('PlenumNET'), 32);
check('sponge_hash length',  h.length,               32);
check('sponge_hash type',    h.constructor.name,     'Uint8Array');

console.log('--- TIS-27 Hash ---');
const t = tm.sponge_hash_tis(Buffer.from('test'), 16);
check('sponge_hash_tis len', t.length,               16);

console.log('');
console.log('Results: ' + pass + ' passed, ' + fail + ' failed');
if (fail > 0) { process.exit(1); }
"@
        $testFile = Join-Path $env:TEMP "plenumnet-wasm-test.js"
        Set-Content -Path $testFile -Value $testScript -Encoding UTF8

        node $testFile
        if ($LASTEXITCODE -ne 0) {
            Write-Fail "Smoke test failed"
            exit 1
        }
        Write-OK "All smoke tests passed"
        Remove-Item $testFile -ErrorAction SilentlyContinue
    } elseif ($SkipTest) {
        Write-Info "Smoke test skipped (-SkipTest)"
    } else {
        Write-Warn "Smoke test skipped (Node.js not available)"
    }
} finally {
    Pop-Location
}

Write-Host ""
Write-Host "==========================================" -ForegroundColor Green
Write-Host " BUILD COMPLETE" -ForegroundColor Green
Write-Host "==========================================" -ForegroundColor Green
Write-Host ""
