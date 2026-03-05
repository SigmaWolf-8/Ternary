<# 
  PlenumNET TDNS - Browser Extension Installer
  Capomastro Holdings Ltd. - Applied Physics Division
  
  Run: irm https://raw.githubusercontent.com/SigmaWolf-8/Ternary/main/services/tdns-v2/install.ps1 | iex
#>

$ErrorActionPreference = "Stop"
$GH_RAW = "https://raw.githubusercontent.com/SigmaWolf-8/Ternary/main/services/tdns-v2/extension/chromium"
$INSTALL_DIR = "$env:LOCALAPPDATA\PlenumNET\extension"

Write-Host ""
Write-Host "  PlenumNET TDNS - Installing..." -ForegroundColor Yellow
Write-Host ""

# Download extension files individually
if (Test-Path $INSTALL_DIR) { Remove-Item $INSTALL_DIR -Recurse -Force }
New-Item -ItemType Directory -Path $INSTALL_DIR -Force | Out-Null

$files = @("manifest.json", "background.js", "popup.html", "icon16.png", "icon48.png", "icon128.png")
foreach ($f in $files) {
    Invoke-WebRequest -Uri "$GH_RAW/$f" -OutFile "$INSTALL_DIR\$f" -UseBasicParsing
}
$fileCount = $files.Count
Write-Host "  [OK] Downloaded extension v2.3.2 ($fileCount files)" -ForegroundColor Green

# Detect and install
$count = 0

# Chromium browsers - preferences-based external extension install
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
            @{ external_crx = $INSTALL_DIR; external_version = "2.3.2" } | ConvertTo-Json | Set-Content "$extJsonDir\plenumnet-tdns.json"
            $bName = $b.Name
            Write-Host "  [OK] $bName" -ForegroundColor Green
            $count++
        } catch {
            $bName = $b.Name
            Write-Host "  [--] $bName skipped" -ForegroundColor DarkGray
        }
    }
}

# Firefox
$ffProfiles = "$env:APPDATA\Mozilla\Firefox\Profiles"
if (Test-Path $ffProfiles) {
    try {
        Get-ChildItem $ffProfiles -Directory | ForEach-Object {
            $extDir = "$($_.FullName)\extensions"
            if (-not (Test-Path $extDir)) { New-Item -ItemType Directory -Path $extDir -Force | Out-Null }
            Copy-Item "$INSTALL_DIR\manifest.json" "$extDir\tdns-resolver@capomastro.com.json" -Force
        }
        Write-Host "  [OK] Firefox" -ForegroundColor Green
        $count++
    } catch {
        Write-Host "  [--] Firefox skipped" -ForegroundColor DarkGray
    }
}

Write-Host ""
if ($count -gt 0) {
    Write-Host "  Done - installed in $count browser(s)." -ForegroundColor Green
    Write-Host "  Restart your browser(s), then type: plm google" -ForegroundColor Cyan
} else {
    Write-Host "  No supported browsers found." -ForegroundColor Yellow
}
Write-Host ""
