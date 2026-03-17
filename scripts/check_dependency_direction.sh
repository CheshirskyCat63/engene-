#!/usr/bin/env bash
set -euo pipefail

ENGINE_DIRS=(src/core src/world src/memory src/physics src/audio src/graphics src/navigation src/content src/simulation src/animation src/input src/body src/network)

violations=""
for d in "${ENGINE_DIRS[@]}"; do
  if [[ -d "$d" ]]; then
    matches=$(rg -n "crate::game::|super::super::game::|use\s+crate::game" "$d" || true)
    if [[ -n "$matches" ]]; then
      violations+=$'\n'"$matches"
    fi
  fi
done

if [[ -n "$violations" ]]; then
  echo "Forbidden engine->game dependencies found:"
  echo "$violations"
  exit 1
fi

echo "Dependency direction gate passed (no engine->game imports)"
