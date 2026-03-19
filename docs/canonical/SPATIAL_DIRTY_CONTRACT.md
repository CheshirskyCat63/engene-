# SPATIAL_DIRTY_CONTRACT

## Purpose

The spatial system must stop behaving like “clear everything and hope coffee helps”.

## Dirty input model

A spatial update request must be representable as explicit dirty input.

### Dirty causes
- entity inserted
- entity moved
- entity removed
- chunk loaded
- chunk unloaded
- editor mutation changed transform
- origin shift / rebuild trigger
- recovery / safety rebuild request

## Update paths

### Incremental path
Use when dirty input is explicit and bounded.

Expected operations:
- insert new entries,
- update moved entries,
- remove unloaded/despawned entries,
- patch affected chunks/cells only.

### Full rebuild fallback
Use when:
- dirty source is incomplete,
- origin shift invalidates broad assumptions,
- recovery mode is requested,
- debug/verification path explicitly asks for rebuild.

## Law

Fallback is allowed.
Fallback without observability is not allowed.

## Required observability

Every spatial update should be classifiable as one of:

- `incremental`
- `full_rebuild`
- `no_op`

Recommended telemetry:
- dirty entity count,
- dirty chunk count,
- reason,
- duration,
- fallback reason if any.

## Equivalence requirement

Incremental path must be validated against full rebuild semantics for the same world state.
That means property checks of the form:

`incremental(world, dirty_input) == rebuild(world_after_changes)`

## Required tests

- inserted / moved / removed equivalence
- chunk load / unload invalidation
- editor mutation propagation
- origin shift rebuild trigger
- fallback path observability

## Current branch truth

- SDK runtime uses explicit path selection: `no_op`, `incremental`, `full_rebuild`.
- Default redraw path is no longer unconditional full rebuild.
- Full rebuild fallback still exists and is used for structural invalidation.
