#!/usr/bin/env pwsh
# ENGENE Certification Lane — performance boundary validation
# Targets: certification_boundary_overhead, certification_kernel_throughput, certification_tick_budget, certification_perf_snapshot

cargo nextest run --profile ci --test certification_boundary_overhead --test certification_kernel_throughput --test certification_tick_budget --test certification_perf_snapshot
