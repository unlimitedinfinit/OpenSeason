<#
.SYNOPSIS
    OpenSeason Document Builder launch script.
.DESCRIPTION
    Easily build, test, and run the OpenSeason application.
.EXAMPLE
    .\launch.ps1            # Opens interactive menu
    .\launch.ps1 run        # Runs standalone OpenSeason.exe
    .\launch.ps1 dev        # Launches Tauri dev server + window
    .\launch.ps1 test       # Runs end-to-end tests
    .\launch.ps1 build      # Builds production release .exe
    .\launch.ps1 cases      # Opens Cases directory in File Explorer
#>
[CmdletBinding()]
param (
    [Parameter(Position = 0)]
    [ValidateSet("run", "exe", "dev", "test", "build", "cases", "packs", "")]
    [string]$Mode = ""
)

$ErrorActionPreference = "Stop"
$RootDir = $PSScriptRoot
Set-Location $RootDir

function Show-Menu {
    Clear-Host
    Write-Host "=======================================================" -ForegroundColor Cyan
    Write-Host "         OpenSeason - Document Builder Launcher        " -ForegroundColor Cyan
    Write-Host "=======================================================" -ForegroundColor Cyan
    Write-Host ""
    Write-Host "  [1] Launch Standalone App (OpenSeason.exe - Final Version)" -ForegroundColor Green
    Write-Host "  [2] Launch Dev Mode (Vite server + live reload)" -ForegroundColor Yellow
    Write-Host "  [3] Run Agent E2E Test Suite (verify PDF and checks)" -ForegroundColor Magenta
    Write-Host "  [4] Rebuild Release Executable (OpenSeason.exe)" -ForegroundColor DarkYellow
    Write-Host "  [5] Open Cases Folder (Documents\JustLegal\Cases)" -ForegroundColor White
    Write-Host "  [6] Open Packs Folder (Documents\JustLegal\Packs)" -ForegroundColor White
    Write-Host "  [7] Exit" -ForegroundColor Gray
    Write-Host ""
    $choice = Read-Host "Select an option [1-7] (Default is 1)"
    if ([string]::IsNullOrWhiteSpace($choice)) { $choice = "1" }
    return $choice
}

function Start-Exe {
    $exePath = Join-Path $RootDir "JustLegalOS.exe"
    if (-not (Test-Path $exePath)) {
        $exePath = Join-Path $RootDir "OpenSeason.exe"
    }
    if (-not (Test-Path $exePath)) {
        $relExe = Join-Path $RootDir "src-tauri\target\release\open-season.exe"
        if (Test-Path $relExe) {
            Copy-Item $relExe (Join-Path $RootDir "JustLegalOS.exe") -Force
            Copy-Item $relExe (Join-Path $RootDir "OpenSeason.exe") -Force
            $exePath = Join-Path $RootDir "JustLegalOS.exe"
        } else {
            Write-Host "`n[WARN] JustLegalOS.exe not found. Building release binary first..." -ForegroundColor Yellow
            Start-Build
            $exePath = Join-Path $RootDir "JustLegalOS.exe"
        }
    }
    Write-Host "`n[INFO] Launching standalone $exePath..." -ForegroundColor Green
    Start-Process -FilePath $exePath
}

function Start-Dev {
    Write-Host "`n[INFO] Starting JustLegal OS in dev mode (Vite + Tauri)..." -ForegroundColor Cyan
    Write-Host "[INFO] Press Ctrl+C in this terminal when finished to stop.`n" -ForegroundColor DarkGray
    npm run tauri dev
}

function Run-Tests {
    Write-Host "`n[INFO] Running agent end-to-end test suite..." -ForegroundColor Cyan
    $bashPath = "C:\Program Files\Git\bin\bash.exe"
    if (Test-Path $bashPath) {
        & $bashPath "examples/agent-e2e.sh"
    } else {
        bash "examples/agent-e2e.sh"
    }
}

function Start-Build {
    Write-Host "`n[INFO] Building production desktop bundles with Tauri..." -ForegroundColor Cyan
    npm run tauri build -- --no-bundle
    if (Test-Path "src-tauri\target\release\open-season.exe") {
        Copy-Item "src-tauri\target\release\open-season.exe" "JustLegalOS.exe" -Force
        Copy-Item "src-tauri\target\release\open-season.exe" "OpenSeason.exe" -Force
        if (Test-Path "src-tauri\target\release\justlegalos-cli.exe") {
            Copy-Item "src-tauri\target\release\justlegalos-cli.exe" "JustLegalOS-CLI.exe" -Force
            Copy-Item "src-tauri\target\release\justlegalos-cli.exe" "justlegalos-cli.exe" -Force
            Copy-Item "src-tauri\target\release\justlegalos-cli.exe" "openseason-cli.exe" -Force
        } else {
            Copy-Item "src-tauri\target\release\openseason.exe" "JustLegalOS-CLI.exe" -Force
            Copy-Item "src-tauri\target\release\openseason.exe" "justlegalos-cli.exe" -Force
            Copy-Item "src-tauri\target\release\openseason.exe" "openseason-cli.exe" -Force
        }
        Write-Host "`n[SUCCESS] Production build complete: JustLegalOS.exe and JustLegalOS-CLI.exe ready." -ForegroundColor Green
    }
}

function Open-Folder([string]$subpath) {
    $target = Join-Path $env:USERPROFILE "Documents\JustLegal\$subpath"
    if (-not (Test-Path $target)) {
        New-Item -ItemType Directory -Path $target -Force | Out-Null
    }
    Write-Host "`n[INFO] Opening $subpath folder: $target" -ForegroundColor Green
    Start-Process explorer.exe $target
}

# Direct execution if Mode specified
switch ($Mode) {
    "run"   { Start-Exe; exit }
    "exe"   { Start-Exe; exit }
    "dev"   { Start-Dev; exit }
    "test"  { Run-Tests; exit }
    "build" { Start-Build; exit }
    "cases" { Open-Folder "Cases"; exit }
    "packs" { Open-Folder "Packs"; exit }
}

# Interactive loop
while ($true) {
    $sel = Show-Menu
    switch ($sel) {
        "1" { Start-Exe; break }
        "2" { Start-Dev; break }
        "3" { Run-Tests; Pause; break }
        "4" { Start-Build; Pause; break }
        "5" { Open-Folder "Cases"; break }
        "6" { Open-Folder "Packs"; break }
        "7" { exit }
        default { Write-Host "Invalid choice." -ForegroundColor Red; Start-Sleep -Seconds 1 }
    }
}
