# ENGENE Release Blockers Register

> **Статус:** АКТИВНЫЙ РЕЕСТР
> **Дата создания:** 2025-01-21
> **Последнее обновление:** 2025-01-21
> **Версия:** 1.0

---

## Определения

### Severity Levels

| Level | Описание | Критерий |
|-------|----------|----------|
| **P0** | CRITICAL | Без этого НЕ выпускать. Блокер релиза. |
| **P1** | HIGH | Можно выпустить только с documented exception + mitigation |
| **P2** | MEDIUM | Желательно исправить, но не блокер |
| **P3** | LOW | Nice to have, можно перенести на post-1.0 |

### Status Values

| Status | Описание |
|--------|----------|
| 🔴 OPEN | Блокер активен, требует исправления |
| 🟡 IN_PROGRESS | Работаем над исправлением |
| 🟢 RESOLVED | Исправлено, протестировано |
| ⚪ DOCUMENTED | Принято как documented exception |

---

## P0 Blockers (CRITICAL)

### RB-01: Direct ECS Storage Access

| Поле | Значение |
|------|----------|
| **Severity** | P0 - CRITICAL |
| **Status** | 🔴 OPEN |
| **Файлы** | `src/core/ecs.rs`, все системы в `src/ai/*`, `src/physics/*`, `src/simulation/*`, `src/economy/*` |
| **Проблема** | Все системы напрямую обращаются к `ecs.transforms`, `ecs.ai_states` и другим storage fields |
| **Почему опасно** | Нарушает архитектуру query-based ECS, делает невозможным parallel scheduling, создаёт хрупкие зависимости |
| **Критерий закрытия** | 0 прямых storage access в runtime systems, query API — единственный путь доступа |
| **Связанные файлы** | `src/core/query.rs`, `src/core/access/queries.rs` |

**Паттерны для поиска:**
```regex
ecs\.[a-zA-Z_]+\.(get|get_mut|insert|remove)\(
```

---

### RB-02: Config Truth Ambiguity

| Поле | Значение |
|------|----------|
| **Severity** | P0 - CRITICAL |
| **Status** | 🔴 OPEN |
| **Файлы** | `src/core/game_config.rs`, `game/data/*.ron`, `src/tools/doctor.rs` |
| **Проблема** | 19 .ron файлов упомянуты, реально загружается ~10. Hardcoded значения дублируют config. Doctor имеет hand-maintained list. |
| **Почему опасно** | Data-code gap. Балансировка невозможна без перекомпиляции. Документация врёт. |
| **Критерий закрытия** | Canonical config list в коде, doctor использует его, hardcoded duplicates удалены или документированы |
| **Связанные файлы** | `game/data/population.ron`, `game/data/simulation.ron` |

**Известные дубликаты:**
- `MAX_NPCS=30` в коде vs `population.ron`
- `L0_RADIUS=300` в коде vs `simulation.ron`
- Job incomes в коде vs `jobs.ron`

---

### RB-03: Persistence Silent Fail

| Поле | Значение |
|------|----------|
| **Severity** | P0 - CRITICAL |
| **Status** | 🔴 OPEN |
| **Файлы** | `src/world/chunk_persistence.rs`, `src/memory/save_chunks.rs` |
| **Проблема** | Silent failure при load: "decode failed -> return 0", нет явной диагностики |
| **Почему опасно** | Потеря данных игрока без уведомления. Невозможность диагностики проблем. |
| **Критерий закрытия** | Typed error classes, явная диагностика, metrics increment, quarantine для bad files |
| **Связанные файлы** | `src/core/build_manifest.rs` (schema versioning) |

---

### RB-04: Unresolved Placeholders in 1.0 Path

