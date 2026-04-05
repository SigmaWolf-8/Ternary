# test-monitor.ps1 — Pull latest & open Array3 Monitor for testing
# Run from any directory on the Windows machine.
# Usage:  powershell -ExecutionPolicy Bypass -File C:\PlenumNET\scripts\test-monitor.ps1

$RepoRoot = "C:\PlenumNET"
$MonitorFile = Join-Path $RepoRoot "services\inter-cube\monitor\array3-monitor-v9.html"

# --- Pull latest from GitHub ---
Write-Host "[1/3] Pulling latest from GitHub..." -ForegroundColor Cyan
Push-Location $RepoRoot
try {
    git pull --ff-only origin main
    if ($LASTEXITCODE -ne 0) {
        Write-Host "ERROR: git pull failed. Resolve conflicts or check connectivity." -ForegroundColor Red
        Pop-Location
        exit 1
    }
} finally {
    Pop-Location
}

# --- Verify monitor file exists ---
Write-Host "[2/3] Verifying monitor file..." -ForegroundColor Cyan
if (-Not (Test-Path $MonitorFile)) {
    Write-Host "ERROR: Monitor file not found at $MonitorFile" -ForegroundColor Red
    exit 1
}

$version = Select-String -Path $MonitorFile -Pattern "Monitor v[\d.]+" | Select-Object -First 1
Write-Host "       Found: $($version.Matches.Value)" -ForegroundColor Green

# --- Open in default browser ---
Write-Host "[3/3] Opening monitor in browser..." -ForegroundColor Cyan
Start-Process $MonitorFile

Write-Host ""
Write-Host "Monitor opened as file:// — paste your relay token in the setup banner." -ForegroundColor Yellow
Write-Host "It will connect to the published relay and show live daemon data." -ForegroundColor Yellow
