#!/usr/bin/env bash
set -euo pipefail

cargo nextest run   --profile default   --test engine_contracts   --test production_candidate
