# run_sdk.ps1 — Launch ENGENE SDK editor.
param(
    [switch]$Build,
    [string]$Profile = "release"
)

$ErrorActionPreference = "Stop"

if ($Build) {
    Write-Host "Building engene_sdk ($Profile)..." -ForegroundColor Yellow
    cargo build --bin engene_sdk --profile $Profile --features sdk_tools
    if ($LASTEXITCODE -ne 0) { exit 1 }
}

$exe = "ENGENE_SDK.exe"
if (Test-Path $exe) {
    Write-Host "Launching $exe..." -ForegroundColor Cyan
    & ".\$exe"
} elseif (Test-Path "dist/release/$exe") {
    Write-Host "Launching dist/release/$exe..." -ForegroundColor Cyan
    & ".\dist\release\$exe"
} else {
    Write-Host "Running via cargo..." -ForegroundColor Yellow
    cargo run --bin engene_sdk --features sdk_tools
}
