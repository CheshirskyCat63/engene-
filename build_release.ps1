# build_release.ps1 — Build all three ENGENE executables in release mode.
param(
    [string]$Profile = "release"
)

$ErrorActionPreference = "Stop"

Write-Host "=== ENGENE Build Release ===" -ForegroundColor Cyan
Write-Host "Profile: $Profile"
Write-Host ""

$targets = @("engene_game", "engene_sdk", "engene_run")

foreach ($bin in $targets) {
    Write-Host "Building $bin ($Profile)..." -ForegroundColor Yellow
    cargo build --bin $bin --profile $Profile
    if ($LASTEXITCODE -ne 0) {
        Write-Host "FAILED: $bin" -ForegroundColor Red
        exit 1
    }
    Write-Host "  OK" -ForegroundColor Green
}

Write-Host ""
Write-Host "All binaries built successfully." -ForegroundColor Green
Write-Host "Run .\package_release.ps1 to package into dist/release/"
