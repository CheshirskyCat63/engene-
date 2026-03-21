# SDK_RUNNER_RS_OWNERSHIP_AUDIT

**File**: `crates/sdk_app/src/lib_complex.rs` (this IS the sdk_runner)
**Purpose**: Audit ownership of each block before extraction

---

## Block Analysis

### Block 1: Startup (lines ~80-100)

```
let heightmap = Arc::new(Heightmap::flat(...));
let engine = ToolsRuntimeAssembly::minimal();
let doctor_report = doctor::run_doctor(&engine, doctor::DoctorMode::Strict);
let mut editor_shell = EditorShell::new();
```

| Code | Current Owner | Target Owner | Status |
|------|---------------|--------------|--------|
| `BuildManifest::current()` | engine_startup | engine_startup | ✅ OK |
| `install_panic_hook()` | engine_startup | engine_startup | ✅ OK |
| `doctor::run_doctor()` | **QUESTION** | engine_tools or sdk_app::editor | ⚠️ UNRESOLVED |
| `ToolsRuntimeAssembly::minimal()` | engine_runtime | engine_runtime | ✅ OK |
| `EditorShell::new()` | sdk_app | sdk_app::editor | 🔄 Need extraction |

### Block 2: RedrawRequested - Tick (lines ~180-190)

```
if !self.sim_paused {
    self.sim_accum += (dt * self.sim_speed).min(0.25);
    while self.sim_accum >= SIM_DT {
        self.sim_accum -= SIM_DT;
        engene::app::sdk_runner::sdk_runner_phases::tick::run(self, SIM_DT);
    }
}
```

| Code | Current Owner | Target Owner | Status |
|------|---------------|--------------|--------|
| `tick::run()` | **engine_runtime** | engine_runtime::phase | 🔄 Phase extraction pending |

### Block 3: RedrawRequested - Asset Poll (lines ~195-200)

```
if let Some(am) = self.engine.resources.get_mut::<Mutex<AssetManager>>() {
    if let Ok(mut am) = am.lock() { am.poll(); }
}
```

| Code | Current Owner | Target Owner | Status |
|------|---------------|--------------|--------|
| `AssetManager::poll()` | engine_content | engine_content | ✅ OK |

### Block 4: RedrawRequested - Streaming (lines ~205-208)

```
let (to_load, to_unload) = engene::app::sdk_runner::sdk_runner_phases::streaming::run(self, cam_pos);
```

| Code | Current Owner | Target Owner | Status |
|------|---------------|--------------|--------|
| `streaming::run()` | **engine_world** | engine_world | 🔄 Phase extraction pending |

### Block 5: RedrawRequested - Persistence (lines ~210-212)

```
engene::app::sdk_runner::sdk_runner_phases::persistence::run(self, &to_load, &to_unload);
```

| Code | Current Owner | Target Owner | Status |
|------|---------------|--------------|--------|
| `persistence::run()` | **engine_world** | engine_world | 🔄 Phase extraction pending |

### Block 6: RedrawRequested - Audio (lines ~214-216)

```
engene::app::sdk_runner::sdk_runner_phases::audio::run(self, cam_pos, cam_fwd, dt);
```

| Code | Current Owner | Target Owner | Status |
|------|---------------|--------------|--------|
| `audio::run()` | **engine_audio** | engine_audio | 🔄 Phase extraction pending |

### Block 7: RedrawRequested - Editor (lines ~218-220)

```
engene::app::sdk_runner::sdk_runner_phases::editor::run(self);
```

| Code | Current Owner | Target Owner | Status |
|------|---------------|--------------|--------|
| `editor::run()` | **sdk_app** | sdk_app::editor | ✅ **FIRST EXTRACTION TARGET** |

### Block 8: RedrawRequested - Spatial (lines ~222-224)

```
engene::app::sdk_runner::sdk_runner_phases::spatial::run(self);
```

| Code | Current Owner | Target Owner | Status |
|------|---------------|--------------|--------|
| `spatial::run()` | **engine_world** | engine_world | 🔄 Phase extraction pending |

### Block 9: RedrawRequested - Render (lines ~226-228)

```
let vp = self.camera.view_projection();
engene::app::sdk_runner::sdk_runner_phases::render::run(self, vp, cam_pos, dt);
```

| Code | Current Owner | Target Owner | Status |
|------|---------------|--------------|--------|
| `render::run()` | **engine_render** | engine_render | 🔄 Phase extraction pending |

---

## Ownership Summary

| Owner | Blocks | Status |
|-------|--------|--------|
| **engine_startup** | BuildManifest, panic hook | ✅ OK |
| **engine_runtime** | tick | 🔄 Phase extraction |
| **engine_content** | AssetManager poll | ✅ OK |
| **engine_world** | streaming, persistence, spatial | 🔄 Phase extraction |
| **engine_audio** | audio | 🔄 Phase extraction |
| **engine_render** | render, camera | 🔄 Phase extraction |
| **sdk_app::editor** | editor shell | ✅ FIRST TARGET |
| **engine_tools** (question) | doctor | ⚠️ Need owner decision |

---

## First Extraction: Editor Slice

The cleanest first slice is `editor::run()` → `sdk_app::editor`:

**What it contains (in phase file):**
- `editor_shell.update_dashboards()` 
- `editor_shell.apply_inspector_edits()`

**Why it's clean:**
- Does NOT mutate world truth
- Does NOT control phase order
- Does NOT affect render truth
- Does NOT affect audio truth
- Only needed in editor/SDK mode
- Purely UI/state management

**What to leave in sdk_runner.rs after extraction:**
```rust
// OWNER: sdk_app::editor
sdk_app::editor::update_editor(&mut self.editor_shell, &mut self.engine);
```

---

## UNRESOLVED: Doctor Ownership

**Question**: Who should own `doctor::run_doctor()`?

| Candidate | Argument |
|-----------|----------|
| **engine_tools** | Doctor is a diagnostic tool, not editor UI |
| **sdk_app::editor** | Doctor runs at SDK startup, displays in editor |

**Current repo truth**: `engene::tools::doctor` module exists → suggests **engine_tools**

**Recommendation**: Leave doctor as-is until engine_tools ownership is clarified.

---

## Next Steps After First Extraction

1. Phase sequencing → `engine_runtime::phase::*`
2. Audio boundary → `engine_audio`
3. Render boundary → `engine_render`
4. World boundaries → `engine_world`
