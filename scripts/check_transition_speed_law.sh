#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

BENCH_FILE="benches/simulation_transition_core.rs"
if [[ ! -f "$BENCH_FILE" ]]; then
  echo "Missing benchmark file: $BENCH_FILE" >&2
  exit 1
fi

declare -A COUNTS=(
  ["simulation_core/classify_only_single_thread"]=131072
  ["simulation_core/classify_only_multi_thread_x8"]=131072
  ["simulation_core/classify_materialize_single_thread"]=131072
  ["simulation_core/classify_materialize_multi_thread_x8"]=131072
  ["simulation_core/classify_order"]=16384
  ["simulation_core/classify_order_resolve"]=16384
  ["simulation_core/deferred_queue_replay"]=1024
  ["simulation_core/deterministic_merge_prep"]=131072
)

declare -A MIN_TPS=(
  ["simulation_core/classify_only_single_thread"]=800000
  ["simulation_core/classify_only_multi_thread_x8"]=2000000
  ["simulation_core/classify_materialize_single_thread"]=500000
  ["simulation_core/classify_materialize_multi_thread_x8"]=1200000
  ["simulation_core/classify_order"]=500000
  ["simulation_core/classify_order_resolve"]=500000
  ["simulation_core/deferred_queue_replay"]=500000
  ["simulation_core/deterministic_merge_prep"]=500000
)

declare -A MAX_MS=(
  ["simulation_core/classify_only_single_thread"]=0.20
  ["simulation_core/classify_only_multi_thread_x8"]=0.50
  ["simulation_core/classify_materialize_single_thread"]=0.35
  ["simulation_core/classify_materialize_multi_thread_x8"]=0.50
  ["simulation_core/classify_order"]=0.50
  ["simulation_core/classify_order_resolve"]=0.50
  ["simulation_core/deferred_queue_replay"]=0.50
  ["simulation_core/deterministic_merge_prep"]=0.50
)

CLASSIFY_ONLY_SCALING_MIN_X8=3.0
CLASSIFY_ONLY_SCALING_TARGET_X8=4.5
CLASSIFY_MATERIALIZE_SCALING_MIN_X8=3.0
CLASSIFY_MATERIALIZE_SCALING_TARGET_X8=4.5

run_bench() {
  cargo bench --bench simulation_transition_core -- --noplot --warm-up-time 0.10 --measurement-time 0.20 --sample-size 10 >/tmp/transition_core_bench.log
}

mean_from_json() {
  local json_path="$1"
  python - <<PY
import json
with open('$json_path','r',encoding='utf-8') as f:
    data=json.load(f)
print(data['mean']['point_estimate'])
PY
}

compute_metrics() {
  local mean_ns="$1"
  local count="$2"
  python - <<PY
mean_ns=float('$mean_ns')
count=float('$count')
mean_ms=mean_ns/1_000_000.0
tps=count*1_000_000_000.0/mean_ns
print(f"{mean_ms:.6f} {tps:.2f}")
PY
}

scaling_ratio() {
  local single_json="$1"
  local multi_json="$2"
  local single_ns multi_ns
  single_ns="$(mean_from_json "$single_json")"
  multi_ns="$(mean_from_json "$multi_json")"
  python - <<PY
single=float('$single_ns')
multi=float('$multi_ns')
print(f"{single/multi:.3f}")
PY
}

latency_status() {
  local mean_ms="$1"
  local max_ms="$2"
  python - <<PY
ratio=float('$mean_ms')/float('$max_ms')
if ratio <= 1.0:
    print('PASS')
elif ratio <= 1.10:
    print('NEAR_MISS')
elif ratio <= 1.50:
    print('MATERIAL_REGRESSION')
else:
    print('CRITICAL_FAILURE')
PY
}

scaling_status() {
  local ratio="$1"
  local required="$2"
  python - <<PY
ratio=float('$ratio')
required=float('$required')
if ratio >= required:
    print('PASS')
elif ratio >= required*0.85:
    print('NEAR_MISS')
elif ratio >= 1.0:
    print('MATERIAL_REGRESSION')
else:
    print('CRITICAL_FAILURE')
PY
}

gate_scaling_pair() {
  local label="$1"
  local single_surface="$2"
  local multi_surface="$3"
  local min_required="$4"
  local target="$5"
  local -n fail_ref="$6"

  local single_key="${single_surface//\//_}"
  local multi_key="${multi_surface//\//_}"
  local st_json="target/criterion/${single_key}/new/estimates.json"
  local mt_json="target/criterion/${multi_key}/new/estimates.json"

  if [[ ! -f "$st_json" || ! -f "$mt_json" ]]; then
    echo "CRITICAL_FAILURE: missing estimates for ${label} scaling pair (${single_surface}, ${multi_surface})" >&2
    fail_ref=1
    return
  fi

  local ratio status
  ratio="$(scaling_ratio "$st_json" "$mt_json")"
  status="$(scaling_status "$ratio" "$min_required")"

  if [[ "$status" == "PASS" ]]; then
    echo "PASS: ${label}_x8 scaling ratio=${ratio} (min ${min_required}, target ${target})"
  elif [[ "$status" == "CRITICAL_FAILURE" ]]; then
    local mt_slower
    mt_slower=$(python - <<PY
print(f"{1.0/float('$ratio'):.2f}")
PY
)
    echo "CRITICAL_FAILURE: ${label}_x8 scaling ratio=${ratio} (min ${min_required}, target ${target}) MT is ${mt_slower}x slower than ST" >&2
    fail_ref=1
  else
    echo "${status}: ${label}_x8 scaling ratio=${ratio} (min ${min_required}, target ${target})" >&2
    fail_ref=1
  fi
}

