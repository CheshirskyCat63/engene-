#!/usr/bin/env bash
set -euo pipefail

cargo nextest run   --profile ci   --test certification_boundary_overhead   --test certification_kernel_throughput   --test certification_tick_budget   --test certification_perf_snapshot
