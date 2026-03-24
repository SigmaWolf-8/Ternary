#Requires -Version 5.1
<#
.SYNOPSIS
    PlenumNET Service Manager for Windows
.DESCRIPTION
    Manages PlenumNET Inter-Cube daemon Windows Services.
    Supports install, uninstall, start, stop, restart, status, and logs.
    Services run under the installing user's account for direct access
    to identity keys in %USERPROFILE%\.plenumnet.
.NOTES
    Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
    Patent(s) Pending - All Rights Reserved
#>

param(
    [Parameter(Position=0, Mandatory=$true)]
    [ValidateSet("status", "start", "stop", "restart", "install", "uninstall", "logs", "watchdog")]
    [string]$Command,

    [Parameter(Position=1)]
    [int]$IdentityId = 0
)

$InstallDir = "C:\PlenumNET"
$IdentityBase = Join-Path $env:USERPROFILE ".plenumnet"
$LogDir = Join-Path $IdentityBase "logs"
$BinaryPath = Join-Path $InstallDir "target\release\inter-cube-daemon.exe"
$BasePeerPort = 8079
$PortStep = 3
$CRS_URL = "https://plenumnet.replit.app"

function Get-ServiceName {
    param([int]$Id)
    return "PlenumNET-Cube-$Id"
}

function Get-ServiceDisplayName {
    param([int]$Id)
    return "PlenumNET Inter-Cube Daemon (Identity #$Id)"
}

function Test-Admin {
    $currentPrincipal = New-Object Security.Principal.WindowsPrincipal([Security.Principal.WindowsIdentity]::GetCurrent())
    return $currentPrincipal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
}

function Require-Admin {
    if (-not (Test-Admin)) {
        Write-Host "  ERROR: This command requires Administrator privileges." -ForegroundColor Red
        Write-Host "  Right-click PowerShell and select 'Run as Administrator'." -ForegroundColor Yellow
        exit 1
    }
}

function Get-ServiceAccountName {
    $identity = [Security.Principal.WindowsIdentity]::GetCurrent()
    return $identity.Name
}

function Grant-LogonAsService {
    param([string]$AccountName)
    $tempDir = Join-Path $env:TEMP "plenumnet-secedit"
    if (-not (Test-Path $tempDir)) { New-Item -ItemType Directory -Path $tempDir -Force | Out-Null }
    $exportFile = Join-Path $tempDir "secpol-export.inf"
    $importFile = Join-Path $tempDir "secpol-import.inf"
    $seceditDb = Join-Path $tempDir "secedit.sdb"

    try {
        & secedit /export /cfg $exportFile /areas USER_RIGHTS 2>&1 | Out-Null
        if (-not (Test-Path $exportFile)) { return $false }

        $content = Get-Content $exportFile -Raw
        $sidObj = (New-Object System.Security.Principal.NTAccount($AccountName)).Translate(
            [System.Security.Principal.SecurityIdentifier])
        $sid = $sidObj.Value

        if ($content -match '(?m)^SeServiceLogonRight\s*=\s*(.*)$') {
            $existing = $Matches[1]
            if ($existing -notmatch [regex]::Escape($sid)) {
                $content = $content -replace "(?m)^(SeServiceLogonRight\s*=\s*.*)$", "`$1,*$sid"
            }
        } else {
            $content = $content -replace '(?m)(\[Privilege Rights\])', "`$1`r`nSeServiceLogonRight = *$sid"
        }

        Set-Content -Path $importFile -Value $content -Encoding Unicode
        & secedit /configure /db $seceditDb /cfg $importFile /areas USER_RIGHTS 2>&1 | Out-Null
        return $true
    } catch {
        Write-Host "  WARNING: Could not grant 'Log on as a service' right: $_" -ForegroundColor Yellow
        return $false
    } finally {
        Remove-Item $tempDir -Recurse -Force -ErrorAction SilentlyContinue
    }
}

