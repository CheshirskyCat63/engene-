#!/bin/bash
# ENGENE Legacy Recovery Lane — isolated broken integration tests
# Targets: all tests in tests_legacy/ directory
# Note: These tests are currently broken and isolated from the green gate

set -e
echo "Running legacy recovery tests (may fail)..."
cd tests_legacy
cargo test