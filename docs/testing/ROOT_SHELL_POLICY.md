# ROOT SHELL POLICY

## DECISION: MODEL A - Compatibility Façade

**Chosen Model**: Root shell = compatibility façade with full re-exports

## Rationale

1. **Backward Compatibility**: Existing consumers expect root re-exports
2. **Migration Path**: Gradual transition to direct crate imports
3. **Workspace Convenience**: Single entry point for common functionality
4. **Minimal Overhead**: Re-exports are compile-time only

## Implementation

### src/lib.rs
- Re-export all canonical crates
- Provide clear documentation
- Mark as transitional where appropriate

### Cargo.toml
- Declare all re-exported crates as dependencies
- Ensure version consistency
- Enable feature flags where needed

### Migration Path
- Phase 1: Current compatibility façade
- Phase 2: Deprecation warnings for direct imports
- Phase 3: Minimal root shell (future consideration)

## Governance

- **Do NOT remove re-exports** without deprecation cycle
- **DO maintain dependency declarations** in sync with re-exports
- **DO document transitional status** clearly
- **DO NOT break existing consumers** without migration path

---
**Policy Fixed: Compatibility Façade Model A**