function Set-ServiceLogonAccount {
    param([string]$ServiceName, [string]$AccountName)
    Write-Host "  Configuring service to run as: $AccountName" -ForegroundColor DarkGray
    Grant-LogonAsService -AccountName $AccountName | Out-Null
    & sc.exe config $ServiceName obj= $AccountName | Out-Null
}

function Get-AllIdentityIds {
    $ids = @()
    if (Test-Path $IdentityBase) {
        Get-ChildItem -Path $IdentityBase -Directory -Filter "identity-*" | ForEach-Object {
            $num = $_.Name -replace 'identity-', ''
            if ($num -match '^\d+$') { $ids += [int]$num }
        }
    }
    return ($ids | Sort-Object)
}

function Invoke-Status {
    param([int]$Id)
    Write-Host ""
    Write-Host "  PlenumNET Daemon Service Status" -ForegroundColor Cyan
    Write-Host ""

    if ($Id -gt 0) {
        $svcName = Get-ServiceName -Id $Id
        $svc = Get-Service -Name $svcName -ErrorAction SilentlyContinue
        if ($svc) {
            Write-Host "  Identity #$Id : $($svc.Status)" -ForegroundColor $(if ($svc.Status -eq 'Running') { 'Green' } else { 'Yellow' })
        } else {
            Write-Host "  Identity #$Id : not registered as service" -ForegroundColor DarkGray
        }
    } else {
        $allIds = Get-AllIdentityIds
        foreach ($i in $allIds) {
            $svcName = Get-ServiceName -Id $i
            $svc = Get-Service -Name $svcName -ErrorAction SilentlyContinue
            if ($svc) {
                Write-Host "  Identity #$i : $($svc.Status)" -ForegroundColor $(if ($svc.Status -eq 'Running') { 'Green' } else { 'Yellow' })
            } else {
                Write-Host "  Identity #$i : not registered as service" -ForegroundColor DarkGray
            }
        }
        if ($allIds.Count -eq 0) {
            Write-Host "  No daemon identities found." -ForegroundColor DarkGray
        }
    }
    Write-Host ""
}

function Invoke-Start {
    param([int]$Id)
    Require-Admin
    $svcName = Get-ServiceName -Id $Id
    $svc = Get-Service -Name $svcName -ErrorAction SilentlyContinue
    if (-not $svc) {
        Write-Host "  Service '$svcName' not found. Run 'install $Id' first." -ForegroundColor Red
        return
    }
    Start-Service -Name $svcName
    Write-Host "  Daemon #$Id started." -ForegroundColor Green
}

function Invoke-Stop {
    param([int]$Id)
    Require-Admin
    $svcName = Get-ServiceName -Id $Id
    $svc = Get-Service -Name $svcName -ErrorAction SilentlyContinue
    if (-not $svc) {
        Write-Host "  Service '$svcName' not found." -ForegroundColor Yellow
        return
    }
    Stop-Service -Name $svcName -Force
    Write-Host "  Daemon #$Id stopped." -ForegroundColor Green
}

function Invoke-Restart {
    param([int]$Id)
    Require-Admin
    $svcName = Get-ServiceName -Id $Id
    Restart-Service -Name $svcName -Force
    Write-Host "  Daemon #$Id restarted." -ForegroundColor Green
}

