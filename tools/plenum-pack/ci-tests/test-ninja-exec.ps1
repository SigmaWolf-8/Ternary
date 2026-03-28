#Requires -RunAsAdministrator
param(
    [Parameter(Mandatory=$true)]
    [string]$MsiPath
)

$ErrorActionPreference = "Stop"
$ProductName = "NinjaExec"
$RegistryPath = "HKLM:\Software\Capomastro\PlenumNET\Apps\$ProductName"
$InstallDir = "$env:ProgramFiles\Capomastro\NinjaExec"
$DataDir = "$env:APPDATA\NinjaExec"
$Binary = "ninja-exec.exe"

function Write-TestResult {
    param([string]$Step, [bool]$Passed, [string]$Detail)
    $status = if ($Passed) { "[OK]" } else { "[FAIL]" }
    Write-Host "$status Step: $Step - $Detail"
    if (-not $Passed) { exit 1 }
}

# Step 1: Clean install (silent)
Write-Host "=== Step 1: Clean Install (Silent) ==="
$msiFile = Get-ChildItem -Path $MsiPath -Filter "*NinjaExec*.msi" | Select-Object -First 1
if (-not $msiFile) {
    Write-TestResult "Clean Install" $false "MSI file not found in $MsiPath"
}

$passphraseFile = "$env:TEMP\ninja-test-passphrase.txt"
"TestPassphrase123!" | Out-File -FilePath $passphraseFile -Encoding utf8 -NoNewline
icacls $passphraseFile /inheritance:r /grant "${env:USERNAME}:(R)" | Out-Null

$proc = Start-Process msiexec -ArgumentList "/i `"$($msiFile.FullName)`" /qn PASSPHRASE_FILE=`"$passphraseFile`"" -Wait -PassThru
Write-TestResult "Clean Install" ($proc.ExitCode -eq 0) "Exit code: $($proc.ExitCode)"

# Step 3: System integration checks
Write-Host "=== Step 3: System Integration Checks ==="

$regExists = Test-Path $RegistryPath
Write-TestResult "Registry Entry" $regExists "Registry path: $RegistryPath"

$binaryExists = Test-Path "$InstallDir\$Binary"
Write-TestResult "Binary Installed" $binaryExists "Binary path: $InstallDir\$Binary"

$startMenuPath = "$env:ProgramData\Microsoft\Windows\Start Menu\Programs\PlenumNET\NinjaExec"
$shortcutExists = Test-Path $startMenuPath
Write-TestResult "Start Menu Shortcuts" $shortcutExists "Start Menu path: $startMenuPath"

$pathCheck = $env:PATH -like "*$InstallDir*"
Write-TestResult "PATH Entry" $pathCheck "Install dir in PATH"

# Step 4: Key material verification
Write-Host "=== Step 4: Key Material Verification ==="
$keystoreExists = Test-Path "$DataDir\keystore"
Write-TestResult "Keystore Created" $keystoreExists "Keystore path: $DataDir\keystore"

if ($keystoreExists) {
    $keystoreSize = (Get-Item "$DataDir\keystore").Length
    Write-TestResult "Keystore Non-Empty" ($keystoreSize -gt 0) "Keystore size: $keystoreSize bytes"
}

# Step 5: Tray process verification
Write-Host "=== Step 5: Process Verification ==="
$process = Get-Process -Name "ninja-exec" -ErrorAction SilentlyContinue
Write-TestResult "Tray Process Running" ($null -ne $process) "Process check for ninja-exec"

# Step 7: Uninstall (standard)
Write-Host "=== Step 7: Standard Uninstall ==="
$proc = Start-Process msiexec -ArgumentList "/x `"$($msiFile.FullName)`" /qn" -Wait -PassThru
Write-TestResult "Uninstall" ($proc.ExitCode -eq 0) "Exit code: $($proc.ExitCode)"

$binaryGone = -not (Test-Path "$InstallDir\$Binary")
Write-TestResult "Binary Removed" $binaryGone "Binary should be removed"

$regGone = -not (Test-Path $RegistryPath)
Write-TestResult "Registry Removed" $regGone "Registry should be removed"

$dataPreserved = Test-Path $DataDir
Write-TestResult "Data Preserved" $dataPreserved "Data directory should be preserved"

# Cleanup
Remove-Item -Path $passphraseFile -Force -ErrorAction SilentlyContinue

Write-Host ""
Write-Host "=== All tests passed for $ProductName ==="
