<# 
  PlenumNET TDNS - Browser Extension Installer v1.0.4
  Capomastro Holdings Ltd. - Applied Physics Division
  
  Run: irm https://plenumnet.replit.app/api/install-script | iex
#>

$ErrorActionPreference = "Continue"
$ZIP_URL = "https://plenumnet.replit.app/api/extension-zip"
$INSTALL_DIR = [System.IO.Path]::Combine($env:LOCALAPPDATA, "PlenumNET", "tdns-extension")
$ZIP_FILE = [System.IO.Path]::Combine($env:TEMP, "plenumnet-tdns-extension.zip")

Write-Host ""
Write-Host "  PlenumNET TDNS - Browser Extension Installer v1.0.4" -ForegroundColor Yellow
Write-Host "  Target: $INSTALL_DIR" -ForegroundColor DarkGray
Write-Host ""

try {
    if (Test-Path $INSTALL_DIR) { 
        Remove-Item $INSTALL_DIR -Recurse -Force 
        Write-Host "  Cleaned old install" -ForegroundColor DarkGray
    }
    [System.IO.Directory]::CreateDirectory($INSTALL_DIR) | Out-Null
    if (-not (Test-Path $INSTALL_DIR)) {
        Write-Host "  ERROR: Failed to create directory: $INSTALL_DIR" -ForegroundColor Red
        Write-Host "  Try running PowerShell as Administrator" -ForegroundColor Yellow
        return
    }
    Write-Host "  [OK] Directory created" -ForegroundColor Green
} catch {
    Write-Host "  ERROR creating directory: $_" -ForegroundColor Red
    return
}

Write-Host "  Downloading extension package..." -ForegroundColor DarkGray
try {
    [Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12
    $wc = New-Object System.Net.WebClient
    $wc.Headers.Add("User-Agent", "PlenumNET-Installer/1.0.4")
    $wc.DownloadFile($ZIP_URL, $ZIP_FILE)
    Write-Host "  [OK] Downloaded" -ForegroundColor Green
} catch {
    Write-Host "  [FAIL] Download error: $_" -ForegroundColor Red
    return
}

Write-Host "  Extracting..." -ForegroundColor DarkGray
try {
    Expand-Archive -Path $ZIP_FILE -DestinationPath $INSTALL_DIR -Force
    Remove-Item $ZIP_FILE -Force -ErrorAction SilentlyContinue
    Write-Host "  [OK] Extracted" -ForegroundColor Green
} catch {
    Write-Host "  [FAIL] Extract error: $_" -ForegroundColor Red
    return
}

$actualFiles = Get-ChildItem $INSTALL_DIR -Recurse -File -ErrorAction SilentlyContinue
if ($actualFiles.Count -eq 0) {
    Write-Host "  ERROR: No files in $INSTALL_DIR" -ForegroundColor Red
    return
}

Write-Host ""
Write-Host "  Installed files:" -ForegroundColor DarkGray
foreach ($af in $actualFiles) {
    $afName = $af.FullName.Replace($INSTALL_DIR + "\", "")
    $afSize = $af.Length
    Write-Host "    $afName ($afSize bytes)" -ForegroundColor DarkGray
}
Write-Host ""

$found = @()
$browsers = @(
    @{ Name="Chrome"; Url="chrome://extensions" },
    @{ Name="Edge";   Url="edge://extensions" },
    @{ Name="Brave";  Url="brave://extensions" },
    @{ Name="Vivaldi";Url="vivaldi://extensions" },
    @{ Name="Opera";  Url="opera://extensions" }
)

$browserPaths = @{
    "Chrome"  = [System.IO.Path]::Combine($env:LOCALAPPDATA, "Google", "Chrome", "User Data")
    "Edge"    = [System.IO.Path]::Combine($env:LOCALAPPDATA, "Microsoft", "Edge", "User Data")
    "Brave"   = [System.IO.Path]::Combine($env:LOCALAPPDATA, "BraveSoftware", "Brave-Browser", "User Data")
    "Vivaldi" = [System.IO.Path]::Combine($env:LOCALAPPDATA, "Vivaldi", "User Data")
    "Opera"   = [System.IO.Path]::Combine($env:APPDATA, "Opera Software", "Opera Stable")
}

foreach ($b in $browsers) {
    $bName = $b.Name
    $bPath = $browserPaths[$bName]
    if (Test-Path $bPath) {
        $found += $b
    }
}

if ($found.Count -gt 0) {
    Write-Host "  Detected browsers:" -ForegroundColor Cyan
    foreach ($b in $found) {
        $bName = $b.Name
        Write-Host "    - $bName" -ForegroundColor Green
    }
    Write-Host ""
    Write-Host "  TO FINISH INSTALLATION:" -ForegroundColor Yellow
    Write-Host ""
    $extUrl = $found[0].Url
    Write-Host "    1. Open your browser to: $extUrl" -ForegroundColor White
    Write-Host "    2. Turn ON 'Developer mode' (top-right toggle)" -ForegroundColor White
    Write-Host "    3. Click 'Load unpacked'" -ForegroundColor White
    Write-Host "    4. Paste this path and press Enter:" -ForegroundColor White
    Write-Host "       $INSTALL_DIR" -ForegroundColor Cyan
    Write-Host ""

    try {
        Set-Clipboard -Value $INSTALL_DIR
        Write-Host "  (Path copied to clipboard)" -ForegroundColor DarkGray
    } catch {}

    $openIt = Read-Host "  Open extensions page now? (Y/n)"
    if ($openIt -ne "n") {
        Start-Process $extUrl
    }
} else {
    Write-Host "  No Chromium browsers detected." -ForegroundColor Yellow
}

Write-Host ""
