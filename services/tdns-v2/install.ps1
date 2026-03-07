<# 
  PlenumNET TDNS - Browser Extension Installer v1.0.4
  Capomastro Holdings Ltd. - Applied Physics Division
  
  Run: irm https://raw.githubusercontent.com/SigmaWolf-8/Ternary/main/services/tdns-v2/install.ps1 | iex
#>

$ErrorActionPreference = "Continue"
[Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12

$GH_RAW = "https://raw.githubusercontent.com/SigmaWolf-8/Ternary/main/services/tdns-v2/extension-chromium"
$BASE_DIR = [System.IO.Path]::Combine($env:LOCALAPPDATA, "PlenumNET", "tdns-extensions")
$OLD_DIR = [System.IO.Path]::Combine($env:LOCALAPPDATA, "PlenumNET", "tdns-extension")

$extensionFiles = @(
    @{ Name="manifest.json"; Dest="manifest.json" },
    @{ Name="background.js"; Dest="background.js" },
    @{ Name="content.js"; Dest="content.js" },
    @{ Name="popup.html"; Dest="popup.html" },
    @{ Name="popup.js"; Dest="popup.js" },
    @{ Name="dimensions.json"; Dest="dimensions.json" },
    @{ Name="report.html"; Dest="report.html" },
    @{ Name="report.js"; Dest="report.js" },
    @{ Name="icons/icon16.png"; Dest="icons\icon16.png" },
    @{ Name="icons/icon48.png"; Dest="icons\icon48.png" },
    @{ Name="icons/icon128.png"; Dest="icons\icon128.png" }
)

$browsers = @(
    @{ Name="Chrome";  Path=[System.IO.Path]::Combine($env:LOCALAPPDATA, "Google", "Chrome", "User Data");                    Dir="chrome";  Url="chrome://extensions" },
    @{ Name="Edge";    Path=[System.IO.Path]::Combine($env:LOCALAPPDATA, "Microsoft", "Edge", "User Data");                    Dir="edge";    Url="edge://extensions" },
    @{ Name="Brave";   Path=[System.IO.Path]::Combine($env:LOCALAPPDATA, "BraveSoftware", "Brave-Browser", "User Data");       Dir="brave";   Url="brave://extensions" },
    @{ Name="Vivaldi"; Path=[System.IO.Path]::Combine($env:LOCALAPPDATA, "Vivaldi", "User Data");                              Dir="vivaldi"; Url="vivaldi://extensions" },
    @{ Name="Opera";   Path=[System.IO.Path]::Combine($env:APPDATA, "Opera Software", "Opera Stable");                         Dir="opera";   Url="opera://extensions" },
    @{ Name="Arc";     Path=[System.IO.Path]::Combine($env:LOCALAPPDATA, "Arc", "User Data");                                  Dir="arc";     Url="arc://extensions" }
)

Write-Host ""
Write-Host "  PlenumNET TDNS - Browser Extension Installer v1.0.4" -ForegroundColor Yellow
Write-Host "  Capomastro Holdings Ltd. - Applied Physics Division" -ForegroundColor DarkGray
Write-Host ""

if (Test-Path $OLD_DIR) {
    Write-Host "  Removing old install at $OLD_DIR..." -ForegroundColor DarkGray
    Remove-Item $OLD_DIR -Recurse -Force -ErrorAction SilentlyContinue
    Write-Host "  [OK] Old install removed" -ForegroundColor Green
    Write-Host ""
    Write-Host "  IMPORTANT: If you previously loaded the extension from:" -ForegroundColor Red
    Write-Host "    $OLD_DIR" -ForegroundColor Yellow
    Write-Host "  Go to your browser extensions page and REMOVE that entry first," -ForegroundColor Red
    Write-Host "  then load the new path shown below." -ForegroundColor Red
    Write-Host ""
}

function Install-ForBrowser {
    param (
        [string]$BrowserName,
        [string]$InstallDir,
        [string]$ExtUrl
    )
    
    Write-Host "  [$BrowserName] Installing to $InstallDir" -ForegroundColor Cyan
    
    try {
        if (Test-Path $InstallDir) { 
            Remove-Item $InstallDir -Recurse -Force 
        }
        [System.IO.Directory]::CreateDirectory($InstallDir) | Out-Null
        [System.IO.Directory]::CreateDirectory([System.IO.Path]::Combine($InstallDir, "icons")) | Out-Null
        if (-not (Test-Path $InstallDir)) {
            Write-Host "  [$BrowserName] ERROR: Failed to create directory" -ForegroundColor Red
            return $false
        }
    } catch {
        Write-Host "  [$BrowserName] ERROR creating directory: $_" -ForegroundColor Red
        return $false
    }

    $downloaded = 0
    foreach ($f in $extensionFiles) {
        $url = "$GH_RAW/$($f.Name)"
        $dest = [System.IO.Path]::Combine($InstallDir, $f.Dest)
        try {
            $wc = New-Object System.Net.WebClient
            $wc.Headers.Add("User-Agent", "PlenumNET-Installer/1.0.4")
            $wc.DownloadFile($url, $dest)
            if ((Test-Path $dest) -and ((Get-Item $dest).Length -gt 0)) {
                $downloaded++
            } else {
                Write-Host "  [$BrowserName] WARN: $($f.Name) - empty or missing" -ForegroundColor Yellow
            }
        } catch {
            Write-Host "  [$BrowserName] FAIL: $($f.Name) - $_" -ForegroundColor Red
        }
    }

    $manifestPath = [System.IO.Path]::Combine($InstallDir, "manifest.json")
    if (Test-Path $manifestPath) {
        $manifest = Get-Content $manifestPath -Raw | ConvertFrom-Json
        $version = $manifest.version
        Write-Host "  [$BrowserName] Version: $version" -ForegroundColor Green
        if ($version -ne "1.0.4") {
            Write-Host "  [$BrowserName] WARNING: Expected v1.0.4 but got v$version" -ForegroundColor Red
        }
    }

    Write-Host "  [$BrowserName] Downloaded $downloaded of $($extensionFiles.Count) files" -ForegroundColor Green
    Write-Host "  [$BrowserName] Path: $InstallDir" -ForegroundColor DarkGray
    Write-Host "  [$BrowserName] Extensions page: $ExtUrl" -ForegroundColor DarkGray
    Write-Host ""
    return $true
}

Write-Host "  Detecting browsers..." -ForegroundColor DarkGray
Write-Host ""

$installedBrowsers = @()

foreach ($b in $browsers) {
    if (Test-Path $b.Path) {
        $installDir = [System.IO.Path]::Combine($BASE_DIR, $b.Dir)
        $result = Install-ForBrowser -BrowserName $b.Name -InstallDir $installDir -ExtUrl $b.Url
        if ($result) {
            $installedBrowsers += $b
        }
    }
}

if ($installedBrowsers.Count -eq 0) {
    Write-Host "  No supported Chromium browsers detected." -ForegroundColor Yellow
    Write-Host "  Installing to default location..." -ForegroundColor DarkGray
    Write-Host ""
    $defaultDir = [System.IO.Path]::Combine($BASE_DIR, "chromium")
    Install-ForBrowser -BrowserName "Default" -InstallDir $defaultDir -ExtUrl "chrome://extensions"
    $installedBrowsers += @{ Name="Default"; Dir="chromium"; Url="chrome://extensions" }
}

Write-Host "  ======================================================" -ForegroundColor Yellow
Write-Host "  IMPORTANT: Remove any old PlenumNET TDNS extension" -ForegroundColor Red
Write-Host "  from your browser FIRST, then load the new path." -ForegroundColor Red
Write-Host "" 
Write-Host "  TO FINISH INSTALLATION:" -ForegroundColor Yellow
Write-Host "  ======================================================" -ForegroundColor Yellow
Write-Host ""

foreach ($b in $installedBrowsers) {
    $bDir = [System.IO.Path]::Combine($BASE_DIR, $b.Dir)
    Write-Host "  $($b.Name):" -ForegroundColor Cyan
    Write-Host "    1. Open: $($b.Url)" -ForegroundColor White
    Write-Host "    2. Remove any existing PlenumNET TDNS extension" -ForegroundColor White
    Write-Host "    3. Enable 'Developer mode' (top-right toggle)" -ForegroundColor White
    Write-Host "    4. Click 'Load unpacked'" -ForegroundColor White
    Write-Host "    5. Paste: $bDir" -ForegroundColor Green
    Write-Host ""
}

if ($installedBrowsers.Count -gt 0) {
    $primaryUrl = $installedBrowsers[0].Url
    $primaryDir = [System.IO.Path]::Combine($BASE_DIR, $installedBrowsers[0].Dir)
    
    try {
        Set-Clipboard -Value $primaryDir
        Write-Host "  ($($installedBrowsers[0].Name) path copied to clipboard)" -ForegroundColor DarkGray
    } catch {}

    $openIt = Read-Host "  Open $($installedBrowsers[0].Name) extensions page now? (Y/n)"
    if ($openIt -ne "n") {
        Start-Process $primaryUrl
    }
    
    if ($installedBrowsers.Count -gt 1) {
        Write-Host ""
        Write-Host "  Remember to repeat steps for:" -ForegroundColor Yellow
        for ($i = 1; $i -lt $installedBrowsers.Count; $i++) {
            Write-Host "    - $($installedBrowsers[$i].Name) ($($installedBrowsers[$i].Url))" -ForegroundColor White
        }
    }
}

Write-Host ""