function Invoke-Install {
    param([int]$Id)
    Require-Admin

    if (-not (Test-Path $BinaryPath)) {
        Write-Host "  ERROR: Daemon binary not found at $BinaryPath" -ForegroundColor Red
        Write-Host "  Build first: cd $InstallDir; cargo build --release -p inter-cube" -ForegroundColor Yellow
        return
    }

    $agentDir = Join-Path $IdentityBase "identity-$Id"
    if (-not (Test-Path $agentDir)) {
        Write-Host "  ERROR: Identity directory not found: $agentDir" -ForegroundColor Red
        Write-Host "  Run the installer first to generate identities." -ForegroundColor Yellow
        return
    }

    $peerPort = $BasePeerPort + (($Id - 1) * $PortStep)
    $enginePort = $peerPort + 1
    $daemonPort = $peerPort + 2
    $svcName = Get-ServiceName -Id $Id
    $displayName = Get-ServiceDisplayName -Id $Id

    if (-not (Test-Path $LogDir)) {
        New-Item -ItemType Directory -Path $LogDir -Force | Out-Null
    }
    $logFile = Join-Path $LogDir "cube-${Id}.log"

    $wrapperDir = Join-Path $InstallDir "services\wrappers"
    if (-not (Test-Path $wrapperDir)) {
        New-Item -ItemType Directory -Path $wrapperDir -Force | Out-Null
    }
    $wrapperBat = Join-Path $wrapperDir "cube-${Id}-start.bat"
    @"
@echo off
set CUBE_MODE=cube
set CUBE_API_PORT=$daemonPort
set LLM_PORT=$enginePort
set CUBE_PEER_PORT=$peerPort
set CUBE_CRS_URL=$CRS_URL
set RELAY_URL=$CRS_URL
set CUBE_IDENTITY_DIR=$agentDir
set CUBE_ROLE=inference
"$BinaryPath" >> "$logFile" 2>&1
"@ | Set-Content -Path $wrapperBat -Encoding ASCII

    $existingSvc = Get-Service -Name $svcName -ErrorAction SilentlyContinue
    if ($existingSvc) {
        Write-Host "  Updating existing service '$svcName'..." -ForegroundColor Yellow
        Stop-Service -Name $svcName -Force -ErrorAction SilentlyContinue
        & sc.exe delete $svcName | Out-Null
        Start-Sleep -Seconds 2
    }

    Write-Host "  Registering Windows Service: $svcName" -ForegroundColor Yellow

    $svcBinPath = "cmd.exe /c `"$wrapperBat`""
    New-Service -Name $svcName `
        -BinaryPathName $svcBinPath `
        -DisplayName $displayName `
        -Description "PlenumNET Inter-Cube infrastructure daemon for identity #$Id" `
        -StartupType Automatic | Out-Null

    & sc.exe failure $svcName reset= 86400 actions= restart/5000/restart/10000/restart/30000 | Out-Null
    & sc.exe config $svcName depend= Tcpip/Afd/Dnscache | Out-Null

    $svcAccount = Get-ServiceAccountName
    Set-ServiceLogonAccount -ServiceName $svcName -AccountName $svcAccount

    Start-Service -Name $svcName
    Write-Host "  Daemon #$Id registered and started as Windows Service." -ForegroundColor Green
    Write-Host "  Running as:    $svcAccount" -ForegroundColor DarkGray
    Write-Host "  Check status:  Get-Service $svcName" -ForegroundColor DarkGray
    Write-Host "  View logs:     Get-Content '$logFile' -Wait" -ForegroundColor DarkGray
}

function Invoke-Uninstall {
    param([int]$Id)
    Require-Admin
    $svcName = Get-ServiceName -Id $Id

    $svc = Get-Service -Name $svcName -ErrorAction SilentlyContinue
    if ($svc) {
        Stop-Service -Name $svcName -Force -ErrorAction SilentlyContinue
        & sc.exe delete $svcName | Out-Null
        Write-Host "  Daemon #$Id service removed." -ForegroundColor Green
    } else {
        Write-Host "  No service found for identity #$Id" -ForegroundColor Yellow
    }

    $wrapperBat = Join-Path $InstallDir "services\wrappers\cube-${Id}-start.bat"
    if (Test-Path $wrapperBat) {
        Remove-Item $wrapperBat -Force
    }
}

function Invoke-Logs {
    param([int]$Id)
    $logFile = Join-Path $LogDir "cube-${Id}.log"
    if (Test-Path $logFile) {
        Get-Content $logFile -Wait
    } else {
        Write-Host "  No log file found at $logFile" -ForegroundColor Yellow
        Write-Host "  Check Windows Event Viewer for service events." -ForegroundColor DarkGray
    }
}

function Invoke-Watchdog {
    Require-Admin
    $watchdogScript = Join-Path $InstallDir "services\wrappers\plenumnet-watchdog.ps1"
    $watchdogDir = Split-Path $watchdogScript -Parent
    if (-not (Test-Path $watchdogDir)) { New-Item -Path $watchdogDir -ItemType Directory -Force | Out-Null }
    @"
`$stopped = Get-Service PlenumNET-Cube-* -ErrorAction SilentlyContinue | Where-Object { `$_.Status -ne 'Running' }
foreach (`$svc in `$stopped) {
    try { Start-Service -Name `$svc.Name -ErrorAction Stop } catch {}
}
"@ | Set-Content -Path $watchdogScript -Encoding ASCII

    $taskName = "PlenumNET-Daemon-Watchdog"
    $existingTask = Get-ScheduledTask -TaskName $taskName -ErrorAction SilentlyContinue
    if ($existingTask) {
        Unregister-ScheduledTask -TaskName $taskName -Confirm:$false -ErrorAction SilentlyContinue
    }

    $action = New-ScheduledTaskAction -Execute "powershell.exe" -Argument "-NoProfile -ExecutionPolicy Bypass -WindowStyle Hidden -File `\"$watchdogScript`\""
    $triggerBoot = New-ScheduledTaskTrigger -AtStartup
    $triggerRepeat = New-ScheduledTaskTrigger -Once -At (Get-Date) -RepetitionInterval (New-TimeSpan -Minutes 5)
    $principal = New-ScheduledTaskPrincipal -UserId "SYSTEM" -RunLevel Highest -LogonType ServiceAccount
    $settings = New-ScheduledTaskSettingsSet -AllowStartIfOnBatteries -DontStopIfGoingOnBatteries -StartWhenAvailable -RestartCount 3 -RestartInterval (New-TimeSpan -Minutes 1)

    Register-ScheduledTask -TaskName $taskName -Action $action -Trigger @($triggerBoot, $triggerRepeat) -Principal $principal -Settings $settings -Description "PlenumNET daemon watchdog -- restarts stopped services every 5 minutes and on boot" | Out-Null
    Write-Host "  [OK] Watchdog scheduled task registered: $taskName" -ForegroundColor Green
    Write-Host "       Checks every 5 minutes + on boot -- restarts any stopped daemons" -ForegroundColor DarkGray
    Write-Host "       Script: $watchdogScript" -ForegroundColor DarkGray
    Write-Host "       Check:  Get-ScheduledTask -TaskName $taskName" -ForegroundColor DarkGray
}

