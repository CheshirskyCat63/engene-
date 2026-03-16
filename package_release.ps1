# package_release.ps1 — Package built executables into dist/release/ and project root.
param(
    [string]$Profile = "release"
)

$ErrorActionPreference = "Stop"

Write-Host "=== ENGENE Package Release ===" -ForegroundColor Cyan

$targetDir = "target/$Profile"
$distDir   = "dist/release"

New-Item -ItemType Directory -Force -Path $distDir | Out-Null

$binaries = @{
    "engene_game.exe"     = "ENGENE_Game.exe"
    "engene_sdk.exe"      = "ENGENE_SDK.exe"
    "engene_headless.exe" = "ENGENE_Headless.exe"
    "engene_test.exe"     = "TEST.exe"
}

foreach ($src in $binaries.Keys) {
    $srcPath  = Join-Path $targetDir $src
    $dstName  = $binaries[$src]
    $distPath = Join-Path $distDir $dstName
    $rootPath = $dstName

    if (-not (Test-Path $srcPath)) {
        Write-Host "NOT FOUND: $srcPath — run build_release.ps1 first" -ForegroundColor Red
        exit 1
    }

    Copy-Item $srcPath $distPath -Force
    Copy-Item $srcPath $rootPath -Force
    $size = (Get-Item $srcPath).Length / 1MB
    Write-Host "  $dstName -> dist/release/ + root  ({0:N1} MB)" -f $size -ForegroundColor Green
}

Write-Host ""
Write-Host "Packaged files:" -ForegroundColor Cyan
Get-ChildItem $distDir | ForEach-Object {
    Write-Host "  $($_.Name)  ($([math]::Round($_.Length / 1MB, 1)) MB)"
}

Write-Host ""
Write-Host "Done. Executables ready in $distDir and project root." -ForegroundColor Green