| Поле | Значение |
|------|----------|
| **Severity** | P0 - CRITICAL |
| **Status** | 🔴 OPEN |
| **Файлы** | `src/graphics/gpu_jobs.rs`, `src/memory/memory_manager.rs`, `src/memory/entity_pool.rs`, `src/memory/streaming_cache.rs` |
| **Проблема** | Файлы существуют как stubs/placeholder, но находятся в 1.0 scope |
| **Почему опасно** | Ложное впечатление готовности. Код компилируется, но не работает. |
| **Критерий закрытия** | Либо реализовано, либо выведено из 1.0 claims с documented exception |
| **Связанные файлы** | V1_SCOPE_LOCK.md |

---

### RB-05: CI Gates Missing

| Поле | Значение |
|------|----------|
| **Severity** | P0 - CRITICAL |
| **Status** | 🔴 OPEN |
| **Файлы** | `.github/workflows/` (не существует) |
| **Проблема** | Нет CI/CD pipelines для release gating |
| **Почему опасно** | Release process не воспроизводим. Нет автоматической валидации. |
| **Критерий закрытия** | CI workflows: build, test, doctor, persistence tests |
| **Минимальный набор** | build-dev, build-release, test-unit, doctor-strict |

---

### RB-06: Critical Unwraps

| Поле | Значение |
|------|----------|
| **Severity** | P0 - CRITICAL |
| **Status** | 🔴 OPEN |
| **Файлы** | TBD (требуется аудит) |
| **Проблема** | `unwrap()` и `expect()` в критических путях (save/load, spawn, config load) |
| **Почему опасно** | Panic в production. Потеря данных. Crash без диагностики. |
| **Критерий закрытия** | 0 unwrap/expect в критических путях, proper error handling |
| **Требуется** | Grep аудит по файлам |

---

### RB-07: Fake Parallel Claim

| Поле | Значение |
|------|----------|
| **Severity** | P0 - CRITICAL |
| **Status** | 🔴 OPEN |
| **Файлы** | `src/core/engine.rs`, `src/main.rs` |
| **Проблема** | Документация/код утверждают parallel tick, но main loop использует sequential |
| **Почему опасно** | Misleading claims. Пользователь ожидает parallel, получает sequential. |
| **Критерий закрытия** | Либо parallel production-ready, либо documented exception с sequential default |
| **Mitigation** | V1_SCOPE_LOCK.md уже фиксирует sequential default |

---

### RB-08: Metrics Partial Wiring

| Поле | Значение |
|------|----------|
| **Severity** | P0 - CRITICAL |
| **Status** | 🔴 OPEN |
| **Файлы** | `src/core/metrics_registry.rs`, runtime модули |
| **Проблема** | Метрики зарегистрированы, но не все обновляются/экспортируются |
| **Почему опасно** | Ложные данные в doctor/dashboards. Невозможность реального мониторинга. |
| **Критерий закрытия** | Таблица: metric → registered → updated → exported → consumed |
| **Связанные файлы** | `src/tools/profiler_dashboard.rs`, `src/tools/doctor.rs` |

---

## P1 Blockers (HIGH - Documented Exception Required)

### RB-09: Content Pipeline Partial

| Поле | Значение |
|------|----------|
| **Severity** | P1 - HIGH |
| **Status** | 🟡 IN_PROGRESS |
| **Файлы** | `src/content/*` |
| **Проблема** | Cook pipeline — partial implementation |
| **Mitigation** | Authored content path works. Documented in V1_SCOPE_LOCK.md |

---

### RB-10: Networking Stub

| Поле | Значение |
|------|----------|
| **Severity** | P1 - HIGH |
| **Status** | ⚪ DOCUMENTED |
| **Файлы** | `src/network/*` |
| **Проблема** | UDP framework есть, но интеграция минимальна |
| **Mitigation** | Out of scope для 1.0 (single-player only) |

---

### RB-11: Deferred Rendering Missing

| Поле | Значение |
|------|----------|
| **Severity** | P1 - HIGH |
| **Status** | ⚪ DOCUMENTED |
| **Файлы** | `src/graphics/lighting.rs` |
| **Проблема** | Deferred shading — stub |
| **Mitigation** | Forward PBR sufficient для 1.0 scope |

---

## Summary

