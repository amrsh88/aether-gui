# Builds the Aether GUI installer from the cross-compiled binary.
#
#   powershell -ExecutionPolicy Bypass -File installer\build.ps1
#
# Expects to be run from anywhere; paths are resolved relative to this script.

$ErrorActionPreference = "Stop"

$root    = Split-Path -Parent $PSScriptRoot     # repo root
$dist    = Join-Path $root "dist"
$target  = Join-Path $root "src-tauri/target/x86_64-pc-windows-gnu/release"
$version = "1.0.0"

# --- sanity checks ------------------------------------------------------------

$needed = @(
    (Join-Path $target "$_/aether-gui.exe" -ErrorAction SilentlyContinue),
    (Join-Path $dist   "wintun.dll"),
    (Join-Path $dist   "WebView2Loader.dll")
)
# aether-gui.exe comes from the cargo target dir; the DLLs may come from either place.
$exe       = Join-Path $target "aether-gui.exe"
$wintun    = if (Test-Path (Join-Path $dist "wintun.dll")) { Join-Path $dist "wintun.dll" } else { Join-Path $root "src-tauri/resources/wintun.dll" }
$webview2  = if (Test-Path (Join-Path $dist "WebView2Loader.dll")) { Join-Path $dist "WebView2Loader.dll" } else { Join-Path $root "src-tauri/resources/WebView2Loader.dll" }

foreach ($file in @($exe, $wintun, $webview2)) {
    if (-not (Test-Path $file)) {
        Write-Host "MISSING: $file" -ForegroundColor Red
        Write-Host ""
        Write-Host "Expected layout:"
        Write-Host "  $exe            <- from 'cargo build --release --target x86_64-pc-windows-gnu'"
        Write-Host "  $wintun"
        Write-Host "  $webview2"
        exit 1
    }
}

if (-not (Get-Command makensis -ErrorAction SilentlyContinue)) {
    Write-Host "makensis not found on PATH." -ForegroundColor Red
    Write-Host "Install NSIS from https://nsis.sourceforge.io/Download"
    exit 1
}

# --- stage --------------------------------------------------------------------

New-Item -ItemType Directory -Force -Path $dist | Out-Null
Copy-Item $exe      $dist -Force
Copy-Item $wintun   $dist -Force
Copy-Item $webview2 $dist -Force

Write-Host "Staged:" -ForegroundColor Cyan
Get-ChildItem $dist | Format-Table Name, Length -AutoSize

# --- compile ------------------------------------------------------------------

Push-Location (Join-Path $root "installer")
& makensis installer.nsi
Pop-Location

$out = Join-Path $root "Aether-GUI-Setup-$version.exe"
if (Test-Path $out) {
    Write-Host ""
    Write-Host "Installer ready: $out" -ForegroundColor Green
    Get-Item $out | Format-Table Name, @{L="SizeMB";E={[math]::Round($_.Length/1MB,1)}} -AutoSize
} else {
    Write-Host "makensis reported success but the output file is missing." -ForegroundColor Red
    exit 1
}
