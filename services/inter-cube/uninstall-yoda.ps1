<#
.SYNOPSIS
    PlenumNET Array3 Uninstaller
    Removes all PlenumNET Array3 services, scheduled tasks, launchers,
    and optionally identity data and source code.

.DESCRIPTION
    Run as Administrator:
      powershell -ExecutionPolicy Bypass -File uninstall-yoda.ps1

    This will:
      1. Stop and remove the 3 Windows Services (PlenumNET-Array3-1/2/3)
      2. Remove the watchdog scheduled task (PlenumNET-Array3-Watchdog)
      3. Remove desktop launchers (Start/Stop PlenumNET Array3.cmd)
      4. Optionally remove identity data (PERMANENT — keys cannot be recovered)
      5. Optionally remove source code and binaries (C:\PlenumNET)
      6. Optionally remove LLM config (C:\ProgramData\PlenumNET)

.NOTES
    Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
    Applied Physics Division
#>

function Test-Admin {
    $currentPrincipal = New-Object Security.Principal.WindowsPrincipal([Security.Principal.WindowsIdentity]::GetCurrent())
    return $currentPrincipal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
}

if (-not (Test-Admin)) {
    Write-Host ""
    Write-Host "  [FAIL] Administrator privileges are required to uninstall." -ForegroundColor Red
    Write-Host "         Right-click PowerShell and select 'Run as administrator'." -ForegroundColor Yellow
    Write-Host ""
    Read-Host "Press Enter to close"
    exit 1
}

Write-Host ""
Write-Host "==========================================================" -ForegroundColor Cyan
Write-Host "  PlenumNET Array3 Uninstaller" -ForegroundColor Cyan
Write-Host "  Capomastro Holdings Ltd." -ForegroundColor Cyan
Write-Host "==========================================================" -ForegroundColor Cyan
Write-Host ""
Write-Host "  This will remove PlenumNET Array3 components from this machine." -ForegroundColor White
Write-Host ""

$confirm = Read-Host "  Continue with uninstall? (yes/no)"
if ($confirm -notin @("yes", "YES", "y", "Y")) {
    Write-Host "  Uninstall cancelled." -ForegroundColor Yellow
    exit 0
}

Write-Host ""

$removedCount = 0

Write-Host "  Step 1: Stopping and removing Windows Services..." -ForegroundColor Yellow
Write-Host "  ---" -ForegroundColor DarkGray
for ($i = 3; $i -ge 1; $i--) {
    $svcName = "PlenumNET-Array3-$i"
    $svc = Get-Service -Name $svcName -ErrorAction SilentlyContinue
    if ($svc) {
        if ($svc.Status -eq 'Running') {
            Stop-Service -Name $svcName -Force -ErrorAction SilentlyContinue
            Start-Sleep -Seconds 2
            Write-Host "  [OK] Stopped $svcName" -ForegroundColor Green
        }
        & sc.exe delete $svcName | Out-Null
        Write-Host "  [OK] Removed service $svcName" -ForegroundColor Green
        $removedCount++
    } else {
        Write-Host "  [INFO] Service $svcName not found (already removed)" -ForegroundColor DarkGray
    }
}

Write-Host ""
Write-Host "  Step 2: Removing watchdog scheduled task..." -ForegroundColor Yellow
Write-Host "  ---" -ForegroundColor DarkGray
$taskName = "PlenumNET-Array3-Watchdog"
try {
    $task = Get-ScheduledTask -TaskName $taskName -ErrorAction SilentlyContinue
    if ($task) {
        Unregister-ScheduledTask -TaskName $taskName -Confirm:$false -ErrorAction Stop
        Write-Host "  [OK] Removed scheduled task $taskName" -ForegroundColor Green
        $removedCount++
    } else {
        Write-Host "  [INFO] Scheduled task $taskName not found (already removed)" -ForegroundColor DarkGray
    }
} catch {
    try {
        schtasks.exe /Delete /TN $taskName /F 2>&1 | Out-Null
        Write-Host "  [OK] Removed scheduled task $taskName (via schtasks.exe)" -ForegroundColor Green
        $removedCount++
    } catch {
        Write-Host "  [WARN] Could not remove scheduled task: $_" -ForegroundColor Yellow
    }
}

