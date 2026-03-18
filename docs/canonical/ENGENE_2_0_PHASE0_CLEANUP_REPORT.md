# ENGENE 2.0 Phase 0 Cleanup Report

Date: 2026-03-18
Branch: `engene-2.0-transition`
Scope: Cleanup / Archaeology / Repository Sanitation only (no new feature work, no crate split).

## 1) Audit findings and classification

| Finding | Location | Classification | Rationale | Action in this batch |
|---|---|---|---|---|
| Legacy planning prompt document (non-canonical roadmap) | `ROADMAP.md` | QUARANTINE | Confusing parallel roadmap path; not canonical 2.0 law source | Moved to `legacy/quarantine/phase0/ROADMAP_LEGACY_PROMPT.md` |
| Root packaged binary committed into repo | `TEST.exe` | QUARANTINE | Binary artifact in source tree; false/legacy entrypoint risk | Moved to `legacy/quarantine/phase0/TEST.exe` |
| Legacy stderr artifact | `test_stderr.txt` | QUARANTINE | Non-source runtime artifact; no canonical ownership | Moved to `legacy/quarantine/phase0/test_stderr.txt` |
| Legacy office note file | `Фаза.docx` | QUARANTINE | Non-canonical handoff path + unclear ownership | Moved to `legacy/quarantine/phase0/phase_notes_legacy.docx` |
| Historical crash dumps in active root crash folder | `crashes/crash_*.ron` | QUARANTINE | Historical runtime artifacts should not clutter active root path | Moved to `legacy/quarantine/crashes_legacy/` and left `crashes/.gitkeep` |
| Doc generator references non-canonical roadmap path | `src/tools/doc_generator.rs` | REFACTOR_THEN_MOVE | False reference to root roadmap creates ambiguity | Updated link to `docs/canonical/ENGENE_2_0_ROADMAP.md` |
| Network module comment points to legacy roadmap path | `src/network/mod.rs` | REFACTOR_THEN_MOVE | Incorrect roadmap pointer for handoff/readability | Updated to canonical roadmap path |
| First-run guide prioritizes packaged `.exe` path and root artifact assumptions | `README_FIRST_RUN.md` | REFACTOR_THEN_MOVE | Conflicts with canonical entrypoint law; creates multiple perceived entrypoints | Rewritten around canonical `cargo run --bin ...` commands |
| Thin dispatch `src/main.rs` exists alongside canonical `src/bin/*` | `src/main.rs` | REFACTOR_THEN_MOVE | Potential false entrypoint confusion in future workspace split | Not changed in batch; scheduled for Phase A clarification |
| PowerShell launcher scripts (`run_game.ps1`, `run_sdk.ps1`) | repo root | CANONICAL_KEEP | Valid optional launch helpers; referenced and documented | Kept |
| Build/package scripts (`build_release.ps1`, `package_release.ps1`) | repo root | CANONICAL_KEEP | Required packaging path; still used in docs/workflow | Kept |
| Direction/ECS guard scripts | `scripts/*` | CANONICAL_KEEP | Phase-gate compliance checks used in canonical docs | Kept |
| Duplicate utility/contract type risk from monolithic layout | multiple `src/*` | REFACTOR_THEN_MOVE | Known Phase A concern; requires structural split planning, not safe Phase 0 deletion | Deferred with explicit tracking |
| Dead code certainty (provable) | broad `src/*` | CANONICAL_KEEP (for now) | No safe proof in this pass without deep semantic refactor; avoid risky deletions | Deferred |

## 2) Ordered cleanup plan (post first-safe batch)

1. **Entrypoint unification pass (pre-Phase A final):**
   - keep canonical `--bin` entrypoints only in docs;
   - mark/retire any duplicate root launch assumptions.
2. **Legacy quarantine indexing:**
   - add machine-readable manifest (`owner/status/target/removal_phase`) for quarantine tree.
3. **Ownership-readability stubs:**
   - add short owner README stubs per major top-level zone (`src/`, `game/`, `docs/`, `scripts/`).
4. **Script/launcher audit:**
   - classify each script as canonical helper vs legacy; retire ambiguity.
5. **Pre-split duplicate-contract audit:**
   - produce a targeted type/utility duplication inventory for Phase A moves.

## 3) First safe cleanup batch applied

### Applied changes
- Quarantined non-canonical root artifacts:
  - `ROADMAP.md` -> `legacy/quarantine/phase0/ROADMAP_LEGACY_PROMPT.md`
  - `TEST.exe` -> `legacy/quarantine/phase0/TEST.exe`
  - `test_stderr.txt` -> `legacy/quarantine/phase0/test_stderr.txt`
  - `Фаза.docx` -> `legacy/quarantine/phase0/phase_notes_legacy.docx`
- Quarantined old crash snapshots:
  - `crashes/crash_*.ron` -> `legacy/quarantine/crashes_legacy/`
  - added `crashes/.gitkeep` to preserve active crash directory contract.
- Corrected legacy references:
  - `src/tools/doc_generator.rs` roadmap link -> canonical 2.0 roadmap doc.
  - `src/network/mod.rs` roadmap note -> canonical 2.0 roadmap doc.
- Rewrote `README_FIRST_RUN.md` to canonical entrypoint law (`cargo run --bin ...`) and explicit optional packaging path.

### Safety statement
- No engine feature work added.
- No crate split performed.
- No runtime behavior intentionally changed beyond cleanup/readability and doc-link corrections.

## 4) Success condition check

Status: **PASS (Phase 0 first-safe batch)**
- Repository is cleaner (legacy artifacts quarantined from root hot path).
- Canonical entrypoint readability improved.
- Handoff ambiguity reduced (canonical roadmap and launch docs aligned).
- Product-boundary and no-feature-change constraints respected.