| Severity | Total | Open | In Progress | Resolved | Documented |
|----------|-------|------|-------------|----------|------------|
| P0 | 8 | 8 | 0 | 0 | 0 |
| P1 | 3 | 0 | 1 | 0 | 2 |
| **Total** | **11** | **8** | **1** | **0** | **2** |

---

## Audit Results (2025-01-21)

### RB-01: ECS Direct Access - IN PROGRESS

**Statistics:**
- Initial direct storage accesses: **663**
- After monster_ai.rs migration: **592**
- After npc_ai.rs migration: **517**
- **Progress: 22% reduction (146 calls fixed)**

**Migrated files:**
| File | Before | After | Reduction |
|------|--------|-------|-----------|
| `monster_ai.rs` | 81 | 6 | 93% |
| `npc_ai.rs` | 79 | 4 | 95% |

**Remaining top files:**
| File | Calls | Priority |
|------|-------|----------|
| `population.rs` | 36 | High |
| `witness.rs` | 35 | High |
| `perception.rs` | 34 | High |
| `reproduction.rs` | 29 | High |
| `engine.rs` | 24 | Medium |
| `combat.rs` | 23 | Medium |

**Mitigation completed:**
- Extended query API in `src/core/query.rs`
- Added filters: WithTransform, WithKind, WithNpc, WithMonster, WithNeeds, WithAiState, WithInventory, WithLifeInfo, WithMemory, WithEmotions, WithPlan, WithNpcEconomy, WithSocialNeeds, WithEcosystemNeeds, WithSpecies, IsAlive, IsDead
- Added query items: NpcQueryItem, MonsterQueryItem, AiEntityQueryItem, CombatantQueryItem
- Added helper methods in Ecs: get_transform(), get_transform_mut(), get_needs(), get_needs_mut(), get_memory(), get_memory_mut(), get_emotions(), get_emotions_mut(), get_plan(), get_ecosystem_needs(), get_social_needs(), get_npc_traits(), get_monster_traits(), etc.

### RB-06: Critical Unwraps - AUDIT COMPLETE

**Statistics:**
- `unwrap()` calls: ~30+ in production code
- `expect()` calls: ~30+ in production code

**Critical locations:**
- `main.rs:47`: `EventLoop::new().unwrap()`
- `emotions.rs:43`: `partial_cmp().unwrap()`
- `memory.rs:255`: `get().unwrap()`
- `engine.rs:98,102`: `expect("ResourceGrid missing")`
- `renderer.rs:342,354`: `expect("no suitable GPU adapter")`

**Pattern issues:**
- Many `partial_cmp().unwrap()` in AI code (emotions, memory, perception)
- EventLoop creation in all binaries
- Resource access in engine

### Config Files - AUDIT COMPLETE

**Actual config count: 16 files** (not 19 as previously documented)

| Config | Status | Has Hardcoded Duplicate |
|--------|--------|-------------------------|
| biomes.ron | ✅ Loaded | Partial |
| economy.ron | ✅ Loaded | Yes |
| food_chain.ron | ✅ Loaded | Yes |
| goals.ron | ✅ Loaded | No |
| jobs.ron | ✅ Loaded | Yes |
| materials.ron | ✅ Loaded | Partial |
| material_bridge.ron | ✅ Loaded | No |
| perception.ron | ✅ Loaded | Yes |
| population.ron | ✅ Loaded | **Yes** |
| rules.ron | ✅ Loaded | No |
| seasons.ron | ✅ Loaded | No |
| simulation.ron | ✅ Loaded | **Yes** |
| species.ron | ✅ Loaded | No |
| surfaces.ron | ✅ Loaded | No |
| tactics.ron | ✅ Loaded | No |
| weapons.ron | ✅ Loaded | No |

---

## Следующие шаги

1. **RB-01**: Начать миграцию на query API
2. **RB-02**: Создать canonical config list
3. **RB-06**: Провести unwrap аудит
4. **RB-05**: Добавить минимальный CI

---

## История изменений

| Дата | Изменение |
|------|-----------|
| 2025-01-21 | Создан реестр, добавлены RB-01..RB-11 |
