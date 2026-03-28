#Requires -RunAsAdministrator
param(
    [Parameter(Mandatory=$true)]
    [string]$MsiPath
)

$ErrorActionPreference = "Stop"
$ProductName = "Array3Watchdog"
$ServiceName = "Array3Watchdog"
$RegistryPath = "HKLM:\Software\Capomastro\PlenumNET\Apps\$ProductName"
$InstallDir = "$env:ProgramFiles\Capomastro\Array3Watchdog"
$DataDir = "$env:APPDATA\Array3Watchdog"
$Binary = "array3-watchdog.exe"

function Write-TestResult {
    param([string]$Step, [bool]$Passed, [string]$Detail)
    $status = if ($Passed) { "[OK]" } else { "[FAIL]" }
    Write-Host "$status Step: $Step - $Detail"
    if (-not $Passed) { exit 1 }
}

# Step 1: Clean install (silent)
Write-Host "=== Step 1: Clean Install (Silent) ==="
$msiFile = Get-ChildItem -Path $MsiPath -Filter "*Array3Watchdog*.msi" | Select-Object -First 1
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

# Service registration check
$service = Get-Service -Name $ServiceName -ErrorAction SilentlyContinue
Write-TestResult "Service Registered" ($null -ne $service) "Service: $ServiceName"

if ($service) {
    Write-TestResult "Service Running" ($service.Status -eq "Running") "Status: $($service.Status)"
}

# Config and key material
Write-Host "=== Step 4: Configuration Verification ==="
$configExists = Test-Path "$DataDir\config.toml"
Write-TestResult "Config Created" $configExists "Config path: $DataDir\config.toml"

$keystoreExists = Test-Path "$DataDir\keystore"
Write-TestResult "Keystore Created" $keystoreExists "Keystore path: $DataDir\keystore"

# Step 7: Uninstall
Write-Host "=== Step 7: Standard Uninstall ==="
$proc = Start-Process msiexec -ArgumentList "/x `"$($msiFile.FullName)`" /qn" -Wait -PassThru
Write-TestResult "Uninstall" ($proc.ExitCode -eq 0) "Exit code: $($proc.ExitCode)"

$serviceGone = $null -eq (Get-Service -Name $ServiceName -ErrorAction SilentlyContinue)
Write-TestResult "Service Removed" $serviceGone "Service should be unregistered"

$dataPreserved = Test-Path $DataDir
Write-TestResult "Data Preserved" $dataPreserved "Data directory should be preserved"

Write-Host ""
Write-Host "=== All tests passed for $ProductName ==="