switch ($Command) {
    "status"    { Invoke-Status -Id $IdentityId }
    "start"     {
        if ($IdentityId -le 0) { Write-Host "  ERROR: Identity ID required." -ForegroundColor Red; exit 1 }
        Invoke-Start -Id $IdentityId
    }
    "stop"      {
        if ($IdentityId -le 0) { Write-Host "  ERROR: Identity ID required." -ForegroundColor Red; exit 1 }
        Invoke-Stop -Id $IdentityId
    }
    "restart"   {
        if ($IdentityId -le 0) { Write-Host "  ERROR: Identity ID required." -ForegroundColor Red; exit 1 }
        Invoke-Restart -Id $IdentityId
    }
    "install"   {
        if ($IdentityId -le 0) { Write-Host "  ERROR: Identity ID required." -ForegroundColor Red; exit 1 }
        Invoke-Install -Id $IdentityId
    }
    "uninstall" {
        if ($IdentityId -le 0) { Write-Host "  ERROR: Identity ID required." -ForegroundColor Red; exit 1 }
        Invoke-Uninstall -Id $IdentityId
    }
    "logs"      {
        if ($IdentityId -le 0) { Write-Host "  ERROR: Identity ID required." -ForegroundColor Red; exit 1 }
        Invoke-Logs -Id $IdentityId
    }
    "watchdog"  { Invoke-Watchdog }
}
