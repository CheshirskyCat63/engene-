# FEATURE_ROLE_POLICY

## Rule

Every switch must be classified as exactly one of:
- `role_*`
- `cap_*`
- `tool_*`
- `profile_*`

No new mixed-purpose feature names may be introduced.

## Current target taxonomy

### Runtime roles
- `role_game`
- `role_sdk`
- `role_headless`
- `role_tools`

### Capabilities
- `cap_physics`
- `cap_render`
- `cap_ai`
- `cap_audio`
- `cap_networking`

### Tooling overlays
- `tool_debug_ui`
- `tool_sdk`
- `tool_doctor`
- `tool_editor_inspection`

### Profiles
- `profile_low_spec`
- `profile_ci`
- `profile_release`

## Transitional compatibility rule

Legacy aliases may remain temporarily:
- `physics`
- `render`
- `ai`
- `audio`
- `headless`
- `debug_ui`
- `full`
- `low_spec`
- `sdk_tools`

But they are compatibility aliases only, not the target semantic model.

## Hard rules

1. Capability is not runtime role.
2. Tool overlay is not engine identity.
3. Profile is not ownership.
4. Any alias kept for compatibility must be recorded in `MIGRATION_LEDGER.md` with a removal condition.
