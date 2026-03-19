# FEATURE_ROLE_POLICY

## Problem

Current feature language mixes unrelated kinds of switches:

- capability: `physics`, `render`, `ai`, `audio`, `networking`
- runtime mode: `headless`
- tooling overlay: `debug_ui`, `sdk_tools`
- quality profile: `low_spec`
- bundle alias: `full`

This is ambiguous and causes architectural drift.

## Required separation

### 1. Runtime role selectors
Examples:
- `role_game`
- `role_sdk`
- `role_headless`
- `role_tools`

### 2. Capabilities
Examples:
- `cap_render`
- `cap_physics`
- `cap_ai`
- `cap_audio`
- `cap_networking`

### 3. Tooling overlays
Examples:
- `tool_debug_ui`
- `tool_editor_inspection`
- `tool_doctor`

### 4. Profiles
Examples:
- `profile_low_spec`
- `profile_ci`
- `profile_release`

## Policy

- A role may enable multiple capabilities.
- A capability may not define the runtime role by itself.
- Tooling overlays must never be described as core engine identity.
- A profile may tune behavior, but must not redefine ownership.

## Current practical rule

Before renaming features in Cargo, use this document as the semantic policy:
every new switch or configuration option must be classified as one of the four categories above.
If classification is unclear, it does not get added yet.
