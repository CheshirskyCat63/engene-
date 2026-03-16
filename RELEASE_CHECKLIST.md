# ENGENE v1.0 Release Verification Checklist
# Выполни команды по порядку и сверь логи с ожидаемыми результатами

================================================================================
PHASE A: DATA-DRIVEN ARCHITECTURE (100%)
================================================================================

# A.1 — Проверка загрузки 16 RON конфигов
# Команда:
cargo check --lib 2>&1 | Select-String "error"

# Ожидаемый результат:
# (пусто или нет error[E...])

# A.2 — Проверка Query API
# Команда:
cargo check --lib 2>&1 | Select-String "query"

# Ожидаемый результат:
# warning: unused import: `crate::core::query::Query`

# A.3 — Проверка компонент-сплита
# Команда:
Get-ChildItem -Path src/world/components -Filter "*.rs" | Select-Object Name

# Ожидаемый результат:
# transform.rs, physics.rs, ai.rs, identity.rs, needs.rs, inventory.rs, etc.

# A.4 — Проверка main.rs < 150 строк
# Команда:
(Get-Content src/main.rs).Count

# Ожидаемый результат:
# < 150 (цель: ~68)

================================================================================
PHASE B: FEATURE GATING & STABILITY (100%)
================================================================================

# B.5 — Проверка unit tests (87+ тестов)
# Команда:
cargo test --lib 2>&1 | Select-String "test result"

# Ожидаемый результат:
# test result: ok. 87 passed; 0 failed; 0 ignored

# B.6 — Проверка error handling (нет unwrap в критических путях)
# Команда:
Select-String -Path src/core/engine.rs -Pattern "\.unwrap\(\)" | Select-Object -First 5

# Ожидаемый результат:
# (минимальное количество unwrap, только в non-critical paths)

# B.7 — Проверка SchemaMigrationRegistry
# Команда:
cargo check --lib 2>&1 | Select-String "SchemaMigration"

# Ожидаемый результат:
# (нет ошибок компиляции)

# B.8 — Проверка Memory Accounting
# Команда:
Select-String -Path src/core -Pattern "memory_manager" -Recurse

# Ожидаемый результат:
# несколько вхождений в memory_manager.rs

# B.9 — Проверка Trust Boundaries (QualityGovernor)
# Команда:
Select-String -Path src/core/quality_governor.rs -Pattern "degradation_order"

# Ожидаемый результат:
# pub fn degradation_order() -> Vec<DegradationEntry>

================================================================================
PHASE C: PERFORMANCE OPTIMIZATION (100%)
================================================================================

# C.1 — Проверка Parallel Tick + DeterminismGate
# Команда:
Select-String -Path src/core/replay/determinism_gate.rs -Pattern "pub struct"

# Ожидаемый результат:
# pub struct DeterminismGate
# pub struct EcsSnapshot
# pub struct DivergenceInfo

# C.2 — Проверка Incremental Spatial Index
# Команда:
Select-String -Path src/world/hierarchical_spatial.rs -Pattern "fn update"

# Ожидаемый результат:
# pub fn update(&mut self, entity: Entity, old_x: i32, old_z: i32, new_x: i32, new_z: i32)

# C.3 — Проверка AI Tick Staggering
# Команда:
Select-String -Path src/ai/ai_scheduler.rs -Pattern "should_tick"

# Ожидаемый результат:
# pub fn should_tick(&self, entity: Entity, elapsed_us: u64) -> bool

# C.4 — Проверка Physics LOD
# Команда:
Select-String -Path src/simulation/simulation_level.rs -Pattern "should_tick_fire"

# Ожидаемый результат:
# pub fn should_tick_fire(level: SimulationLevel, frame: u64) -> bool

# C.5 — Проверка Render Budget
# Команда:
Select-String -Path src/graphics/renderer.rs -Pattern "apply_budget_settings"

# Ожидаемый результат:
# pub fn apply_budget_settings(&mut self, governor: &QualityGovernor)

# C.6 — Проверка Benchmarks
# Команда:
Get-ChildItem -Path benches -Filter "*.rs"

# Ожидаемый результат:
# hot_paths.rs
# engine_benchmarks.rs

# C.7 — Проверка Metrics Registry (40+ метрик)
# Команда:
Select-String -Path src/core/metrics_registry.rs -Pattern "AtomicCounter\|AtomicGauge" | Measure-Object

# Ожидаемый результат:
# Count > 40

================================================================================
PHASE D: TOOLING & CONTENT GOVERNANCE (95%)
================================================================================

# D.1 — Проверка Shader Hot Reload
# Команда:
Select-String -Path src/core/hot_reload.rs -Pattern "ShaderHotReloader"

# Ожидаемый результат:
# pub struct ShaderHotReloader

# D.2 — Проверка Config Hot Reload
# Команда:
Select-String -Path src/core/hot_reload.rs -Pattern "ConfigHotReloader"

# Ожидаемый результат:
# pub struct ConfigHotReloader

# D.3 — Проверка Content Pipeline
# Команда:
Select-String -Path src/content/pipeline.rs -Pattern "ContentPipeline"

# Ожидаемый результат:
# pub struct ContentPipeline

# D.5 — Проверка Doctor Improvements
# Команда:
Select-String -Path src/tools/doctor.rs -Pattern "check_ron_configs\|check_shader_files\|check_schema_versions"

# Ожидаемый результат:
# fn check_ron_configs
# fn check_shader_files
# fn check_schema_versions

# D.6 — Проверка Documentation Generator
# Команда:
Select-String -Path src/tools/doc_generator.rs -Pattern "DocGenerator"

# Ожидаемый результат:
# pub struct DocGenerator

# D.7 — Проверка Schema Governance
# Команда:
Select-String -Path src/content/schema_governance.rs -Pattern "SchemaRegistry"

# Ожидаемый результат:
# pub struct SchemaRegistry

# D.8 — Проверка Save/Load Torture Tests
# Команда:
Get-ChildItem -Path tests -Filter "*.rs"

# Ожидаемый результат:
# save_load_torture.rs

================================================================================
FINAL BUILD CHECK
================================================================================

# Чистая компиляция
cargo check --lib 2>&1 | Select-String "error\|warning" | Select-Object -Last 10

# Ожидаемый результат:
# warning: `engene` (lib) generated N warnings (run `cargo fix...`
# Finished `dev` profile [unoptimized + debuginfo] target(s) in X.XXs

# Проверка зависимостей
cargo tree | Select-String "walkdir\|serde_json\|glob"

# Ожидаемый результат:
# walkdir v2.4
# serde_json v1.X
# glob v0.3

================================================================================
UNIT TESTS RUN
================================================================================

# Все тесты
cargo test --lib 2>&1 | Select-String "test result\|passed\|failed"

# Ожидаемый результат:
# test result: ok. N passed; 0 failed

================================================================================
INTEGRATION SMOKE TEST
================================================================================

# Быстрая проверка что игра запускается
cargo build --release 2>&1 | Select-String "Finished"

# Ожидаемый результат:
# Finished `release` profile [optimized] target(s) in XX.XXs

================================================================================
SUMMARY: Должны быть ВСЕ "Ожидаемые результаты" = ✅ ГОТОВО К РЕЛИЗУ
================================================================================
