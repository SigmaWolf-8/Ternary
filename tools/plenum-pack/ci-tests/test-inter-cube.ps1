#Requires -RunAsAdministrator
param(
    [Parameter(Mandatory=$true)]
    [string]$MsiPath,
    [string]$CrsEndpoint = "http://127.0.0.1:18080"
)

$ErrorActionPreference = "Stop"
$ProductName = "InterCubeDaemon"
$ServiceName = "InterCubeDaemon"
$RegistryPath = "HKLM:\Software\Capomastro\PlenumNET\Apps\$ProductName"
$InstallDir = "$env:ProgramFiles\Capomastro\InterCubeDaemon"
$DataDir = "$env:APPDATA\InterCubeDaemon"
$Binary = "inter-cube-daemon.exe"

function Write-TestResult {
    param([string]$Step, [bool]$Passed, [string]$Detail)
    $status = if ($Passed) { "[OK]" } else { "[FAIL]" }
    Write-Host "$status Step: $Step - $Detail"
    if (-not $Passed) { exit 1 }
}

# Step 1: Clean install (silent)
Write-Host "=== Step 1: Clean Install (Silent) ==="
$msiFile = Get-ChildItem -Path $MsiPath -Filter "*InterCube*.msi" | Select-Object -First 1
if (-not $msiFile) {
    Write-TestResult "Clean Install" $false "MSI file not found in $MsiPath"
}

$proc = Start-Process msiexec -ArgumentList "/i `"$($msiFile.FullName)`" /qn CRS_ENDPOINT=`"$CrsEndpoint`"" -Wait -PassThru
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
    Write-TestResult "Service Auto Start" ($service.StartType -eq "Automatic") "StartType: $($service.StartType)"
}

# Key material check
Write-Host "=== Step 4: Key Material Verification ==="
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
