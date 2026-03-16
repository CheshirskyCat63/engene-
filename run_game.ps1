# run_game.ps1 — Launch ENGENE Game.
param(
    [switch]$Build,
    [string]$Profile = "release"
)

$ErrorActionPreference = "Stop"

if ($Build) {
    Write-Host "Building engene_game ($Profile)..." -ForegroundColor Yellow
    cargo build --bin engene_game --profile $Profile
    if ($LASTEXITCODE -ne 0) { exit 1 }
}

$exe = "ENGENE_Game.exe"
if (Test-Path $exe) {
    Write-Host "Launching $exe..." -ForegroundColor Cyan
    & ".\$exe"
} elseif (Test-Path "dist/release/$exe") {
    Write-Host "Launching dist/release/$exe..." -ForegroundColor Cyan
    & ".\dist\release\$exe"
} else {
    Write-Host "Running via cargo..." -ForegroundColor Yellow
    cargo run --bin engene_game
}
