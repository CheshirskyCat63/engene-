# Test Classification Policy

## Overview

This document defines the canonical classification system for all tests in the engene project. It ensures tests serve their intended purpose and prevents architectural drift.

## Classification Taxonomy

### 1. Semantic Laws (`*_semantic_laws.rs`)

**Purpose:** Verify fundamental semantic guarantees that must never change.

**What they test:**
- Meaning and purpose of system components
- Core architectural invariants
- System-level guarantees that transcend implementation details

**When they fail:** The system's fundamental purpose is broken.

**Examples:**
- "Streaming never creates invalid states"
- "Known chunks maintain idempotence"
- "System provides meaningful error handling"

**Promotion criteria:** Never promoted from other categories - must be designed as semantic laws from inception.

---

### 2. Resource Laws (`*_resource_laws.rs`)

**Purpose:** Verify resource boundary guarantees that must never change.

**What they test:**
- Budget compliance and caps
- Resource usage limits
- Memory/CPU/time boundaries
- Resource monotonicity properties

**When they fail:** Resource management is broken or unsafe.

**Examples:**
- "Budget is never exceeded"
- "Unload count is capped by pending count"
- "Resource usage is monotonic with budget increases"

**Promotion criteria:** Can be promoted from Implementation Shape when resource behavior proves to be fundamental.

---

### 3. Determinism Laws (`*_determinism_laws.rs`)

**Purpose:** Verify repeatability and temporal consistency guarantees.

**What they test:**
- Pure input-output determinism
- Temporal consistency (tick-independence where applicable)
- Input permutation invariance
- Floating-point stability
- Stress condition repeatability

**When they fail:** System behavior is unpredictable or non-repeatable.

**Examples:**
- "Same input always produces same output"
- "Tick number does not affect streaming decisions"
- "Known chunks order does not affect output"

**Promotion criteria:** Can be promoted from Implementation Shape when determinism proves to be fundamental.

---

### 4. Implementation Shape (`*_implementation_shape.rs`)

**Purpose:** Document current implementation specifics that MAY change.

**What they test:**
- Current traversal algorithms
- Specific prioritization strategies
- Exact output ordering
- Algorithmic choices and heuristics

**When they fail:** Implementation has changed (this is EXPECTED and OK).

**Examples:**
- "Current algorithm uses row-major traversal"
- "Budget truncation follows current order"
- "Unload algorithm prioritizes distant chunks"

**Promotion criteria:** Promote to Law categories ONLY when behavior proves to be fundamental architectural requirement.

---

### 5. Deterministic Smoke (`*_deterministic_smoke.rs`)

**Purpose:** Verify basic functionality and prevent catastrophic failures.

**What they test:**
- Basic entrypoint functionality
- Error handling robustness
- Minimal state impact verification
- Performance reasonableness checks

**When they fail:** System is fundamentally broken or unusable.

**Examples:**
- "Tick completes successfully"
- "Handles edge cases without crashing"
- "Duration is reasonable"

**Promotion criteria:** Can be promoted to Semantic Laws when functionality becomes sufficiently rich and fundamental.

---

## Classification Rules

### Rule 1: Single Responsibility
Each test file belongs to exactly ONE classification. No mixing.

### Rule 2: Honest Naming
File names MUST reflect their classification:
- `*_semantic_laws.rs`
- `*_resource_laws.rs` 
- `*_determinism_laws.rs`
- `*_implementation_shape.rs`
- `*_deterministic_smoke.rs`

### Rule 3: Clear Documentation
Each file MUST have header comments explaining:
- Its classification purpose
- What constitutes failure
- When tests should be promoted/demoted

### Rule 4: Implementation Independence

**Semantic/Resource/Determinism Laws:** Must pass regardless of implementation changes that preserve semantic meaning.

**Implementation Shape:** MAY fail when implementation changes (this is expected).

**Smoke Tests:** Must pass as long as basic functionality exists.

## Promotion/Demotion Criteria

### From Implementation Shape → Law

**Requirements:**
1. Behavior proves fundamental across multiple implementation attempts
2. Breaking the behavior would violate architectural principles
3. Behavior is documented in architectural specifications
4. Team consensus that this is a permanent requirement

**Process:**
1. Create proposal with justification
2. Architectural review
3. Team approval
4. Move test file and update all references

### From Law → Implementation Shape

**Requirements:**
1. Law was incorrectly classified as fundamental
2. Behavior is actually implementation-specific
3. Multiple valid implementations exist with different behaviors
4. Team consensus that this should not be enforced

**Process:**
1. Create proposal showing why law is too restrictive
2. Demonstrate valid alternative implementations
3. Architectural review
4. Team approval
5. Move test file and update all references

### From Smoke → Law

**Requirements:**
1. Smoke test covers functionality that becomes fundamental
2. Functionality has rich semantic meaning
3. Failure would indicate architectural violation
4. Team consensus that this is now a core guarantee

**Process:**
1. Enhance test with semantic assertions
2. Move to appropriate Law category
3. Update documentation
4. Team review

## Audit Requirements

### Monthly Audit
- Review all test classifications
- Identify misplaced tests
- Verify naming consistency
- Check for implementation dependencies in Laws

### Architecture Change Review
- Re-evaluate all Implementation Shape tests
- Promote any that became fundamental
- Demote any Laws that proved too restrictive

### New Test Review
- All new tests must have clear classification
- Review committee must approve classification
- Document justification for classification choice

## Enforcement

### Automated Checks
- CI validates file naming patterns
- Documentation completeness checks
- Classification consistency verification

### Manual Review
- Architectural review for Law changes
- Team review for classification disputes
- Monthly audit reports

## Examples of Proper Classification

### ✅ Correct: Semantic Law
```rust
// streaming_semantic_laws.rs
/// Semantic Law: Known chunks maintain idempotence
#[test] 
fn streaming_maintains_known_chunks_idempotence() {
    // Tests that known chunks are never reloaded
    // This is fundamental regardless of algorithm used
}
```

### ✅ Correct: Implementation Shape  
```rust
// streaming_implementation_shape.rs
/// Implementation Shape: Current algorithm uses row-major traversal
#[test]
fn streaming_uses_row_major_traversal_order() {
    // Tests current traversal order
    // EXPECTED to fail if traversal algorithm changes
}
```

### ❌ Incorrect: Mixed Classification
```rust
// WRONG: Tests implementation details in semantic law
fn streaming_semantic_law() {
    assert_eq!(output.chunks[0], [-2, -2]); // Implementation detail!
}
```

## Conclusion

This classification system ensures:
1. **Clarity:** Tests clearly communicate their purpose
2. **Stability:** Fundamental guarantees are protected
3. **Flexibility:** Implementation can evolve without breaking legitimate tests
4. **Honesty:** Test names match their actual intent

Following this policy prevents the common pitfall of "architectural tests" that actually just freeze current implementation details.
