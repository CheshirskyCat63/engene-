$ErrorActionPreference = "Stop"

cargo nextest run `
  --profile default `
  --test engine_contracts `
  --test determinism_and_sdk `
  --test runtime_systems
