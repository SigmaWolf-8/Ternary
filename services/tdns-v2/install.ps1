<# 
  PlenumNET TDNS - Browser Extension Installer
  Capomastro Holdings Ltd. - Applied Physics Division
  
  Run: irm https://raw.githubusercontent.com/SigmaWolf-8/Ternary/main/services/tdns-v2/install.ps1 | iex
#>

$ErrorActionPreference = "Continue"
$GH_RAW = "https://raw.githubusercontent.com/SigmaWolf-8/Ternary/main/services/tdns-v2/extension/chromium"
$INSTALL_DIR = [System.IO.Path]::Combine($env:LOCALAPPDATA, "PlenumNET", "tdns-extension")

Write-Host ""
Write-Host "  PlenumNET TDNS - Browser Extension Installer" -ForegroundColor Yellow
Write-Host "  Target: $INSTALL_DIR" -ForegroundColor DarkGray
Write-Host ""

# Create directory
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

# Download each file
$files = @("manifest.json", "background.js", "content.js", "popup.html", "popup.js", "icon16.png", "icon48.png", "icon128.png")
$downloaded = 0

foreach ($f in $files) {
    $url = "$GH_RAW/$f"
    $dest = [System.IO.Path]::Combine($INSTALL_DIR, $f)
    try {
        $wc = New-Object System.Net.WebClient
        $wc.Headers.Add("User-Agent", "PlenumNET-Installer/2.3.2")
        $wc.DownloadFile($url, $dest)
        if (Test-Path $dest) {
            $size = (Get-Item $dest).Length
            if ($size -gt 0) {
                $downloaded++
            } else {
                Write-Host "  [FAIL] $f - empty file" -ForegroundColor Red
            }
        } else {
            Write-Host "  [FAIL] $f - file not saved" -ForegroundColor Red
        }
    } catch {
        Write-Host "  [FAIL] $f - $_" -ForegroundColor Red
    }
}

Write-Host "  [OK] Downloaded $downloaded of $($files.Count) files" -ForegroundColor Green
Write-Host ""

# Verify folder exists and has files
$actualFiles = Get-ChildItem $INSTALL_DIR -ErrorAction SilentlyContinue
if ($actualFiles.Count -eq 0) {
    Write-Host "  ERROR: No files in $INSTALL_DIR" -ForegroundColor Red
    Write-Host "  Downloads may have failed. Check your internet connection." -ForegroundColor Yellow
    return
}

Write-Host "  Installed files:" -ForegroundColor DarkGray
foreach ($af in $actualFiles) {
    $afName = $af.Name
    $afSize = $af.Length
    Write-Host "    $afName ($afSize bytes)" -ForegroundColor DarkGray
}
Write-Host ""

# Detect browsers
$found = @()
$browsers = @(
    @{ Name="Edge";    Url="edge://extensions" },
    @{ Name="Chrome";  Url="chrome://extensions" },
    @{ Name="Brave";   Url="brave://extensions" },
    @{ Name="Vivaldi"; Url="vivaldi://extensions" },
    @{ Name="Opera";   Url="opera://extensions" }
)

$edgeData = [System.IO.Path]::Combine($env:LOCALAPPDATA, "Microsoft", "Edge", "User Data")
$chromeData = [System.IO.Path]::Combine($env:LOCALAPPDATA, "Google", "Chrome", "User Data")
$braveData = [System.IO.Path]::Combine($env:LOCALAPPDATA, "BraveSoftware", "Brave-Browser", "User Data")
$vivaldiData = [System.IO.Path]::Combine($env:LOCALAPPDATA, "Vivaldi", "User Data")
$operaData = [System.IO.Path]::Combine($env:APPDATA, "Opera Software", "Opera Stable")

$browserPaths = @{
    "Edge" = $edgeData
    "Chrome" = $chromeData
    "Brave" = $braveData
    "Vivaldi" = $vivaldiData
    "Opera" = $operaData
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

    # Copy path to clipboard
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
