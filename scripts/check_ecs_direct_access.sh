#!/usr/bin/env bash
set -euo pipefail

ROOT="${1:-src}"
ALLOWLIST="scripts/ecs_direct_access_allowlist.txt"
TMP_ALLOW="$(mktemp)"
trap 'rm -f "$TMP_ALLOW"' EXIT

grep -v -E '^\s*(#|$)' "$ALLOWLIST" > "$TMP_ALLOW"

violations=$(rg -n "ecs\.[a-z_]+\.(get|get_mut|insert|remove)" "$ROOT" | grep -v -F -f "$TMP_ALLOW" || true)

if [[ -n "$violations" ]]; then
  echo "Forbidden ECS direct-access patterns found:"
  echo "$violations"
  exit 1
fi

echo "ECS direct-access gate passed"