Write-Host ""
Write-Host "  Step 3: Removing desktop launchers..." -ForegroundColor Yellow
Write-Host "  ---" -ForegroundColor DarkGray
$desktop = [Environment]::GetFolderPath("Desktop")
$launchers = @(
    "Start PlenumNET Array3.cmd",
    "Stop PlenumNET Array3.cmd",
    "Start PlenumNET Array3.bat",
    "Stop PlenumNET Array3.bat"
)
foreach ($launcher in $launchers) {
    $path = Join-Path $desktop $launcher
    if (Test-Path $path) {
        Remove-Item $path -Force
        Write-Host "  [OK] Removed $path" -ForegroundColor Green
        $removedCount++
    }
}

Write-Host ""
Write-Host "  Step 4: Removing wrapper scripts..." -ForegroundColor Yellow
Write-Host "  ---" -ForegroundColor DarkGray
$wrapperDir = "C:\PlenumNET\services\wrappers"
if (Test-Path $wrapperDir) {
    Remove-Item $wrapperDir -Recurse -Force -ErrorAction SilentlyContinue
    Write-Host "  [OK] Removed $wrapperDir" -ForegroundColor Green
    $removedCount++
} else {
    Write-Host "  [INFO] Wrapper directory not found" -ForegroundColor DarkGray
}

Write-Host ""
Write-Host "  Step 5: LLM configuration (C:\ProgramData\PlenumNET)..." -ForegroundColor Yellow
Write-Host "  ---" -ForegroundColor DarkGray
$plenumDataDir = "C:\ProgramData\PlenumNET"
if (Test-Path $plenumDataDir) {
    $removeLlm = Read-Host "  Remove LLM engine config at $plenumDataDir? (yes/no)"
    if ($removeLlm -in @("yes", "YES", "y", "Y")) {
        Remove-Item $plenumDataDir -Recurse -Force -ErrorAction SilentlyContinue
        Write-Host "  [OK] Removed $plenumDataDir" -ForegroundColor Green
        $removedCount++
    } else {
        Write-Host "  [INFO] Preserved $plenumDataDir" -ForegroundColor DarkGray
    }
}

Write-Host ""
Write-Host "  Step 6: Node identity data..." -ForegroundColor Yellow
Write-Host "  ---" -ForegroundColor DarkGray
$identityBase = "C:\PlenumNET\plenumnet-data"
$identityPreserved = $false
if (Test-Path $identityBase) {
    Write-Host ""
    Write-Host "  WARNING: Removing identity data is PERMANENT." -ForegroundColor Red
    Write-Host "  Your node private keys (master.key) will be destroyed." -ForegroundColor Red
    Write-Host "  You will need to generate new identities and re-register" -ForegroundColor Red
    Write-Host "  with the coordinator if you reinstall." -ForegroundColor Red
    Write-Host ""
    $removeIdentity = Read-Host "  Remove identity data at $identityBase? (yes/no)"
    if ($removeIdentity -in @("yes", "YES", "y", "Y")) {
        $confirmAgain = Read-Host "  Type 'DELETE' to confirm permanent key deletion"
        if ($confirmAgain -eq "DELETE") {
            $keyFiles = Get-ChildItem -Path $identityBase -Filter "master.key" -Recurse -ErrorAction SilentlyContinue
            foreach ($kf in $keyFiles) {
                try {
                    $rng = [System.Security.Cryptography.RandomNumberGenerator]::Create()
                    $fileLen = (Get-Item $kf.FullName).Length
                    $randomBytes = New-Object byte[] $fileLen
                    $rng.GetBytes($randomBytes)
                    [System.IO.File]::WriteAllBytes($kf.FullName, $randomBytes)
                    $rng.Dispose()
                } catch {}
            }
            Remove-Item $identityBase -Recurse -Force -ErrorAction SilentlyContinue
            Write-Host "  [OK] Removed identity data (keys securely overwritten)" -ForegroundColor Green
            $removedCount++
        } else {
            Write-Host "  [INFO] Identity data preserved (confirmation not matched)" -ForegroundColor DarkGray
            $identityPreserved = $true
        }
    } else {
        Write-Host "  [INFO] Identity data preserved" -ForegroundColor DarkGray
        $identityPreserved = $true
    }
}

