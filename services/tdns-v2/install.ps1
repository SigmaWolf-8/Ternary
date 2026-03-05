<# 
  PlenumNET TDNS - Browser Extension Installer
  Capomastro Holdings Ltd. - Applied Physics Division
  
  Run: irm https://raw.githubusercontent.com/SigmaWolf-8/Ternary/main/services/tdns-v2/install.ps1 | iex
#>

$ErrorActionPreference = "Stop"
$GH_RAW = "https://raw.githubusercontent.com/SigmaWolf-8/Ternary/main/services/tdns-v2/extension/chromium"
$INSTALL_DIR = "$env:LOCALAPPDATA\PlenumNET\tdns-extension"

Write-Host ""
Write-Host "  PlenumNET TDNS - Browser Extension Installer" -ForegroundColor Yellow
Write-Host ""

# Download extension files
if (Test-Path $INSTALL_DIR) { Remove-Item $INSTALL_DIR -Recurse -Force }
New-Item -ItemType Directory -Path $INSTALL_DIR -Force | Out-Null

$files = @("manifest.json", "background.js", "content.js", "popup.html", "popup.js", "icon16.png", "icon48.png", "icon128.png")
foreach ($f in $files) {
    Invoke-WebRequest -Uri "$GH_RAW/$f" -OutFile "$INSTALL_DIR\$f" -UseBasicParsing
}
$fileCount = $files.Count
Write-Host "  [OK] Downloaded $fileCount extension files (v2.3.2)" -ForegroundColor Green
Write-Host ""

# Detect browsers
$found = @()
$browsers = @(
    @{ Name="Edge";    Data="$env:LOCALAPPDATA\Microsoft\Edge\User Data";                  Url="edge://extensions" },
    @{ Name="Chrome";  Data="$env:LOCALAPPDATA\Google\Chrome\User Data";                   Url="chrome://extensions" },
    @{ Name="Brave";   Data="$env:LOCALAPPDATA\BraveSoftware\Brave-Browser\User Data";     Url="brave://extensions" },
    @{ Name="Vivaldi"; Data="$env:LOCALAPPDATA\Vivaldi\User Data";                         Url="vivaldi://extensions" },
    @{ Name="Opera";   Data="$env:APPDATA\Opera Software\Opera Stable";                    Url="opera://extensions" }
)

foreach ($b in $browsers) {
    if (Test-Path $b.Data) {
        $found += $b
    }
}

Write-Host "  Extension saved to:" -ForegroundColor Cyan
Write-Host "  $INSTALL_DIR" -ForegroundColor White
Write-Host ""

if ($found.Count -gt 0) {
    Write-Host "  Detected browsers:" -ForegroundColor Cyan
    foreach ($b in $found) {
        $bName = $b.Name
        Write-Host "    - $bName" -ForegroundColor Green
    }
    Write-Host ""
    Write-Host "  To install, open your browser and:" -ForegroundColor Yellow
    Write-Host "    1. Go to the extensions page:" -ForegroundColor White
    $extUrl = $found[0].Url
    Write-Host "       $extUrl" -ForegroundColor Cyan
    Write-Host "    2. Enable 'Developer mode' (toggle in top-right)" -ForegroundColor White
    Write-Host "    3. Click 'Load unpacked'" -ForegroundColor White
    Write-Host "    4. Select this folder:" -ForegroundColor White
    Write-Host "       $INSTALL_DIR" -ForegroundColor Cyan
    Write-Host ""

    # Try to open the extensions page in the first detected browser
    $firstUrl = $found[0].Url
    $openIt = Read-Host "  Open $($found[0].Name) extensions page now? (Y/n)"
    if ($openIt -ne "n") {
        Start-Process $firstUrl
    }
} else {
    Write-Host "  No supported Chromium browsers detected." -ForegroundColor Yellow
}

# Copy path to clipboard
try {
    Set-Clipboard -Value $INSTALL_DIR
    Write-Host "  Extension path copied to clipboard." -ForegroundColor DarkGray
} catch {}

Write-Host ""
