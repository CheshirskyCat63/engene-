# TEST GOVERNANCE RULES

## Mock Types Classification

### TYPE A: Test Seams (ALLOWED)
**Definition**: Temporary test-only constructs that explicitly do not replace canonical API.
**Allowed when**:
- Clearly marked as test-only
- Do not have same name as canonical types
- Used only for testing internal logic
- Have TODO to replace with real API

**Examples**:
```rust
// ✅ ALLOWED - Test-only helper
#[cfg(test)]
struct TestHelper {
    data: Vec<u8>,
}

// ✅ ALLOWED - Explicit test seam
#[cfg(test)]
struct MockChunkLoader {
    // TODO: Replace with engine_world::ChunkLoader
}
```

### TYPE B: Production Duplicates (FORBIDDEN)
**Definition**: Local types that pretend to be real engine API but are fake implementations.
**Forbidden when**:
- Have same name as canonical types
- Replace real API in tests
- Create alternative production surface
- Not marked as test-only

**Examples**:
```rust
// ❌ FORBIDDEN - Fake production type
struct Transform {
    position: [f32; 3],
    rotation: [f32; 3],
    scale: [f32; 3],
}

// ❌ FORBIDDEN - Mock pretending to be real
struct EventBus {
    // Fake implementation that pretends to be real
}
```

### TYPE C: Mock Stubs (ALLOWED)
**Definition**: Placeholder checks for future functionality.
**Allowed when**:
- Clearly stubs for unimplemented features
- Minimal implementation for validation
- Marked as placeholder/temporary

**Examples**:
```rust
// ✅ ALLOWED - Placeholder stub
#[cfg(test)]
fn future_feature_placeholder() {
    // TODO: Implement when feature is ready
    assert!(true); // Placeholder validation
}
```

## Governance Rules

1. **No TYPE B mocks allowed** - All production duplicates must be removed
2. **TYPE A seams must be marked** - Use `#[cfg(test)]` and clear comments
3. **TYPE C stubs must be tracked** - Document in TODO comments
4. **Canonical imports only** - Use real types from engine_* crates when available
5. **Mock cleanup is ongoing** - Each mock must have migration path

## Migration Priority

1. **High**: Remove all TYPE B mocks immediately
2. **Medium**: Mark all TYPE A seams clearly
3. **Low**: Track TYPE C stubs for future implementation

## Validation

Meta-tests will enforce:
- `no_local_production_mocks` - No TYPE B mocks
- `test_seams_are_marked` - TYPE A seams are clearly marked
- `canonical_imports_used` - Real types when available
