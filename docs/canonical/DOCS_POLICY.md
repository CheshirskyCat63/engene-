# DOCS_POLICY

**Status**: Law
**Purpose**: Define what belongs where, and when documents can move.

---

## 1. Four-Layer Structure

```
docs/
├── canonical/     # действующие законы (laws)
├── target/        # целевая архитектура (future design)
├── transition/    # переходные заметки (migration notes)
└── reports/       # аудиты, батчи, исследования (audits/batches)
```

---

## 2. What Belongs in Canonical

**Canonical = law.** Only documents that define enforceable rules go here.

### Allowed in canonical

| Document type | Examples |
|---|---|
| Architecture laws | DEPENDENCY_LAW, ROOT_CRATE_POLICY |
| Execution contracts | PHASE_ORDER_CONTRACT, SPATIAL_DIRTY_CONTRACT |
| Invariants | RUNTIME_INVARIANTS |
| Ownership maps | RUNTIME_ROLE_MATRIX, WORKSPACE_OWNERSHIP_MAP |
| Test governance | TEST_LANE_MAP |
| Entrypoint truth | ENTRYPOINT_TRUTH |
| Current state reference | CURRENT_RUNTIME_TRUTH |
| Target architecture | ENGENE_2_0_GOD_TIER_ARCHITECTURE |
| Gap analysis | GAP_MAP_CURRENT_TO_GOD_TIER |

### Forbidden in canonical

- Aspirational documents that describe "what we hope to have"
- Batch reports from migration phases
- Audit findings that haven't become law
- Transition notes
- Any document that says "will be" instead of "is" or "must be"

---

## 3. What Belongs in Target

**Target = future design.** Documents that describe the destination but aren't yet law.

### Allowed in target

- Future architecture designs (before they become canonical)
- Feature policy proposals (before ratification)
- Ownership maps for future state (before enforcement)
- CI/performance roadmaps (before implementation)

### Example files in target/

```
docs/target/
├── 90_CI_GATES_AND_BENCH_HARNESS.md      # future CI gates
├── FEATURE_ROLE_POLICY.md                 # future feature taxonomy
├── RUNTIME_ROLE_MATRIX.md                 # future role model
└── WORKSPACE_OWNERSHIP_MAP.md             # future ownership
```

---

## 4. What Belongs in Transition

**Transition = migration notes.** Documents that track temporary states and removal plans.

### Allowed in transition

- Migration ledgers (what's being moved)
- Removal plans (what's being deleted)
- Temporary workarounds with sunset conditions
- Scaffold documentation

### Example files in transition/

```
docs/transition/
├── MIGRATION_LEDGER.md    # active migration items
└── REMOVAL_PLAN.md        # planned deletions
```

---

## 5. What Belongs in Reports

**Reports = audits, batch reports, research.** Documents that capture investigation results but aren't law.

### Allowed in reports

- Audit findings (before they become law)
- Phase batch reports (ENGENE_2_0_PHASEA_BATCH*.md)
- Status boards
- Active repo state snapshots

### Example files in reports/

```
docs/reports/
├── ACTIVE_REPO_STATE.md
├── DOC_STATUS_BOARD.md
└── (moved from archive/reports/*)
```

---

## 6. How to Move a Document

### From target to canonical

A document moves from target to canonical when:
1. It defines enforceable rules or contracts
2. It has been reviewed and approved as law
3. It doesn't contradict CURRENT_RUNTIME_TRUTH.md
4. Exit criteria from target are met

Process:
1. Move file from `docs/target/` to `docs/canonical/`
2. Update status in any index
3. Ensure no fantasy language ("will be", "should be") remains

### From transition to canonical

A document moves from transition to canonical when:
1. The temporary state becomes permanent law
2. Exit conditions are met
3. Document is rewritten from "temporary" to "enforceable"

### From reports to archive

A document moves to archive when:
1. Audit is complete and findings are addressed
2. Batch report is superseded by later phase
3. Report no longer provides actionable information

---

## 7. When a Report Becomes Law

An audit/report becomes law when:
1. Findings are converted to enforceable invariants
2. Rules are written in must/shall language
3. Document moves to canonical
4. Original report moves to archive

**Rule**: A report does not become law by being well-written. It becomes law by being converted to contract.

---

## 8. Document Status Types

| Status | Meaning | Location |
|---|---|---|
| canonical law | Enforceable rule | canonical/ |
| target design | Future architecture | target/ |
| transition note | Temporary state | transition/ |
| report | Audit/batch/research | reports/ |
| archive | Superseded/completed | archive/ |

---

## 9. Anti-Patterns

### Fantasy docs
Documents that describe "what we hope to have" as if it already exists.
**Rule**: Any doc in canonical must be true now. If it describes future state, it belongs in target.

### Zombie docs
Documents that were created for a phase but never moved or archived.
**Rule**: Every document must have a clear status and exit condition.

### Duplicate docs
Same document in multiple locations with slight variations.
**Rule**: One canonical copy. All others deleted or archived.

### Contradictory docs
Two documents that make incompatible claims about the same subject.
**Rule**: CURRENT_RUNTIME_TRUTH.md wins. Others must be corrected.

---

## 10. Mandatory Reference

Every document that describes runtime behavior must include:

```
Current runtime truth: see docs/canonical/CURRENT_RUNTIME_TRUTH.md
```

No doc may describe "how it works" without linking to CURRENT_RUNTIME_TRUTH.md.

---

## 11. Review Cycle

Documents should be reviewed:
- **On merge** — ensure no fantasy docs enter canonical
- **On phase completion** — move documents to appropriate layer
- **Quarterly** — audit for zombie docs and contradictions

---

## 12. Enforcement

| Violation | Action |
|---|---|
| Fantasy doc in canonical | Move to target or reject merge |
| Zombie doc | Archive or delete |
| Duplicate doc | Delete extra copies |
| Contradictory doc | Resolve per CURRENT_RUNTIME_TRUTH.md |
| Missing reference | Reject merge |

---

## 13. Relationship to Other Documents

This policy governs:
- All documents in docs/canonical/
- All documents in docs/target/
- All documents in docs/transition/
- All documents in docs/reports/

The three navigation documents form a system:
- **CURRENT_RUNTIME_TRUTH.md** — current reality (canonical)
- **GAP_MAP_CURRENT_TO_GOD_TIER.md** — blockers (canonical)
- **ENGENE_2_0_GOD_TIER_ARCHITECTURE.md** — target (canonical)

All must remain in canonical as the source of architectural truth.
