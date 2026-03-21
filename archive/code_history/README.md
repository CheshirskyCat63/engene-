# Code History Archive

## Purpose

This directory contains archived code from pre-engine-* architecture era.

## Archive Structure

```
archive/
├── code_history/
│   ├── pre_engine_crate_migration/  # Before engine_* split
│   ├── monolithic_src/             # Old src/ structure
│   └── deprecated_bins/           # Old engene_* binaries
├── tests_legacy_pre_2024/           # Historical test package
└── docs_history/                  # Historical documentation
```

## Migration Status

- ✅ **Monolithic src/** → crates/engine_** migration completed**
- ✅ **Legacy binaries archived** (engene_headless, engene_bootstrap, etc.)
- ✅ **Tests separated** into legacy package
- ✅ **Documentation consolidated** into canonical/

## Access Policy

This is **read-only archive**. Code here should:

1. Never be directly referenced from active code
2. Serve only as historical reference
3. Not be modified unless for archival purposes
4. Be considered "frozen" - use git history for active development

## Notes

- All active development happens in crates/engine_*/
- All active tests happen in tests/ (not archive/)
- This represents the "before" state for current engine architecture