gate_from_existing() {
  local fail=0
  local missing=()

  for bench in "${!COUNTS[@]}"; do
    local criterion_key="${bench//\//_}"
    local json_path="target/criterion/${criterion_key}/new/estimates.json"
    if [[ ! -f "$json_path" ]]; then
      missing+=("$json_path")
    fi
  done

  if [[ "${#missing[@]}" -ne 0 ]]; then
    echo "CRITICAL_FAILURE: --from-existing requires precomputed criterion estimates; missing ${#missing[@]} file(s)." >&2
    for path in "${missing[@]}"; do
      echo "  - $path" >&2
    done
    echo "Run without --from-existing to regenerate estimates, then rerun --from-existing." >&2
    exit 1
  fi

  for bench in "${!COUNTS[@]}"; do
    local criterion_key="${bench//\//_}"
    local json_path="target/criterion/${criterion_key}/new/estimates.json"

    local mean_ns mean_ms tps
    mean_ns="$(mean_from_json "$json_path")"
    read -r mean_ms tps <<<"$(compute_metrics "$mean_ns" "${COUNTS[$bench]}")"

    local tps_ok latency_state ratio
    tps_ok=$(python - <<PY
print(1 if float('$tps') >= float('${MIN_TPS[$bench]}') else 0)
PY
)
    latency_state="$(latency_status "$mean_ms" "${MAX_MS[$bench]}")"
    ratio=$(python - <<PY
print(f"{float('$mean_ms')/float('${MAX_MS[$bench]}'):.3f}")
PY
)

    if [[ "$latency_state" == "PASS" && "$tps_ok" == "1" ]]; then
      echo "PASS: $bench mean=${mean_ms}ms throughput=${tps}/s"
    else
      local severity="$latency_state"
      if [[ "$tps_ok" != "1" && "$latency_state" == "PASS" ]]; then
        severity="CRITICAL_FAILURE"
      fi
      echo "${severity}: $bench mean=${mean_ms}ms (max ${MAX_MS[$bench]}ms, ratio=${ratio}x) throughput=${tps}/s (min ${MIN_TPS[$bench]}/s)" >&2
      fail=1
    fi
  done

  local available_cpus
  if command -v nproc >/dev/null 2>&1; then
    available_cpus="$(nproc)"
  else
    available_cpus="$(getconf _NPROCESSORS_ONLN)"
  fi

  if [[ "$available_cpus" -lt 8 ]]; then
    echo "CRITICAL_FAILURE: environment invalid for x8 scaling proof (available_cpus=${available_cpus}, required>=8)" >&2
    echo "transition-core speed law regression gate failed" >&2
    exit 1
  fi

  gate_scaling_pair \
    "classify_only" \
    "simulation_core/classify_only_single_thread" \
    "simulation_core/classify_only_multi_thread_x8" \
    "$CLASSIFY_ONLY_SCALING_MIN_X8" \
    "$CLASSIFY_ONLY_SCALING_TARGET_X8" \
    fail

  gate_scaling_pair \
    "classify_materialize" \
    "simulation_core/classify_materialize_single_thread" \
    "simulation_core/classify_materialize_multi_thread_x8" \
    "$CLASSIFY_MATERIALIZE_SCALING_MIN_X8" \
    "$CLASSIFY_MATERIALIZE_SCALING_TARGET_X8" \
    fail

  if [[ "$fail" -ne 0 ]]; then
    echo "transition-core speed law regression gate failed" >&2
    exit 1
  fi
  echo "transition-core speed law regression gate passed"
}

proof_isolation() {
  local mode="$1"
  local log_file="/tmp/transition_core_${mode}_build.log"

  if [[ "$mode" == "cold" ]]; then
    cargo clean
  fi

  cargo rustc --bench simulation_transition_core --profile bench -vv -- -C debuginfo=0 >"$log_file" 2>&1

  local bin_hits
  if command -v rg >/dev/null 2>&1; then
    bin_hits=$( (rg -o "src/bin/engene_(game|sdk|headless|test)\.rs" "$log_file" || true) | wc -l | tr -d " " )
  elif command -v grep >/dev/null 2>&1; then
    bin_hits=$( (grep -Eo "src/bin/engene_(game|sdk|headless|test)\.rs" "$log_file" || true) | wc -l | tr -d " " )
  else
    echo "CRITICAL_FAILURE: ${mode}-build isolation proof requires rg or grep to scan build log" >&2
    exit 1
  fi

  if [[ "$bin_hits" == "0" ]]; then
    echo "PASS: ${mode}-build isolation proof (no root runtime bins compiled by isolated bench rustc path)"
  else
    echo "CRITICAL_FAILURE: ${mode}-build isolation proof detected root runtime bin compile hits=${bin_hits}" >&2
    exit 1
  fi
}

case "${1:-}" in
  --from-existing)
    gate_from_existing
    ;;
  --proof-cold-build)
    proof_isolation cold
    ;;
  --proof-warm-build)
    proof_isolation warm
    ;;
  "")
    echo "Running transition-core benchmark proof pass..."
    run_bench
    gate_from_existing
    ;;
  *)
    echo "Unknown arg: $1" >&2
    echo "Usage: $0 [--from-existing|--proof-cold-build|--proof-warm-build]" >&2
    exit 2
    ;;
esac
