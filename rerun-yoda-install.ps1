<#
.SYNOPSIS
    PlenumNET Array3 — Clean Reinstall Script
    Stops old services, kills old daemons, deletes stale services,
    pulls latest code from GitHub, and re-runs the deployer.

.DESCRIPTION
    Run from any PowerShell (elevated or not — it self-elevates):
      irm https://plenumnet.replit.app/api/yoda-installer | iex
#>

$ErrorActionPreference = "Continue"

function Test-Admin {
    $currentPrincipal = New-Object Security.Principal.WindowsPrincipal([Security.Principal.WindowsIdentity]::GetCurrent())
    return $currentPrincipal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
}

if (-not (Test-Admin)) {
    Write-Host ""
    Write-Host "  Elevating to Administrator..." -ForegroundColor Yellow
    $tempScript = Join-Path $env:TEMP "plenumnet-reinstall.ps1"
    $scriptContent = (New-Object Net.WebClient).DownloadString("https://plenumnet.replit.app/api/yoda-installer")
    Set-Content -Path $tempScript -Value $scriptContent -Encoding UTF8
    Start-Process powershell.exe -Verb RunAs -ArgumentList "-NoProfile -ExecutionPolicy Bypass -File `"$tempScript`""
    Write-Host "  Elevated window launched. This window can be closed." -ForegroundColor Green
    exit 0
}

Write-Host ""
Write-Host "==========================================================" -ForegroundColor Cyan
Write-Host "  PlenumNET Array3 — Clean Reinstall" -ForegroundColor Cyan
Write-Host "  Running as Administrator" -ForegroundColor Green
Write-Host "==========================================================" -ForegroundColor Cyan
Write-Host ""

Write-Host "STEP 1: Stopping services..." -ForegroundColor Yellow
foreach ($svc in @("PlenumNET-Array3-1", "PlenumNET-Array3-2", "PlenumNET-Array3-3")) {
    $s = Get-Service -Name $svc -ErrorAction SilentlyContinue
    if ($s) {
        Stop-Service -Name $svc -Force -ErrorAction SilentlyContinue
        Write-Host "  [OK] Stopped $svc" -ForegroundColor Green
    } else {
        Write-Host "  [--] $svc not found (already removed)" -ForegroundColor DarkGray
    }
}
Start-Sleep -Seconds 2

Write-Host ""
Write-Host "STEP 2: Killing any remaining daemon processes..." -ForegroundColor Yellow
$procs = Get-Process -Name "inter-cube-daemon" -ErrorAction SilentlyContinue
if ($procs) {
    $procs | Stop-Process -Force -ErrorAction SilentlyContinue
    Start-Sleep -Seconds 2
    $remaining = Get-Process -Name "inter-cube-daemon" -ErrorAction SilentlyContinue
    if ($remaining) {
        Write-Host "  [!!] Force-killing with taskkill..." -ForegroundColor Yellow
        & taskkill /F /IM "inter-cube-daemon.exe" 2>$null
        Start-Sleep -Seconds 2
    }
    Write-Host "  [OK] All daemon processes terminated" -ForegroundColor Green
} else {
    Write-Host "  [--] No daemon processes running" -ForegroundColor DarkGray
}

Write-Host ""
Write-Host "STEP 3: Deleting old services..." -ForegroundColor Yellow
foreach ($svc in @("PlenumNET-Array3-1", "PlenumNET-Array3-2", "PlenumNET-Array3-3")) {
    $result = & sc.exe delete $svc 2>&1
    if ($LASTEXITCODE -eq 0) {
        Write-Host "  [OK] Deleted $svc" -ForegroundColor Green
    } else {
        Write-Host "  [--] $svc already gone" -ForegroundColor DarkGray
    }
}

Write-Host ""
Write-Host "STEP 4: Removing watchdog scheduled task..." -ForegroundColor Yellow
$task = Get-ScheduledTask -TaskName "PlenumNET-Array3-Watchdog" -ErrorAction SilentlyContinue
if ($task) {
    Unregister-ScheduledTask -TaskName "PlenumNET-Array3-Watchdog" -Confirm:$false
    Write-Host "  [OK] Watchdog removed" -ForegroundColor Green
} else {
    Write-Host "  [--] No watchdog task found" -ForegroundColor DarkGray
}

Write-Host ""
Write-Host "STEP 5: Cleaning old wrapper scripts..." -ForegroundColor Yellow
$repoDir = "C:\PlenumNET"
$wrapperDir = Join-Path $repoDir "services\wrappers"
if (Test-Path $wrapperDir) {
    Remove-Item -Recurse -Force $wrapperDir -ErrorAction SilentlyContinue
    Write-Host "  [OK] Removed old wrappers" -ForegroundColor Green
} else {
    Write-Host "  [--] No old wrappers" -ForegroundColor DarkGray
}

Write-Host ""
Write-Host "STEP 6: Updating source code..." -ForegroundColor Yellow
if (Test-Path $repoDir) {
    Push-Location $repoDir
    & git fetch origin main 2>$null
    & git reset --hard origin/main 2>$null
    Write-Host "  [OK] Source updated to latest" -ForegroundColor Green
    Pop-Location
} else {
    Write-Host "  [!!] C:\PlenumNET not found — cloning..." -ForegroundColor Yellow
    & git clone "https://github.com/SigmaWolf-8/Ternary.git" $repoDir 2>$null
    Write-Host "  [OK] Cloned to $repoDir" -ForegroundColor Green
}

Write-Host ""
Write-Host "STEP 7: Launching deployer..." -ForegroundColor Yellow
Write-Host ""
$deployScript = Join-Path $repoDir "services\inter-cube\deploy-yoda.ps1"
if (Test-Path $deployScript) {
    & $deployScript
} else {
    Write-Host "  [!!] deploy-yoda.ps1 not found at $deployScript" -ForegroundColor Red
    Write-Host "  Downloading from API instead..." -ForegroundColor Yellow
    $tempDeploy = Join-Path $env:TEMP "deploy-yoda.ps1"
    (New-Object Net.WebClient).DownloadFile("https://plenumnet.replit.app/api/deploy-yoda", $tempDeploy)
    & $tempDeploy
}