Write-Host ""
Write-Host "  Step 7: Source code and binaries (C:\PlenumNET)..." -ForegroundColor Yellow
Write-Host "  ---" -ForegroundColor DarkGray
$repoDir = "C:\PlenumNET"
if (Test-Path $repoDir) {
    if ($identityPreserved) {
        Write-Host "  [INFO] Identity data was preserved in Step 6." -ForegroundColor DarkGray
        Write-Host "         Removing $repoDir will delete source code and binaries" -ForegroundColor DarkGray
        Write-Host "         but identity data at $identityBase will be moved to a" -ForegroundColor DarkGray
        Write-Host "         safe location first." -ForegroundColor DarkGray
    }
    $dirSize = [math]::Round((Get-ChildItem $repoDir -Recurse -ErrorAction SilentlyContinue | Measure-Object -Property Length -Sum).Sum / 1MB, 0)
    $removeRepo = Read-Host "  Remove $repoDir (~${dirSize} MB)? (yes/no)"
    if ($removeRepo -in @("yes", "YES", "y", "Y")) {
        if ($identityPreserved -and (Test-Path $identityBase)) {
            $backupDir = Join-Path $env:ProgramData "PlenumNET-identity-backup"
            Write-Host "  [INFO] Moving identity data to $backupDir before deletion..." -ForegroundColor DarkGray
            if (-not (Test-Path $backupDir)) { New-Item -ItemType Directory -Path $backupDir -Force | Out-Null }
            Copy-Item -Path $identityBase -Destination $backupDir -Recurse -Force
            Write-Host "  [OK] Identity data backed up to $backupDir" -ForegroundColor Green
        }
        Remove-Item $repoDir -Recurse -Force -ErrorAction SilentlyContinue
        Write-Host "  [OK] Removed $repoDir" -ForegroundColor Green
        $removedCount++
    } else {
        Write-Host "  [INFO] Preserved $repoDir" -ForegroundColor DarkGray
    }
}

$oldIdentityBase = Join-Path $env:USERPROFILE ".plenumnet"
if (Test-Path $oldIdentityBase) {
    Write-Host ""
    Write-Host "  Step 8: Legacy identity data ($oldIdentityBase)..." -ForegroundColor Yellow
    Write-Host "  ---" -ForegroundColor DarkGray
    $removeLegacy = Read-Host "  Remove legacy identity data? (yes/no)"
    if ($removeLegacy -in @("yes", "YES", "y", "Y")) {
        $legacyKeys = Get-ChildItem -Path $oldIdentityBase -Filter "master.key" -Recurse -ErrorAction SilentlyContinue
        foreach ($lk in $legacyKeys) {
            try {
                $rng = [System.Security.Cryptography.RandomNumberGenerator]::Create()
                $fileLen = (Get-Item $lk.FullName).Length
                $randomBytes = New-Object byte[] $fileLen
                $rng.GetBytes($randomBytes)
                [System.IO.File]::WriteAllBytes($lk.FullName, $randomBytes)
                $rng.Dispose()
            } catch {}
        }
        Remove-Item $oldIdentityBase -Recurse -Force -ErrorAction SilentlyContinue
        Write-Host "  [OK] Removed $oldIdentityBase (keys securely overwritten)" -ForegroundColor Green
        $removedCount++
    } else {
        Write-Host "  [INFO] Preserved $oldIdentityBase" -ForegroundColor DarkGray
    }
}

Write-Host ""
Write-Host "==========================================================" -ForegroundColor Green
Write-Host "  PlenumNET Array3 Uninstall Complete" -ForegroundColor Green
Write-Host "  Capomastro Holdings Ltd." -ForegroundColor Green
Write-Host "==========================================================" -ForegroundColor Green
Write-Host ""
Write-Host "  $removedCount component(s) removed." -ForegroundColor White
Write-Host ""
Write-Host "  Note: Rust and LLVM toolchains were not removed (they may be used" -ForegroundColor DarkGray
Write-Host "  by other applications). To remove them manually:" -ForegroundColor DarkGray
Write-Host "    Rust:  rustup self uninstall" -ForegroundColor DarkGray
Write-Host "    LLVM:  winget uninstall LLVM.LLVM" -ForegroundColor DarkGray
Write-Host ""
Read-Host "Press Enter to close"
