$ErrorActionPreference = "Stop"

cargo nextest run `
  --profile default `
  --test engine_contracts `
  --test production_candidate
