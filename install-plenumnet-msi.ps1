<#
  .SYNOPSIS
      PlenumNET Installer
      Downloads and installs pre-built PlenumNET products.

  .DESCRIPTION
      Served from https://plenumnet.replit.app/api/install-msi
      Run with:  irm https://plenumnet.replit.app/api/install-msi | iex
      Or download the .bat wrapper from the Distribution page.

      This script:
        1. Detects existing installations
        2. Checks prerequisites (WebView2 only)
        3. Downloads pre-built MSI packages from GitHub Releases
        4. Installs MSI packages (Inter-Cube Daemon first, then Launcher and NinjaExec)
        5. Displays installation summary

  .NOTES
      Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
      Applied Physics Division
  #>

  $ErrorActionPreference = "Continue"
  $REMOTE_CRS     = "https://plenumnet.replit.app"
  $GitHubOwner    = "SigmaWolf-8"
  $GitHubRepo     = "Ternary"
  $OutputDir      = Join-Path $env:TEMP "PlenumNET-Install"
  $LogFile        = Join-Path $env:TEMP "PlenumNET_MSI_Install.log"
  $installStart   = Get-Date

  $Products = @(
      @{ Name = "PlenumNET-Launcher"; DisplayName = "PlenumNET Launcher"; InstallDir = "PlenumNET-Launcher"; Binary = "plenum-launcher.exe"; AssetPrefix = "PlenumNET-Launcher"; Description = "Manage all your PlenumNET applications from one panel in your taskbar (auto-starts with Windows)" },
      @{ Name = "InterCubeDaemon"; DisplayName = "Inter-Cube Daemon"; InstallDir = "InterCubeDaemon"; Binary = "inter-cube-daemon.exe"; AssetPrefix = "InterCubeDaemon"; Description = "Connects your machine to the PlenumNET network (3 service instances)" },
      @{ Name = "NinjaExec"; DisplayName = "NinjaExec"; InstallDir = "NinjaExec"; Binary = "ninja-exec.exe"; AssetPrefix = "NinjaExec"; Description = "Securely signs and authenticates your PlenumNET operations (auto-starts with Windows)" }
  )

  function Write-Log {
      param([string]$Message, [string]$Color = "White")
      $ts = Get-Date -Format "HH:mm:ss"
      $logLine = "[$ts] $Message"
      Add-Content -Path $LogFile -Value $logLine -ErrorAction SilentlyContinue
      Write-Host $Message -ForegroundColor $Color
  }

  function Write-StepTime {
      param([string]$StepName)
      $elapsed = (Get-Date) - $installStart
      $mins = [math]::Floor($elapsed.TotalMinutes)
      $secs = $elapsed.Seconds
      Write-Log "  Elapsed: ${mins}m ${secs}s" "DarkGray"
  }

  function Test-Admin {
      $p = New-Object Security.Principal.WindowsPrincipal([Security.Principal.WindowsIdentity]::GetCurrent())
      return $p.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
  }

  function Test-Command($cmd) {
      try { Get-Command $cmd -ErrorAction Stop | Out-Null; return $true }
      catch { return $false }
  }

  function Get-InstalledProducts {
      $programFiles = $env:ProgramFiles
      $results = @()

      foreach ($product in $Products) {
          $installPath = Join-Path $programFiles "Capomastro\$($product.InstallDir)"
          $binaryPath = Join-Path $installPath $product.Binary
          $installed = $false
          $version = $null
          $running = $false

          if ((Test-Path $installPath) -and (Test-Path $binaryPath)) {
              $installed = $true

              $versionFile = Join-Path $installPath "version.txt"
              if (Test-Path $versionFile) {
                  $version = (Get-Content $versionFile -ErrorAction SilentlyContinue | Select-Object -First 1).Trim()
              }

              if (-not $version) {
                  try {
                      $regEntries = Get-ItemProperty "HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\*" -ErrorAction SilentlyContinue |
                          Where-Object { $_.DisplayName -like "*$($product.DisplayName)*" -or $_.DisplayName -like "*$($product.Name)*" }
                      if ($regEntries) {
                          $version = ($regEntries | Select-Object -First 1).DisplayVersion
                      }
                  } catch {}
              }
          }

          if ($product.Name -eq "InterCubeDaemon") {
              try {
                  $tcpTest = Test-NetConnection -ComputerName localhost -Port 11124 -WarningAction SilentlyContinue -ErrorAction SilentlyContinue
                  if ($tcpTest.TcpTestSucceeded) { $running = $true }
              } catch {}
          }

          $results += @{
              Name = $product.Name
              DisplayName = $product.DisplayName
              Installed = $installed
              Version = $version
              Running = $running
              InstallPath = $installPath
          }
      }

      return $results
  }

  # == Brand Banner =============================================================
  Write-Host ""
  Write-Host "==========================================================" -ForegroundColor Cyan
  Write-Host "       ____  " -ForegroundColor Cyan
  Write-Host "      |  _ \ " -ForegroundColor Cyan
  Write-Host "      | |_) |" -ForegroundColor Cyan
  Write-Host "      |  __/ " -ForegroundColor Cyan
  Write-Host "      |_|    " -ForegroundColor Cyan
  Write-Host "" -ForegroundColor Cyan
  Write-Host "  PlenumNET Installer" -ForegroundColor Cyan
  Write-Host "  Post-Quantum Internet Infrastructure" -ForegroundColor Cyan
  Write-Host "  Capomastro Holdings Ltd." -ForegroundColor Cyan
  Write-Host "==========================================================" -ForegroundColor Cyan
  Write-Host ""
  Write-Host "  This installer sets up the PlenumNET suite:" -ForegroundColor White
  Write-Host "    -- Launcher (application hub)" -ForegroundColor White
  Write-Host "    -- NinjaExec (signing agent)" -ForegroundColor White
  Write-Host "    -- Inter-Cube Daemon (network service)" -ForegroundColor White
  Write-Host ""
  Write-Host "  Learn more: https://plenumnet.replit.app" -ForegroundColor DarkGray
  Write-Host ""
  Write-Host "  Console colors: Cyan = brand, Green = success," -ForegroundColor DarkGray
  Write-Host "  Yellow = warning, Red = error, Gray = debug" -ForegroundColor DarkGray
  Write-Host "  Log file: $LogFile" -ForegroundColor DarkGray
  Write-Host ""

  if (-not (Test-Admin)) {
      Write-Host "  Administrator privileges required for MSI installation." -ForegroundColor Yellow
      Write-Host "  Elevating..." -ForegroundColor DarkGray
      $scriptPath = $MyInvocation.MyCommand.Definition
      if ($scriptPath) {
          Start-Process powershell.exe -Verb RunAs -ArgumentList "-NoProfile -ExecutionPolicy Bypass -File `"$scriptPath`""
          Write-Host "  Re-launched as Administrator. This window can be closed." -ForegroundColor Green
          exit 0
      } else {
          $tempScript = Join-Path $env:TEMP "install-plenumnet-msi-elevated.ps1"
          Invoke-WebRequest -Uri "$REMOTE_CRS/api/install-msi" -OutFile $tempScript -UseBasicParsing
          Start-Process powershell.exe -Verb RunAs -ArgumentList "-NoProfile -ExecutionPolicy Bypass -File `"$tempScript`""
          Write-Host "  Re-launched as Administrator. This window can be closed." -ForegroundColor Green
          exit 0
      }
  }

  # == Detect architecture =======================================================
  try {
      $cpuArch = [System.Runtime.InteropServices.RuntimeInformation]::OSArchitecture.ToString()
  } catch {
      $cpuArch = $env:PROCESSOR_ARCHITECTURE
      if (-not $cpuArch) { $cpuArch = "AMD64" }
  }

  $archFlag = "x86_64"
  if ($cpuArch -eq "Arm64") {
      $archFlag = "aarch64"
  }

  # == STEP 1/5: Pre-flight detection ============================================
  Write-Host ""
  Write-Log "STEP 1/5: Detecting existing installations" "Yellow"
  Write-Host "---"

  Write-Log "  Checking internet connectivity..." "White"
  try {
      $null = Invoke-WebRequest -Uri "https://api.github.com" -Method Head -TimeoutSec 10 -UseBasicParsing -ErrorAction Stop
      Write-Log "  [OK] Internet connectivity" "Green"
  } catch {
      Write-Log "  Error: Cannot reach GitHub. Check your internet connection." "Red"
      Write-Log "  PlenumNET packages are downloaded from GitHub Releases." "Red"
      Read-Host "Press Enter to close"
      exit 1
  }

  $installedProducts = Get-InstalledProducts
  $anyInstalled = $false
  $allInstalled = $true
  $freshNinjaExec = $true

  foreach ($p in $installedProducts) {
      if ($p.Installed) {
          $anyInstalled = $true
          $verStr = if ($p.Version) { "v$($p.Version)" } else { "version unknown" }
          $runStr = if ($p.Running) { " (running)" } else { "" }
          Write-Log "  [INSTALLED] $($p.DisplayName) -- $verStr$runStr" "Green"
          Write-Log "              $($p.InstallPath)" "DarkGray"
          if ($p.Name -eq "NinjaExec") { $freshNinjaExec = $false }
      } else {
          $allInstalled = $false
          Write-Log "  [NOT FOUND] $($p.DisplayName)" "DarkGray"
      }
  }

  Write-Log "  Architecture: $cpuArch ($archFlag)" "DarkGray"
  Write-StepTime "Detection"

  # == Pre-install summary with user choice ======================================
  Write-Host ""
  Write-Host "==========================================================" -ForegroundColor Cyan
  Write-Host "  Pre-Install Summary" -ForegroundColor Cyan
  Write-Host "==========================================================" -ForegroundColor Cyan
  Write-Host ""

  $programFiles = $env:ProgramFiles
  $appData = $env:APPDATA

  $installMode = "install"

  if ($anyInstalled) {
      if ($allInstalled) {
          Write-Host "  All PlenumNET products are already installed." -ForegroundColor Green
          Write-Host ""
          Write-Host "  Options:" -ForegroundColor White
          Write-Host "    U = Upgrade (download latest, update changed products)" -ForegroundColor White
          Write-Host "    R = Reinstall (download and reinstall everything)" -ForegroundColor White
          Write-Host "    C = Cancel" -ForegroundColor White
          Write-Host ""
          $choice = Read-Host "  Choose (U/R/C)"
          switch ($choice.ToUpper()) {
              "U" { $installMode = "upgrade" }
              "R" { $installMode = "reinstall" }
              default {
                  Write-Host ""
                  Write-Host "  Installation cancelled." -ForegroundColor Yellow
                  exit 0
              }
          }
      } else {
          Write-Host "  Some PlenumNET products are already installed." -ForegroundColor Yellow
          Write-Host ""
          Write-Host "  Options:" -ForegroundColor White
          Write-Host "    U = Upgrade (install missing, update existing)" -ForegroundColor White
          Write-Host "    R = Reinstall (download and reinstall everything)" -ForegroundColor White
          Write-Host "    C = Cancel" -ForegroundColor White
          Write-Host ""
          $choice = Read-Host "  Choose (U/R/C)"
          switch ($choice.ToUpper()) {
              "U" { $installMode = "upgrade" }
              "R" { $installMode = "reinstall" }
              default {
                  Write-Host ""
                  Write-Host "  Installation cancelled." -ForegroundColor Yellow
                  exit 0
              }
          }
      }
  } else {
      $actionWord = "installed"
      Write-Host "  The following products will be installed:" -ForegroundColor White
      Write-Host ""
      $idx = 1
      foreach ($product in $Products) {
          Write-Host "  $idx. $($product.DisplayName)" -ForegroundColor Green
          Write-Host "     $($product.Description)" -ForegroundColor White
          Write-Host "     Install: $programFiles\Capomastro\$($product.InstallDir)" -ForegroundColor DarkGray
          Write-Host "     Data:    $appData\$($product.InstallDir)" -ForegroundColor DarkGray
          Write-Host ""
          $idx++
      }

      $confirm = Read-Host "  Continue? (Y/N)"
      if ($confirm -ne 'Y' -and $confirm -ne 'y') {
          Write-Host ""
          Write-Host "  Installation cancelled." -ForegroundColor Yellow
          exit 0
      }
  }

  Write-Host ""

  # == STEP 2/5: Prerequisites (WebView2 only) ===================================
  Write-Host ""
  Write-Log "STEP 2/5: Checking prerequisites" "Yellow"
  Write-Host "---"

  $webview2Installed = $false
  try {
      $wv2Key = Get-ItemProperty -Path "HKLM:\SOFTWARE\WOW6432Node\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}" -ErrorAction SilentlyContinue
      if ($wv2Key) { $webview2Installed = $true }
  } catch {}
  if (-not $webview2Installed) {
      try {
          $wv2Appx = Get-AppxPackage "Microsoft.WebView2Runtime" -ErrorAction SilentlyContinue
          if ($wv2Appx) { $webview2Installed = $true }
      } catch {}
  }
  if ($webview2Installed) {
      Write-Log "  [OK] WebView2 Runtime" "Green"
  } else {
      Write-Log "  -> WebView2 Runtime not found -- installing..." "Yellow"
      $wv2Bootstrapper = Join-Path $env:TEMP "MicrosoftEdgeWebview2Setup.exe"
      try {
          Invoke-WebRequest -Uri "https://go.microsoft.com/fwlink/p/?LinkId=2124703" -OutFile $wv2Bootstrapper -UseBasicParsing
          Start-Process -FilePath $wv2Bootstrapper -ArgumentList "/silent /install" -Wait -NoNewWindow
          Remove-Item $wv2Bootstrapper -Force -ErrorAction SilentlyContinue
          Write-Log "  [OK] WebView2 Runtime installed" "Green"
      } catch {
          Write-Log "  Warning: WebView2 Runtime install failed." "Yellow"
          Write-Log "  The Launcher popup may not work without it." "Yellow"
          Write-Log "  Download from: https://developer.microsoft.com/en-us/microsoft-edge/webview2/" "Yellow"
      }
  }

  Write-StepTime "Prerequisites"

  # == STEP 3/5: Download pre-built packages from GitHub Releases =================
  Write-Host ""
  Write-Log "STEP 3/5: Downloading PlenumNET packages (< 2 minutes)" "Yellow"
  Write-Host "---"

  New-Item -ItemType Directory -Force -Path $OutputDir | Out-Null

  $releaseInfo = $null
  $releaseVersion = $null
  try {
      Write-Log "  Fetching latest release from GitHub..." "White"
      $headers = @{ "User-Agent" = "PlenumNET-Installer/1.0" }
      $releaseInfo = Invoke-RestMethod -Uri "https://api.github.com/repos/$GitHubOwner/$GitHubRepo/releases/latest" -Headers $headers -ErrorAction Stop
      $releaseVersion = $releaseInfo.tag_name -replace "^v", ""
      Write-Log "  [OK] Latest release: $($releaseInfo.tag_name)" "Green"
  } catch {
      Write-Log "  Error: Could not fetch release information from GitHub." "Red"
      Write-Log "  Check your internet connection and try again." "Red"
      Write-Log "  $_" "DarkGray"
      Read-Host "Press Enter to close"
      exit 1
  }

  $generatedMSIs = @()
  $skippedProducts = @()

  foreach ($product in $Products) {
      $assetName = "$($product.AssetPrefix)-$archFlag.msi"
      $asset = $releaseInfo.assets | Where-Object { $_.name -eq $assetName }

      if (-not $asset) {
          Write-Log "  Warning: No package found for $($product.DisplayName) ($assetName)" "Yellow"
          Write-Log "  This product will not be installed." "Yellow"
          continue
      }

      if ($installMode -eq "upgrade") {
          $existing = $installedProducts | Where-Object { $_.Name -eq $product.Name }
          if ($existing -and $existing.Installed -and $existing.Version -eq $releaseVersion) {
              Write-Log "  [SKIP] $($product.DisplayName) v$releaseVersion -- already current" "Green"
              $skippedProducts += $product.Name
              continue
          }
      }

      $destPath = Join-Path $OutputDir $assetName
      $sizeMB = [math]::Round($asset.size / 1MB, 1)
      Write-Log "  Downloading $($product.DisplayName) ($sizeMB MB)..." "White"

      try {
          $downloadUrl = $asset.browser_download_url
          $webClient = New-Object System.Net.WebClient
          $webClient.Headers.Add("User-Agent", "PlenumNET-Installer/1.0")
          $webClient.DownloadFile($downloadUrl, $destPath)
          $webClient.Dispose()

          if (Test-Path $destPath) {
              $generatedMSIs += @{ Path = $destPath; Product = $product.Name; FileName = $assetName }
              Write-Log "  [OK] $assetName" "Green"
          } else {
              Write-Log "  Error: Download failed for $assetName" "Red"
          }
      } catch {
          Write-Log "  Error: Download failed for $($product.DisplayName)" "Red"
          Write-Log "  $_" "DarkGray"
      }
  }

  if ($generatedMSIs.Count -eq 0 -and $skippedProducts.Count -gt 0) {
      Write-Host ""
      Write-Log "  All products are already at the latest version ($($releaseInfo.tag_name))." "Green"
      Write-Log "  Nothing to do." "Green"
      Write-Host ""
      Write-Host "==========================================================" -ForegroundColor Cyan
      Write-Host "  Everything is up to date. Thank you for choosing Capomastro." -ForegroundColor Cyan
      Write-Host "  Capomastro Holdings Ltd. -- Applied Physics Division" -ForegroundColor Cyan
      Write-Host "==========================================================" -ForegroundColor Cyan
      Write-Host ""
      Read-Host "Press Enter to close"
      exit 0
  }

  if ($generatedMSIs.Count -eq 0) {
      Write-Host ""
      Write-Log "  No packages were downloaded." "Red"
      Write-Log "  The release may not contain packages for your" "Red"
      Write-Log "  architecture ($archFlag). Check:" "Red"
      Write-Log "    https://github.com/$GitHubOwner/$GitHubRepo/releases" "Red"
      Read-Host "Press Enter to close"
      exit 1
  }

  Write-StepTime "Download"

  # == NinjaExec Passphrase Setup ================================================
  $passphraseTempFile = $null
  $ninjaExecMsi = $generatedMSIs | Where-Object { $_.Product -eq "NinjaExec" } | Select-Object -First 1

  if ($ninjaExecMsi -and $freshNinjaExec) {
      Write-Host ""
      Write-Host "==========================================================" -ForegroundColor Cyan
      Write-Host "  NinjaExec Passphrase Setup" -ForegroundColor Cyan
      Write-Host "==========================================================" -ForegroundColor Cyan
      Write-Host ""
      Write-Host "  Create a passphrase to protect your signing key." -ForegroundColor White
      Write-Host ""
      Write-Host "  This passphrase encrypts your NinjaExec identity." -ForegroundColor White
      Write-Host "  You will need it to authorize signing operations." -ForegroundColor White
      Write-Host ""
      Write-Host "  Choose something memorable -- at least 12 characters." -ForegroundColor Yellow
      Write-Host ""

      $passphraseAccepted = $false
      while (-not $passphraseAccepted) {
          $secPass = Read-Host "  Enter passphrase" -AsSecureString
          $bstr = [System.Runtime.InteropServices.Marshal]::SecureStringToBSTR($secPass)
          $plainPass = [System.Runtime.InteropServices.Marshal]::PtrToStringAuto($bstr)
          [System.Runtime.InteropServices.Marshal]::ZeroFreeBSTR($bstr)

          if ($plainPass.Length -lt 12) {
              Write-Host "  Passphrase must be at least 12 characters. You entered $($plainPass.Length)." -ForegroundColor Red
              $plainPass = ""
              continue
          }

          $secConfirm = Read-Host "  Confirm passphrase" -AsSecureString
          $bstrC = [System.Runtime.InteropServices.Marshal]::SecureStringToBSTR($secConfirm)
          $confirmPass = [System.Runtime.InteropServices.Marshal]::PtrToStringAuto($bstrC)
          [System.Runtime.InteropServices.Marshal]::ZeroFreeBSTR($bstrC)

          if ($plainPass -ne $confirmPass) {
              Write-Host "  Passphrases do not match. Please try again." -ForegroundColor Red
              $plainPass = ""
              $confirmPass = ""
              continue
          }
          $confirmPass = ""

          $passphraseAccepted = $true
          Write-Host "  Passphrase accepted -- $($plainPass.Length) characters." -ForegroundColor Green
          Write-Host ""

          $passphraseTempFile = Join-Path $env:TEMP "plenumnet-passphrase-$([guid]::NewGuid().ToString('N')).tmp"
          try {
              [System.IO.File]::WriteAllText($passphraseTempFile, $plainPass)
              $acl = Get-Acl $passphraseTempFile
              $acl.SetAccessRuleProtection($true, $false)
              $rule = New-Object System.Security.AccessControl.FileSystemAccessRule(
                  [System.Security.Principal.WindowsIdentity]::GetCurrent().Name,
                  "FullControl",
                  "Allow"
              )
              $acl.AddAccessRule($rule)
              Set-Acl -Path $passphraseTempFile -AclObject $acl
          } catch {
              Write-Log "  Warning: Could not set ACL on passphrase file." "Yellow"
          }
          $plainPass = ""
      }
  }

  # == STEP 4/5: Install MSI packages (1-3 minutes) =============================
  Write-Host ""
  Write-Log "STEP 4/5: Installing PlenumNET products (1-3 minutes)" "Yellow"
  Write-Host "---"

  if ($generatedMSIs.Count -eq 0) {
      Write-Log "  No MSI files available. Skipping installation." "Yellow"
  } else {
      # Install order: Inter-Cube Daemon FIRST (Launcher depends on it)
      $daemonMsi = $generatedMSIs | Where-Object { $_.Product -eq "InterCubeDaemon" } | Select-Object -First 1
      $launcherMsi = $generatedMSIs | Where-Object { $_.Product -eq "PlenumNET-Launcher" } | Select-Object -First 1
      $otherMsis = $generatedMSIs | Where-Object { $_.Product -ne "InterCubeDaemon" -and $_.Product -ne "PlenumNET-Launcher" }

      # Check if daemon is already running
      $daemonAlreadyRunning = $false
      try {
          $tcpTest = Test-NetConnection -ComputerName localhost -Port 11124 -WarningAction SilentlyContinue -ErrorAction SilentlyContinue
          if ($tcpTest.TcpTestSucceeded) {
              $daemonAlreadyRunning = $true
              Write-Log "  Existing Inter-Cube Daemon detected on port 11124." "Green"
              Write-Log "  Skipping daemon installation -- Launcher will connect" "Green"
              Write-Log "  to the existing cluster." "Green"
          }
      } catch {}

      # 1. Install Inter-Cube Daemon first (if not already running)
      if ($daemonMsi -and -not $daemonAlreadyRunning) {
          $msiPath = $daemonMsi.Path
          Write-Log "  Installing Inter-Cube Daemon..." "White"
          $msiLog = Join-Path $env:TEMP "PlenumNET_InterCubeDaemon_install.log"
          $proc = Start-Process -FilePath "msiexec.exe" -ArgumentList "/i `"$msiPath`" /qb /l*v `"$msiLog`"" -Wait -PassThru
          if ($proc.ExitCode -eq 0) {
              Write-Log "  [OK] Inter-Cube Daemon installed" "Green"

              # Start the daemon service and wait for it to be ready
              Write-Log "  Starting Inter-Cube Daemon service..." "White"
              try {
                  & sc.exe start InterCubeDaemon 2>&1 | Out-Null
              } catch {}

              $daemonReady = $false
              $waitStart = Get-Date
              while (((Get-Date) - $waitStart).TotalSeconds -lt 30) {
                  Start-Sleep -Seconds 2
                  try {
                      $tcpTest = Test-NetConnection -ComputerName localhost -Port 11124 -WarningAction SilentlyContinue -ErrorAction SilentlyContinue
                      if ($tcpTest.TcpTestSucceeded) {
                          $daemonReady = $true
                          break
                      }
                  } catch {}
              }

              if ($daemonReady) {
                  Write-Log "  [OK] Inter-Cube Daemon is running on port 11124" "Green"
              } else {
                  Write-Log "  Warning: Inter-Cube Daemon did not start within 30s." "Yellow"
                  Write-Log "  The Launcher may show 'Connecting to daemon...' until" "Yellow"
                  Write-Log "  the service starts. You can start it manually:" "Yellow"
                  Write-Log "    sc.exe start InterCubeDaemon" "Yellow"
              }
          } else {
              $exitCode = $proc.ExitCode
              Write-Log "  Warning: Inter-Cube Daemon install exited with code $exitCode." "Yellow"
              Write-Log "  Log: $msiLog" "Yellow"
          }
      }

      # 2. Install PlenumNET Launcher
      if ($launcherMsi) {
          $msiPath = $launcherMsi.Path
          Write-Log "  Installing PlenumNET Launcher..." "White"
          $msiLog = Join-Path $env:TEMP "PlenumNET_Launcher_install.log"
          $proc = Start-Process -FilePath "msiexec.exe" -ArgumentList "/i `"$msiPath`" /qb /l*v `"$msiLog`"" -Wait -PassThru
          if ($proc.ExitCode -eq 0) {
              Write-Log "  [OK] PlenumNET Launcher installed" "Green"
          } else {
              $exitCode = $proc.ExitCode
              Write-Log "  Warning: Launcher install exited with code $exitCode." "Yellow"
              Write-Log "  Log: $msiLog" "Yellow"
          }
      }

      # 3. Install NinjaExec (with passphrase file if available)
      foreach ($msiEntry in $otherMsis) {
          $msiPath = $msiEntry.Path
          $msiBaseName = $msiEntry.Product
          Write-Log "  Installing $msiBaseName..." "White"
          $msiLog = Join-Path $env:TEMP "PlenumNET_${msiBaseName}_install.log"
          $msiArgs = "/i `"$msiPath`" /qb /l*v `"$msiLog`""
          if ($msiBaseName -eq "NinjaExec" -and $passphraseTempFile -and (Test-Path $passphraseTempFile)) {
              $msiArgs = "/i `"$msiPath`" /qb /l*v `"$msiLog`" PASSPHRASE_FILE=`"$passphraseTempFile`""
          }
          $proc = Start-Process -FilePath "msiexec.exe" -ArgumentList $msiArgs -Wait -PassThru
          if ($proc.ExitCode -eq 0) {
              Write-Log "  [OK] $msiBaseName installed" "Green"
          } else {
              $exitCode = $proc.ExitCode
              Write-Log "  Warning: $msiBaseName install exited with code $exitCode." "Yellow"
              Write-Log "  Log: $msiLog" "Yellow"
          }
      }

      # Write version markers for future detection
      foreach ($msiEntry in $generatedMSIs) {
          $product = $Products | Where-Object { $_.Name -eq $msiEntry.Product } | Select-Object -First 1
          if ($product -and $releaseVersion) {
              $installPath = Join-Path $programFiles "Capomastro\$($product.InstallDir)"
              if (Test-Path $installPath) {
                  $versionFile = Join-Path $installPath "version.txt"
                  try {
                      Set-Content -Path $versionFile -Value $releaseVersion -ErrorAction SilentlyContinue
                  } catch {}
              }
          }
      }
  }

  # Clean up passphrase temp file
  if ($passphraseTempFile -and (Test-Path $passphraseTempFile)) {
      try {
          Remove-Item -Path $passphraseTempFile -Force -ErrorAction SilentlyContinue
      } catch {}
  }

  Write-StepTime "Installation"

  # == STEP 5/5: Summary ========================================================
  Write-Host ""
  Write-Log "STEP 5/5: Summary" "Yellow"
  Write-Host "==========================================================" -ForegroundColor Cyan
  Write-Host ""

  $elapsed = (Get-Date) - $installStart
  $totalMins = [math]::Floor($elapsed.TotalMinutes)
  $totalSecs = $elapsed.Seconds

  $msiCount = $generatedMSIs.Count
  if ($msiCount -gt 0) {
      $actionWord = if ($installMode -eq "upgrade") { "Upgrade" } elseif ($installMode -eq "reinstall") { "Reinstall" } else { "Installation" }
      Write-Host "  $actionWord complete. PlenumNET is ready." -ForegroundColor Green
      Write-Host ""
      Write-Host "  Your applications will appear in the system tray" -ForegroundColor White
      Write-Host "  momentarily." -ForegroundColor White
      Write-Host ""

      if ($installMode -eq "upgrade" -and $skippedProducts.Count -gt 0) {
          Write-Host "  Skipped (already current):" -ForegroundColor DarkGray
          foreach ($skipped in $skippedProducts) {
              $p = $Products | Where-Object { $_.Name -eq $skipped } | Select-Object -First 1
              if ($p) { Write-Host "    -- $($p.DisplayName) v$releaseVersion" -ForegroundColor DarkGray }
          }
          Write-Host ""
      }

      Write-Host "  Updated products:" -ForegroundColor Green
      foreach ($msiEntry in $generatedMSIs) {
          $msiFileName = $msiEntry.FileName
          Write-Host "    -- $msiFileName" -ForegroundColor Green
      }

      Write-Host ""
      Write-Host "  Install locations:" -ForegroundColor White
      Write-Host "    PlenumNET Launcher:  $programFiles\Capomastro\PlenumNET-Launcher" -ForegroundColor White
      Write-Host "    NinjaExec:           $programFiles\Capomastro\NinjaExec" -ForegroundColor White
      Write-Host "    Inter-Cube Daemon:   $programFiles\Capomastro\InterCubeDaemon" -ForegroundColor White
      Write-Host ""
      Write-Host "  Data directories (preserved on uninstall):" -ForegroundColor White
      Write-Host "    $appData\PlenumNET-Launcher" -ForegroundColor White
      Write-Host "    $appData\NinjaExec" -ForegroundColor White
      Write-Host "    $appData\InterCubeDaemon" -ForegroundColor White
      Write-Host ""
      Write-Host "  To get started:" -ForegroundColor White
      Write-Host "    https://plenumnet.replit.app/getting-started" -ForegroundColor Cyan
      Write-Host ""
      Write-Host "  To uninstall:" -ForegroundColor White
      Write-Host "    Settings > Apps > search 'PlenumNET' or 'Capomastro'" -ForegroundColor White
      Write-Host "    Uninstall order: NinjaExec first, then Launcher, then" -ForegroundColor DarkGray
      Write-Host "    Inter-Cube Daemon last. Daemon services are stopped" -ForegroundColor DarkGray
      Write-Host "    automatically. Data directories are preserved by" -ForegroundColor DarkGray
      Write-Host "    default for reinstallation." -ForegroundColor DarkGray
  } else {
      Write-Log "  No products were installed." "Yellow"
  }

  Write-Host ""
  Write-Host "  Architecture:  $cpuArch" -ForegroundColor DarkGray
  Write-Host "  Release:       $($releaseInfo.tag_name)" -ForegroundColor DarkGray
  Write-Host "  Log file:      $LogFile" -ForegroundColor DarkGray
  Write-Host "  Total time:    ${totalMins}m ${totalSecs}s" -ForegroundColor DarkGray
  Write-Host ""
  Write-Host "==========================================================" -ForegroundColor Cyan
  Write-Host "  Thank you for choosing Capomastro." -ForegroundColor Cyan
  Write-Host "  Capomastro Holdings Ltd. -- Applied Physics Division" -ForegroundColor Cyan
  Write-Host "==========================================================" -ForegroundColor Cyan
  Write-Host ""
  Read-Host "Press Enter to close"
  