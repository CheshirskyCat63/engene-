# ENGENE Legacy Recovery Lane — isolated broken integration tests
# Targets: all tests in tests_legacy/ directory
# Note: These tests are currently broken and isolated from the green gate

Write-Host "Running legacy recovery tests (may fail)..."
Push-Location tests_legacy
cargo test
Pop-Location