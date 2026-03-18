# ENGENE Debugging Plan - Phase 0 Status

> **Дата:** 2025-01-21
> **Статус:** PHASE 0 IN PROGRESS

---

## PHASE 0: Freeze & Truth Alignment - PROGRESS

### ✅ Completed

| Задача | Статус | Файл |
|--------|--------|------|
| V1_SCOPE_LOCK.md создан | ✅ DONE | `docs/canonical/V1_SCOPE_LOCK.md` |
| RELEASE_BLOCKERS.md создан | ✅ DONE | `docs/canonical/RELEASE_BLOCKERS.md` |
| CONFIG_WIRING_MATRIX.md создан | ✅ DONE | `docs/generated/CONFIG_WIRING_MATRIX.md` |
| Canonical config count определён | ✅ DONE | 16 файлов |
| ECS direct access аудит | ✅ DONE | 663 вызова |
| Unwrap/expect аудит | ✅ DONE | ~60+ вызовов |
| Query API расширен | ✅ DONE | `src/core/query.rs` |

### 🟡 In Progress

| Задача | Статус | Проблема |
|--------|--------|----------|
| Cargo check | 🟡 BLOCKED | rustc ACCESS_VIOLATION |
| Миграция систем на query API | 🟡 TODO | 663 вызова для миграции |

---

## PHASE 1: ECS Boundary Enforcement - NEXT

### Цель
Убрать 663 прямых доступа к ECS storage.

### Приоритет файлов для миграции

| Приоритет | Файл | Вызовов | Сложность |
|-----------|------|---------|-----------|
| 1 | `monster_ai.rs` | 81 | Высокая |
| 2 | `npc_ai.rs` | 79 | Высокая |
| 3 | `population.rs` | 36 | Средняя |
| 4 | `witness.rs` | 35 | Средняя |
| 5 | `perception.rs` | 34 | Средняя |
| 6 | `reproduction.rs` | 29 | Средняя |
| 7 | `engine.rs` | 24 | Низкая |
| 8 | `combat.rs` | 23 | Средняя |

### Подход

1. **Не переписывать всё сразу** - мигрировать по файлу
2. **Использовать новый query API** - `iter_monsters()`, `iter_ai_entities()`, etc.
3. **Добавить lint-check** - grep для `ecs\.[a-z_]+\.(get|get_mut|insert|remove)`
4. **Тестировать после каждого файла**

---

## PHASE 2: Config & Runtime Truth Hardening

### Цель
Убрать hardcoded duplicates.

### Известные дубликаты

| Файл | Config | Код |
|------|--------|-----|
| `ai/reproduction.rs` | `population.ron` | MAX_NPCS, MAX_WOLVES, etc. |
| `simulation/simulation_level.rs` | `simulation.ron` | L0/L1/L2 радиусы |
| `economy/*.rs` | `economy.ron` | monthly_required, thresholds |
| `ai/perception.rs` | `perception.ron` | hunt_radius, fear_radius |

### Решение

1. Добавить `GameConfig` parameter в системы
2. Заменить константы на `config.field`
3. Удалить hardcoded defaults

---

## PHASE 3: Persistence Hardening

### Цель
Убрать silent failures.

### Известные проблемы

- `chunk_persistence.rs`: silent return 0 on decode failure
- `save_chunks.rs`: no error propagation

### Решение

1. Добавить typed error classes
2. Заменить `return 0` на `return Err(...)`
3. Добавить metrics для failures

---

## PHASE 4: Cut Fake Completeness

### Решения

| Элемент | Решение |
|---------|---------|
| Parallel tick | Sequential default, documented exception |
| Content pipeline | Authored content only, cook = partial |
| GPU jobs | Stub, вывести из 1.0 claims |

---

## PHASE 5: Release Engineering & CI

### Минимальный CI

```yaml
jobs:
  build-dev:
  build-release:
  test-unit:
  doctor-strict:
```

---

## Текущие блокеры

1. **rustc ACCESS_VIOLATION** - нужно перезапустить IDE/терминал
2. **663 ECS direct access** - миграция в процессе
3. **~60 unwrap/expect** - нужно исправить

---

## Следующие шаги

1. ✅ Перезапустить терминал/IDE
2. 🔄 Запустить `cargo check`
3. 🔄 Мигрировать `monster_ai.rs` на query API
4. 🔄 Мигрировать `npc_ai.rs` на query API
5. 🔄 Добавить CI workflow
