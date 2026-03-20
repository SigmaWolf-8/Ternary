$ErrorActionPreference = "Stop"
Write-Host "=== PlenumNET Consolidation Script ===" -ForegroundColor Cyan
Write-Host ""

$SOURCE = "C:\Users\Sigma\PlenumNET"
$DEST   = "C:\PlenumNET"

if (-not (Test-Path $SOURCE)) {
    Write-Host "ERROR: Source not found at $SOURCE" -ForegroundColor Red
    exit 1
}
if (-not (Test-Path $DEST)) {
    Write-Host "ERROR: Destination not found at $DEST" -ForegroundColor Red
    exit 1
}

Write-Host "Source:      $SOURCE"
Write-Host "Destination: $DEST"
Write-Host ""

Write-Host "[1/4] Copying built binary..." -ForegroundColor Yellow
$srcBin = Join-Path (Join-Path $SOURCE "target") "release"
$dstBin = Join-Path (Join-Path $DEST "target") "release"
if (Test-Path (Join-Path $srcBin "inter-cube-daemon.exe")) {
    New-Item -ItemType Directory -Force -Path $dstBin | Out-Null
    Copy-Item -Path (Join-Path $srcBin "inter-cube-daemon.exe") -Destination $dstBin -Force
    Write-Host "  OK inter-cube-daemon.exe -> $dstBin" -ForegroundColor Green
} else {
    Write-Host "  SKIP: No binary found in $srcBin" -ForegroundColor Yellow
}

Write-Host ""
Write-Host "[2/4] Copying identity keys..." -ForegroundColor Yellow
$srcIdentity = Join-Path (Join-Path $env:USERPROFILE ".plenumnet") "identity"
if (Test-Path $srcIdentity) {
    $keyFiles = Get-ChildItem -Path $srcIdentity -File -ErrorAction SilentlyContinue
    if ($keyFiles.Count -gt 0) {
        Write-Host "  OK Identity directory found: $srcIdentity ($($keyFiles.Count) file(s))" -ForegroundColor Green
        foreach ($f in $keyFiles) {
            Write-Host "    - $($f.Name) ($($f.Length) bytes)"
        }
        Write-Host "  Identity stays in $srcIdentity (daemon reads from ~/.plenumnet/identity/)" -ForegroundColor Cyan
    }
} else {
    Write-Host "  SKIP: No identity directory at $srcIdentity" -ForegroundColor Yellow
}

Write-Host ""
Write-Host "[3/4] Copying build dependencies (target/release/*.dll, *.lib)..." -ForegroundColor Yellow
if (Test-Path $srcBin) {
    $deps = Get-ChildItem -Path $srcBin -File -ErrorAction SilentlyContinue |
        Where-Object { $_.Extension -in ".dll", ".lib", ".pdb" }
    if ($deps.Count -gt 0) {
        foreach ($d in $deps) {
            Copy-Item -Path $d.FullName -Destination $dstBin -Force
            Write-Host "  OK $($d.Name)" -ForegroundColor Green
        }
    } else {
        Write-Host "  No .dll/.lib/.pdb files to copy (static build)" -ForegroundColor Cyan
    }
}

Write-Host ""
Write-Host "[4/4] Copying cargo build cache (so you don't rebuild from scratch)..." -ForegroundColor Yellow
$srcTarget = Join-Path $SOURCE "target"
$dstTarget = Join-Path $DEST "target"
$srcFingerprint = Join-Path (Join-Path $srcTarget "release") ".fingerprint"
if (Test-Path $srcFingerprint) {
    $dstFingerprint = Join-Path (Join-Path $dstTarget "release") ".fingerprint"
    if (-not (Test-Path $dstFingerprint)) {
        Write-Host "  Copying .fingerprint cache (speeds up future builds)..."
        New-Item -ItemType Directory -Force -Path $dstFingerprint | Out-Null
        Copy-Item -Path (Join-Path $srcFingerprint "*") -Destination $dstFingerprint -Recurse -Force
        Write-Host "  OK Fingerprint cache copied" -ForegroundColor Green
    } else {
        Write-Host "  SKIP: .fingerprint already exists at destination" -ForegroundColor Cyan
    }
} else {
    Write-Host "  SKIP: No .fingerprint cache in source" -ForegroundColor Yellow
}

Write-Host ""
Write-Host "=== Verification ===" -ForegroundColor Cyan
$finalBin = Join-Path $dstBin "inter-cube-daemon.exe"
if (Test-Path $finalBin) {
    $size = [math]::Round((Get-Item $finalBin).Length / 1MB, 1)
    Write-Host "  OK Binary ready: $finalBin ($size MB)" -ForegroundColor Green
    Write-Host ""
    Write-Host "=== Connect to PlenumLAN ===" -ForegroundColor Cyan
    Write-Host "Run this to join the live CRS on Replit:" -ForegroundColor Yellow
    Write-Host ""
    Write-Host '  $env:CUBE_MODE="cube"' -ForegroundColor White
    Write-Host '  $env:CUBE_API_PORT="8080"' -ForegroundColor White
    Write-Host '  $env:CUBE_CRS_URL="https://f4db13cb-d3ea-46aa-84d6-6760abe923d5-00-xxd9pb9n5txn.picard.replit.dev"' -ForegroundColor White
    Write-Host "  & `"$finalBin`"" -ForegroundColor White
} else {
    Write-Host "  ERROR: Binary not found at $finalBin" -ForegroundColor Red
}

Write-Host ""
Read-Host "Press Enter to close"
