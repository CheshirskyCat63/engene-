# ENGENE v1.0 Scope Lock

> **Статус:** ЗАФИКСИРОВАН
> **Дата:** 2025-01-21
> **Версия документа:** 1.0

---

## 1. Определение версии 1.0

**ENGENE 1.0** — это **single-player production-ready игровой движок** для survival-симулятора с открытым миром.

---

## 2. Scope 1.0 — ВКЛЮЧЕНО

### 2.1 Core Engine
- ✅ ECS (Entity Component System) на SparseSet
- ✅ Event Bus (типизированная, фазная)
- ✅ Job System (WorkerPool, FrameGraph)
- ✅ Resources Registry
- ✅ Command Buffer
- ✅ Determinism infrastructure
- ✅ Crash telemetry

### 2.2 Gameplay Systems
- ✅ AI (plan-based, не BT) для NPC и монстров
- ✅ Physics (Rapier3D + custom ballistics)
- ✅ Economy system
- ✅ Ecosystem simulation
- ✅ Body damage system
- ✅ Fire/Water/Cloth simulation

### 2.3 World
- ✅ 2×2 км мир (40×40 ячеек)
- ✅ Chunk streaming
- ✅ LOD simulation (L0/L1/L2)
- ✅ Surface system с разрушениями

### 2.4 Rendering
- ✅ Forward PBR rendering (wgpu)
- ✅ Shadow maps (cascaded)
- ✅ Sky/Weather system
- ✅ Particles
- ✅ Vegetation

### 2.5 Tools
- ✅ SDK Editor (egui-based)
- ✅ Doctor diagnostics
- ✅ Profiler dashboard

### 2.6 Data-Driven Architecture
- ✅ 16+ .ron config files
- ✅ Prefab system
- ✅ Content validation

### 2.7 Persistence
- ✅ Save/Load chunks
- ✅ Entity persistence
- ✅ Schema versioning

### 2.8 Release Engineering
- ✅ Build scripts
- ✅ Package scripts
- ✅ Version stamping

---

## 3. Scope 1.0 — ИСКЛЮЧЕНО (out of scope)

### 3.1 Networking
- ❌ Multiplayer
- ❌ Dedicated server
- ❌ P2P
- **Статус:** Планируется в v1.1+

### 3.2 Advanced Animation
- ❌ Full ragdoll physics
- ❌ Advanced IK
- ❌ Motion matching
- **Статус:** Частично реализовано, не production-ready

### 3.3 Deferred Rendering
- ❌ Deferred shading pipeline
- ❌ SSAO
- **Статус:** Forward-only для 1.0

### 3.4 Parallel Tick
- ❌ Многопоточный tick по умолчанию
- **Статус:** Sequential shipping default. Parallel = experimental feature.
- **Причина:** Требует дополнительного тестирования на determinism

### 3.5 Content Pipeline (Cook)
- ❌ Full production cook pipeline
- **Статус:** Authored/static content only для 1.0
- **Причина:** Cook phase — partial implementation

### 3.6 GPU Compute Jobs
- ❌ GPU-driven culling
- ❌ GPU particle simulation
- **Статус:** Планируется в v1.1+

---

## 4. Shipping Defaults

| Параметр | Значение | Обоснование |
|----------|----------|-------------|
| Tick mode | **Sequential** | Determinism guarantee |
| Render mode | **Forward PBR** | Простота, стабильность |
| Max entities | 1000 | Single-player scope |
| World size | 2×2 км | Single-player scope |
| Max clients | 1 | Single-player only |
| Save format | Binary (bincode) | Performance |

---

## 5. Качественные критерии 1.0

### 5.1 Stability
- [ ] 0 direct ECS storage access в runtime systems
- [ ] 0 silent config failures (strict mode)
- [ ] 0 silent persistence failures
- [ ] 0 unwrap/expect в критических путях

### 5.2 Data-Driven
- [ ] Все required configs wired
- [ ] Config values = runtime values (no hardcoded duplicates)
- [ ] Doctor использует canonical config list

### 5.3 Release Readiness
- [ ] CI gates implemented
- [ ] Package buildable без шаманства
- [ ] First-run validation passed
- [ ] Symbol archive produced

### 5.4 Documentation
- [ ] Roadmap/doc/doctor говорят одно и то же
- [ ] RELEASE_BLOCKERS.md актуален
- [ ] CONFIG_WIRING_MATRIX.md существует

---

## 6. Documented Exceptions (P1)

Следующие элементы могут быть выпущены с documented exception:

| ID | Элемент | Статус | Mitigation |
|----|---------|--------|------------|
| EX-01 | Parallel tick | Experimental | Sequential default documented |
| EX-02 | Metrics wiring | Partial | Baseline observability works |
| EX-03 | Content cook | Partial | Authored content path works |
| EX-04 | GPU jobs | Stub | CPU fallback works |

---

## 7. Изменение Scope Lock

Scope Lock может быть изменён только:
1. Явным решением lead architect
2. С обновлением этого документа
3. С обновлением RELEASE_BLOCKERS.md

**Текущая версия scope lock считается замороженной до релиза 1.0.**

---

## 8. История изменений

| Дата | Версия | Изменение |
|------|--------|-----------|
| 2025-01-21 | 1.0 | Initial scope lock |
