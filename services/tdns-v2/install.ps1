<# 
  PlenumNET TDNS — Browser Extension Installer
  Capomastro Holdings Ltd. — Applied Physics Division
  
  Run: irm https://raw.githubusercontent.com/SigmaWolf-8/Ternary/main/services/tdns-v2/install.ps1 | iex
#>

$ErrorActionPreference = "Stop"
$GH_RAW = "https://raw.githubusercontent.com/SigmaWolf-8/Ternary/main/services/tdns-v2"
$INSTALL_DIR = "$env:LOCALAPPDATA\PlenumNET\extension"

Write-Host ""
Write-Host "  PlenumNET TDNS — Installing..." -ForegroundColor Yellow
Write-Host ""

# Download
if (Test-Path $INSTALL_DIR) { Remove-Item $INSTALL_DIR -Recurse -Force }
New-Item -ItemType Directory -Path $INSTALL_DIR -Force | Out-Null
$zip = "$env:TEMP\plenumnet-ext.zip"
Invoke-WebRequest -Uri "$GH_RAW/chromium-extension.zip" -OutFile $zip -UseBasicParsing
Expand-Archive -Path $zip -DestinationPath $INSTALL_DIR -Force
Remove-Item $zip -Force
Write-Host "  [OK] Downloaded extension v2.3.2" -ForegroundColor Green

# Detect and install
$count = 0

# Chromium browsers — preferences-based external extension install
$chromiumBrowsers = @(
    @{ Name="Chrome";  Data="$env:LOCALAPPDATA\Google\Chrome\User Data" },
    @{ Name="Edge";    Data="$env:LOCALAPPDATA\Microsoft\Edge\User Data" },
    @{ Name="Brave";   Data="$env:LOCALAPPDATA\BraveSoftware\Brave-Browser\User Data" },
    @{ Name="Vivaldi"; Data="$env:LOCALAPPDATA\Vivaldi\User Data" },
    @{ Name="Opera";   Data="$env:APPDATA\Opera Software\Opera Stable" }
)

foreach ($b in $chromiumBrowsers) {
    if (Test-Path $b.Data) {
        try {
            $extJsonDir = "$($b.Data)\Default\External Extensions"
            if (-not (Test-Path $extJsonDir)) { New-Item -ItemType Directory -Path $extJsonDir -Force | Out-Null }
            @{ external_crx = "$INSTALL_DIR\manifest.json" ; external_version = "2.3.2" } | ConvertTo-Json | Set-Content "$extJsonDir\plenumnet-tdns.json"
            Write-Host "  [OK] $($b.Name)" -ForegroundColor Green
            $count++
        } catch {
            Write-Host "  [--] $($b.Name) skipped" -ForegroundColor DarkGray
        }
    }
}

# Firefox
$ffProfiles = "$env:APPDATA\Mozilla\Firefox\Profiles"
if (Test-Path $ffProfiles) {
    try {
        $xpi = "$env:TEMP\plenumnet-tdns.xpi"
        Invoke-WebRequest -Uri "$GH_RAW/chromium-extension.zip" -OutFile $xpi -UseBasicParsing
        Get-ChildItem $ffProfiles -Directory | ForEach-Object {
            $extDir = "$($_.FullName)\extensions"
            if (-not (Test-Path $extDir)) { New-Item -ItemType Directory -Path $extDir -Force | Out-Null }
            Copy-Item $xpi "$extDir\tdns-resolver@capomastro.com.xpi" -Force
        }
        Remove-Item $xpi -Force
        Write-Host "  [OK] Firefox" -ForegroundColor Green
        $count++
    } catch {
        Write-Host "  [--] Firefox skipped" -ForegroundColor DarkGray
    }
}

Write-Host ""
if ($count -gt 0) {
    Write-Host "  Done — installed in $count browser(s)." -ForegroundColor Green
    Write-Host "  Restart your browser(s), then type: plm google" -ForegroundColor Cyan
} else {
    Write-Host "  No supported browsers found." -ForegroundColor Yellow
}
Write-Host ""
