<# 
  PlenumNET TDNS — Browser Extension Installer
  Capomastro Holdings Ltd. — Applied Physics Division
  
  Run: irm https://plenumnet.replit.app/install.ps1 | iex
#>

$ErrorActionPreference = "Stop"
$PLM_API = "https://plenumnet.replit.app"
$INSTALL_DIR = "$env:LOCALAPPDATA\PlenumNET\extension"

Write-Host ""
Write-Host "  PlenumNET TDNS — Installing..." -ForegroundColor Yellow
Write-Host ""

# Download
if (Test-Path $INSTALL_DIR) { Remove-Item $INSTALL_DIR -Recurse -Force }
New-Item -ItemType Directory -Path $INSTALL_DIR -Force | Out-Null
$zip = "$env:TEMP\plenumnet-ext.zip"
Invoke-WebRequest -Uri "$PLM_API/api/extension/chromium" -OutFile $zip -UseBasicParsing
Expand-Archive -Path $zip -DestinationPath $INSTALL_DIR -Force
Remove-Item $zip -Force
Write-Host "  [OK] Downloaded" -ForegroundColor Green

# Detect and install
$count = 0

# Chromium browsers — registry-based external extension install
$chromiumBrowsers = @(
    @{ Name="Chrome";  Data="$env:LOCALAPPDATA\Google\Chrome\User Data";           Reg="HKLM:\SOFTWARE\Google\Chrome\Extensions" },
    @{ Name="Edge";    Data="$env:LOCALAPPDATA\Microsoft\Edge\User Data";           Reg="HKLM:\SOFTWARE\Microsoft\Edge\Extensions" },
    @{ Name="Brave";   Data="$env:LOCALAPPDATA\BraveSoftware\Brave-Browser\User Data"; Reg="HKLM:\SOFTWARE\BraveSoftware\Brave-Browser\Extensions" },
    @{ Name="Vivaldi"; Data="$env:LOCALAPPDATA\Vivaldi\User Data";                 Reg="HKLM:\SOFTWARE\Vivaldi\Extensions" },
    @{ Name="Opera";   Data="$env:APPDATA\Opera Software\Opera Stable";            Reg="HKLM:\SOFTWARE\Opera Software\Opera Stable\Extensions" }
)

foreach ($b in $chromiumBrowsers) {
    if (Test-Path $b.Data) {
        try {
            # Preferences-based install: write to External Extensions
            $extJsonDir = "$($b.Data)\Default\External Extensions"
            if (-not (Test-Path $extJsonDir)) { New-Item -ItemType Directory -Path $extJsonDir -Force | Out-Null }
            @{ external_crx = $INSTALL_DIR; external_version = "2.3.2" } | ConvertTo-Json | Set-Content "$extJsonDir\plenumnet-tdns.json"
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
        Invoke-WebRequest -Uri "$PLM_API/api/extension/firefox" -OutFile $xpi -UseBasicParsing
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
    Write-Host "  Done — installed in $count browser(s). Restart browsers, then type: plm google" -ForegroundColor Green
} else {
    Write-Host "  No supported browsers found." -ForegroundColor Yellow
}
Write-Host ""
