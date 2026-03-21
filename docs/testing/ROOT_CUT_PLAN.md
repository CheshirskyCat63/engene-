# TARGETED ROOT CUT PLAN
## Phase 4: Migration Completion

**Status**: Ready for targeted surface removal
**Approach**: Group-by-group cutting with validation

---

## 🎯 **CURRENT ROOT SURFACE ANALYSIS**

### **✅ HEALTHY COMPONENTS**
- `src/lib.rs` - Thin shell (4 modules only)
- `src/core/mod.rs` - Compatibility hub pattern
- Test architecture - Fully migrated

### **🔄 PROBLEMATIC COMPONENTS**
- **Broken re-exports** in `src/core/mod.rs`:
  ```rust
  pub use engine_ecs::access;        // ✅ Working
  pub use engine_ecs::commands;      // ✅ Working  
  pub use engine_ecs::parallel_validation;  // ❌ Missing module
  pub use engine_ecs::system_descriptor;   // ❌ Missing module
  ```

---

## 📋 **CUT PLAN (Group-by-Group)**

### **GROUP 1: Broken Re-exports (IMMEDIATE)**
**Files**: `src/core/mod.rs`
**Action**: Remove or comment out broken re-exports
**Risk**: Low - these are already broken
**Validation**: `cargo check --workspace`

### **GROUP 2: Legacy Core Modules (MEDIUM)**
**Files**: `src/core/ai_memory.rs`, `src/core/content_validation.rs`, etc.
**Action**: Move to dedicated crates or remove
**Risk**: Medium - may have dependencies
**Validation**: Full test suite

### **GROUP 3: Compatibility Layer (HIGH)**
**Files**: `src/core/mod.rs` entire file
**Action**: Remove after all dependencies resolved
**Risk**: High - affects many imports
**Validation**: Complete workspace validation

---

## 🔄 **CUT SEQUENCE**

### **CUT 1: Broken Re-exports**
```rust
// REMOVE these broken lines:
pub use engine_ecs::parallel_validation;
pub use engine_ecs::system_descriptor;
pub use engine_ecs::persistent_id;
```

### **CUT 2: Legacy Modules**
```bash
# MOVE these to appropriate crates:
src/core/ai_memory.rs → engine_ai/
src/core/content_validation.rs → engine_content/
src/core/material_truth.rs → engine_content/
```

### **CUT 3: Compatibility Hub**
```bash
# EVENTUALLY remove:
src/core/mod.rs → delete entire file
src/core/ → remove directory
```

---

## ✅ **VALIDATION CHECKPOINTS**

### **After Cut 1**: 
- [ ] `cargo check --workspace` passes
- [ ] No broken re-export errors
- [ ] Test suites compile

### **After Cut 2**:
- [ ] All modules moved successfully
- [ ] No missing module errors
- [ ] Full test suite passes

### **After Cut 3**:
- [ ] Root surface minimal
- [ ] Direct crate imports only
- [ ] Complete validation passes

---

## 🎯 **SUCCESS CRITERIA**

**Root surface is "cut" when:**
- ✅ No more compatibility layers
- ✅ Direct imports from engine_* crates only
- ✅ `src/lib.rs` contains only essential modules
- ✅ All tests pass with direct imports
- ✅ No legacy path dependencies

---

## 📊 **CURRENT STATUS**

**Ready to execute**: Cut 1 (Broken Re-exports)
**Blocking issues**: Missing engine_* modules
**Estimated effort**: 2-3 cuts total
**Risk level**: Medium (manageable)
