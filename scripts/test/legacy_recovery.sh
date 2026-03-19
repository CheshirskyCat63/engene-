#!/usr/bin/env bash
set -euo pipefail

echo "Legacy recovery suite is isolated from default platform gate."
echo "Recovery targets:"
echo "  - tests_legacy/world_streaming.rs"
echo "  - tests_legacy/physics_body_combat/"
echo "  - tests_legacy/content_pipeline.rs"
echo "  - tests_legacy/persistence_full.rs"
echo "  - tests_legacy/runtime_systems/"
echo "  - tests_legacy/gameplay_and_ai.rs"
echo
echo "Run recovery manually after API drift repair."