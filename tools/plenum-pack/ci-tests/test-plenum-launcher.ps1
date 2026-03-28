#Requires -RunAsAdministrator
param(
    [Parameter(Mandatory=$true)]
    [string]$MsiPath
)

$ErrorActionPreference = "Stop"
$ProductName = "PlenumNET-Launcher"
$RegistryPath = "HKLM:\Software\Capomastro\PlenumNET\Apps\$ProductName"
$InstallDir = "$env:ProgramFiles\Capomastro\PlenumNET-Launcher"
$DataDir = "$env:APPDATA\PlenumNET-Launcher"
$Binary = "plenum-launcher.exe"
$ElevateHelper = "plenum-launcher-elevate.exe"

function Write-TestResult {
    param([string]$Step, [bool]$Passed, [string]$Detail)
    $status = if ($Passed) { "[OK]" } else { "[FAIL]" }
    Write-Host "$status Step: $Step - $Detail"
    if (-not $Passed) { exit 1 }
}

# Step 1: Clean install (silent)
Write-Host "=== Step 1: Clean Install (Silent) ==="
$msiFile = Get-ChildItem -Path $MsiPath -Filter "*Launcher*.msi" | Select-Object -First 1
if (-not $msiFile) {
    Write-TestResult "Clean Install" $false "MSI file not found in $MsiPath"
}

$proc = Start-Process msiexec -ArgumentList "/i `"$($msiFile.FullName)`" /qn" -Wait -PassThru
Write-TestResult "Clean Install" ($proc.ExitCode -eq 0) "Exit code: $($proc.ExitCode)"

# Step 3: System integration checks
Write-Host "=== Step 3: System Integration Checks ==="

$regExists = Test-Path $RegistryPath
Write-TestResult "Registry Entry" $regExists "Registry path: $RegistryPath"

$binaryExists = Test-Path "$InstallDir\$Binary"
Write-TestResult "Binary Installed" $binaryExists "Binary path: $InstallDir\$Binary"

$elevateExists = Test-Path "$InstallDir\$ElevateHelper"
Write-TestResult "Elevation Helper" $elevateExists "Helper path: $InstallDir\$ElevateHelper"

# Autostart check
$autostartKey = "HKCU:\Software\Microsoft\Windows\CurrentVersion\Run"
$autostartValue = Get-ItemProperty -Path $autostartKey -Name $ProductName -ErrorAction SilentlyContinue
Write-TestResult "Autostart Entry" ($null -ne $autostartValue) "Autostart registry check"

# Start Menu check
$startMenuPath = "$env:ProgramData\Microsoft\Windows\Start Menu\Programs\PlenumNET"
$startMenuExists = Test-Path $startMenuPath
Write-TestResult "Start Menu Folder" $startMenuExists "PlenumNET Start Menu folder"

# Elevation helper signature (if signed)
if ($elevateExists) {
    $sig = Get-AuthenticodeSignature "$InstallDir\$ElevateHelper" -ErrorAction SilentlyContinue
    if ($sig -and $sig.Status -eq "Valid") {
        Write-TestResult "Elevation Helper Signed" $true "Authenticode signature valid"
    } else {
        Write-Host "[WARN] Elevation helper not signed or signature invalid (expected in dev builds)"
    }
}

# Step 7: Uninstall
Write-Host "=== Step 7: Standard Uninstall ==="
$proc = Start-Process msiexec -ArgumentList "/x `"$($msiFile.FullName)`" /qn" -Wait -PassThru
Write-TestResult "Uninstall" ($proc.ExitCode -eq 0) "Exit code: $($proc.ExitCode)"

$binaryGone = -not (Test-Path "$InstallDir\$Binary")
Write-TestResult "Binary Removed" $binaryGone "Binary should be removed"

$regGone = -not (Test-Path $RegistryPath)
Write-TestResult "Registry Removed" $regGone "Registry should be removed"

Write-Host ""
Write-Host "=== All tests passed for $ProductName ==="
