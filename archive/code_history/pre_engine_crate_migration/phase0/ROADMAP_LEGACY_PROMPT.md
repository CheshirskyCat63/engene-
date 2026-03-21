
Conduct full engineering audit for ENGENE
17 мар.
·
engene-
·
main
·
+722
-0

Архивировать

Поделиться

Создать PR


You are performing a FULL ENGINEERING AUDIT, BUG AUDIT, ARCHITECTURAL RISK REVIEW, and DOCUMENTATION GENERATION for a game engine repository.

Your task is NOT to summarize casually.  
You must scan the entire codebase and produce a **production-grade technical documentation package**, plus a **real engineering verdict**.

Treat this as if you were a senior engine architect and release reviewer preparing the final internal technical review before a 1.0 release.

The engine name is: ENGENE.

Your goals:

1. Extract the complete architecture of the engine
2. Verify roadmap compliance
3. Identify all subsystems
4. Identify all dependencies and module relations
5. Extract configuration sources
6. Validate data-driven architecture
7. Verify ECS access rules
8. Verify tests and coverage
9. Verify determinism / save / crash policies
10. Audit bugs, risky code paths, and architectural fragility
11. Detect false completeness (features that exist in code but are not actually integrated)
12. Produce an honest production-readiness verdict
13. Generate final technical documentation

You must work step-by-step, and you must distinguish clearly between:
- IMPLEMENTED
- INTEGRATED
- TESTED
- SHIPPING-READY
- PARTIAL
- STUB
- NOT FOUND

Do not assume.  
Do not hallucinate.  
Always reference file paths and code evidence.

--------------------------------------------------
PHASE 1 — REPOSITORY STRUCTURE SCAN

Scan the entire repository and output:

1. Full module tree
2. All directories
3. All Rust modules
4. Binary entry points
5. Tools and scripts
6. Assets directories
7. Config directories

Produce:
- Module tree
- Subsystem grouping

Also list all binaries:
src/bin/*
and their roles.

--------------------------------------------------
PHASE 2 — DEPENDENCY GRAPH

Build the internal dependency graph.

For each module determine:
- imports
- cross-module dependencies
- cyclic dependencies
- ownership boundaries

Output:
1. Dependency matrix
2. subsystem boundaries
3. hot files (high fan-in)
4. architectural risk zones
5. likely merge-conflict hotspots

--------------------------------------------------
PHASE 3 — ECS ANALYSIS

Detect the ECS implementation.

Locate:
- Ecs struct
- component storages
- query system
- systems

Check for violations of the rule:
NO DIRECT STORAGE ACCESS

Search for patterns like:
ecs.transforms.get
ecs.ai_states.get
ecs.*.get(
direct storage field access
pub fields exposing storages

Report:
1. Query API usage
2. Direct storage access occurrences
3. System → component access patterns

Produce a table:
System | Components Read | Components Written | Access Method (Query / Direct / Mixed)

Mark violations clearly.

--------------------------------------------------
PHASE 4 — CONFIGURATION SYSTEM

Scan for configuration loading.

Identify:
- .ron files
- config loaders
- GameConfig
- data_loader.rs
- fallback constants
- hardcoded constants that overlap configs

List every config file.

Expected canonical set (example):
biomes.ron
economy.ron
food_chain.ron
goals.ron
jobs.ron
materials.ron
material_bridge.ron
perception.ron
population.ron
rules.ron
seasons.ron
simulation.ron
species.ron
surfaces.ron
tactics.ron
weapons.ron

For each config output:
File | Exists | Parsed | Bound in GameConfig | Runtime Usage | Hardcoded duplicates | Scope (required/optional)

Also detect:
- config files referenced only by tooling
- config files referenced only by doctor.rs
- config files present but unused
- fake configs (exist but not consumed)
- hidden fallback logic that bypasses config

--------------------------------------------------
PHASE 5 — SYSTEMS AND GAME LOOP

Locate:
main.rs
engine.rs
tick pipeline
system registration

Determine:
- game loop architecture
- system execution order
- streaming integration
- render integration
- audio integration
- startup sequence
- shutdown sequence

Produce:
ENGINE EXECUTION PIPELINE DIAGRAM

Also identify:
- orchestration still left outside systems
- monolithic control flow
- duplicated execution logic

--------------------------------------------------
PHASE 6 — SUBSYSTEM INVENTORY

Detect all subsystems.

Expected major areas:
Core
ECS
Events
Jobs / parallelism
Physics
Ballistics
Fire simulation
AI
Emotion system
Economy
World streaming
Chunk persistence
Rendering
Particles
Audio
Navigation
Content pipeline
Editor
Memory manager
Metrics / observability

For each subsystem produce:
Subsystem name
Main files
Status (production / experimental / partial / planned / deprecated)
Integration level
Test coverage
Critical dependencies
Known risks

--------------------------------------------------
PHASE 7 — TEST COVERAGE

Scan:
tests/
#[cfg(test)]
benches/

List all tests.

Classify them:
- Unit tests
- Integration tests
- Benchmarks
- Soak tests
- Determinism tests
- Save/load tests
- Trust boundary tests
- Migration tests
- Tooling tests

Output:
Module | Test Count | Coverage type | Risk level if under-tested

Also detect:
- modules with zero tests
- files that are high-risk and only integration-tested
- tests marked ignored
- tests that exist but do not assert much

--------------------------------------------------
PHASE 8 — SAVE / PERSISTENCE

Locate:
save_chunks.rs
chunk_persistence.rs
build_manifest.rs

Verify:
- save schema version
- chunk schema version
- entity snapshot schema
- migration registry
- atomic write
- backup saves
- partial recovery
- load validation
- corrupted save handling

Output:
SAVE PIPELINE DOCUMENTATION

Also answer:
- Can this system survive interrupted writes?
- Can it load previous versions?
- Is persistence deterministic enough for production?
- Is save/load torture evidence present in code/tests?

--------------------------------------------------
PHASE 9 — OBSERVABILITY

Locate:
metrics_registry.rs
profiler_dashboard.rs
runtime dashboards
doctor.rs

List all metrics.

Categories:
- Engine metrics
- AI metrics
- Physics metrics
- Memory metrics
- Event bus metrics
- Streaming metrics
- Render metrics
- Crash metrics

Output:
- metrics table
- producers
- consumers
- export paths
- retention or artifact paths if present

Also detect:
- metrics registered but never updated
- metrics used in doctor thresholds
- observability gaps

--------------------------------------------------
PHASE 10 — PERFORMANCE SYSTEM

Locate:
parallel tick
frame graph
job scheduler
AI budgets
LOD systems
quality governor

Determine:
- parallelization model
- access conflict detection
- determinism tests
- fallback rules
- perf budget enforcement
- AI scheduling
- physics LOD enforcement
- renderer degradation strategy

Output:
ENGINE PERFORMANCE ARCHITECTURE

Also detect:
- code that exists but is not enabled
- fake parallelism
- fallback-only code paths
- perf-critical systems still running sequentially

--------------------------------------------------
PHASE 11 — CONTENT PIPELINE

Locate:
import pipeline
cook pipeline
validator
asset manager
content hash / invalidation code

Describe pipeline:
Import → Cook → Validate → Runtime load

Also detect:
- cooked schema versioning
- prefab schema versioning
- invalidation rules
- cycle detection
- quarantine handling for invalid content

--------------------------------------------------
PHASE 12 — TOOLING

Detect tools:
doctor.rs
editor
cook tools
doc generators
CLI tools
safe mode
layout persistence
symbolication scripts

Describe capabilities.

Also detect:
- tools that exist but are not wired
- partial editors
- panels that can crash without isolation
- doctor checks that are stale or wrong

--------------------------------------------------
PHASE 13 — RELEASE SYSTEM

Locate:
CI workflows
build scripts
packaging scripts
version stamping
symbol generation
first-run validation scripts
compatibility docs
release checklist

Describe release pipeline.

Also detect:
- workflows present but not complete
- jobs not actually gating release
- missing symbol generation
- lack of reproducibility controls
- missing clean-machine validation

--------------------------------------------------
PHASE 14 — ROADMAP VALIDATION

Compare actual code with roadmap requirements.

Check:
- Data-driven configs
- Query ECS
- Stub removal
- Parallel tick
- Metrics
- Content governance
- Save torture
- CI gates
- Release blockers
- Scope lock violations
- stop-doing violations

Produce:
ROADMAP COMPLIANCE REPORT

For each roadmap item mark:
IMPLEMENTED / PARTIAL / NOT FOUND / CONTRADICTED

--------------------------------------------------
PHASE 15 — BUG AUDIT (NEW)

Perform a real bug-oriented audit.

Look for:
- unwrap() / expect() in critical paths
- TODO / FIXME / unimplemented! / panic! in shipping paths
- dead code in critical modules
- stale feature flags
- code paths that can never execute
- obvious race risks
- persistence corruption risks
- invalid fallback logic
- impossible assumptions
- duplicated logic
- mismatch between config and runtime
- missing error propagation
- unsafe startup assumptions
- missing file existence checks
- silent failures
- metrics/reporting that can lie

Output:
BUG AUDIT TABLE

Columns:
ID | Severity | File | Problem | Why it is dangerous | Suggested fix

Severity values:
CRITICAL / HIGH / MEDIUM / LOW

--------------------------------------------------
PHASE 16 — ARCHITECTURAL RISK AUDIT (NEW)

Perform a dedicated architectural risk review.

Detect:
- excessive coupling
- monolithic hot files
- fake abstraction boundaries
- game-specific logic inside engine core
- feature garden syndrome
- code claiming to be generic but actually game-bound
- duplicated source of truth
- systems too central to change safely
- bus factor = 1 zones
- release process fragility
- schema compatibility fragility

Output:
ARCHITECTURAL RISK REGISTER

Columns:
Risk ID | Severity | Area | Description | Evidence | Release impact | Recommendation

--------------------------------------------------
PHASE 17 — FALSE COMPLETENESS AUDIT (NEW)

Identify features that appear complete in code but are not truly production-ready.

For each major feature/subsystem classify:
- CODE EXISTS
- ENABLED IN RUNTIME
- USED BY DEFAULT
- COVERED BY TESTS
- SHIPPING READY

Look especially for:
- systems compiled but never registered
- features gated but not reachable
- files that exist but are unused
- fallback-only implementations
- metrics present but not consumed
- config files present but not bound
- tools present but not wired
- test files present but not run in CI

Output:
FALSE COMPLETENESS MATRIX

Feature | Code Exists | Registered | Used in Runtime | Tested | Shipping Ready | Notes

--------------------------------------------------
PHASE 18 — RELEASE BLOCKER AUDIT (NEW)

Evaluate the codebase against a release-blocker model.

Check for:
- direct ECS storage access
- config duplication
- oversized manual orchestration in main.rs
- unresolved stubs in 1.0 scope
- unwrap in critical paths
- undefined trust boundaries
- missing determinism guarantees or fallback
- missing observability
- failing save/load torture guarantees
- unversioned content formats
- doctor strict gaps
- first-run validation gaps
- missing symbol archive
- benchmark regressions

Output:
RELEASE BLOCKER STATUS TABLE

Columns:
Blocker | Status (PASS / FAIL / PARTIAL / EXCEPTION) | Evidence | Blocking? | Notes

Be strict.

--------------------------------------------------
PHASE 19 — PRODUCTION READINESS VERDICT (NEW)

Based only on code evidence, tests, tooling, and release system, answer honestly:

Is this engine:
1. NOT READY FOR PRODUCTION
2. RELEASE CANDIDATE ONLY
3. READY FOR SINGLE-PLAYER 1.0 PRODUCTION
4. READY FOR FULL PLATFORM-QUALITY PRODUCTION

You must justify the verdict with hard evidence.

Output:
PRODUCTION READINESS VERDICT

Include:
- final verdict
- strongest evidence for readiness
- strongest evidence against readiness
- remaining blockers
- acceptable documented exceptions
- exact next steps before release

Important:
Do not use motivational language.
Do not say "looks good overall" unless evidence supports it.
Be willing to say "NOT READY".

--------------------------------------------------
PHASE 20 — FINAL DOCUMENTATION OUTPUT

Generate full documentation sections:

1. Engine Overview
2. Architecture
3. Subsystem Map
4. ECS Design
5. Configuration System
6. Execution Pipeline
7. Physics System
8. AI System
9. Economy System
10. Rendering System
11. Streaming System
12. Persistence System
13. Observability
14. Tooling
15. CI/CD
16. Release Process
17. Bug Audit Summary
18. Architectural Risk Summary
19. False Completeness Summary
20. Production Readiness Verdict

Also include:
- Risk assessment
- Technical debt
- Architecture strengths
- Recommended improvements
- Release blockers and mitigations

--------------------------------------------------
IMPORTANT RULES

Do not hallucinate.
If something is missing in code, mark it clearly as:
NOT FOUND
PARTIAL
UNIMPLEMENTED

Do not assume.

Always reference file paths.

If roadmap text contradicts repository state, say so explicitly.

If documentation claims something but code evidence does not confirm it, mark:
CLAIM NOT VERIFIED

--------------------------------------------------
FINAL OUTPUT FORMAT

Return a structured ENGINE DOCUMENTATION + AUDIT report.

Include:
- Architecture diagrams
- Subsystem tables
- Config mapping tables
- Test coverage tables
- Dependency graph
- Roadmap compliance analysis
- Bug audit table
- Architectural risk register
- Release blocker status table
- Final production readiness verdict

The final document should read like an internal engine technical manual plus a release audit report.
Копировать
Summary
Added a new full technical documentation and audit package at docs/canonical/ENGENE_PRODUCTION_AUDIT_2026.md, covering all 20 requested phases: architecture mapping, dependency graph, ECS compliance, config wiring, game loop/system ordering, subsystem inventory, tests, persistence, observability, performance, content pipeline, tooling, release process, roadmap compliance, bug/risk audits, false-completeness checks, release blockers, and final production-readiness verdict.

The report explicitly classifies findings using the requested statuses (IMPLEMENTED / INTEGRATED / TESTED / SHIPPING-READY / PARTIAL / STUB / NOT FOUND), and includes blocker/risk matrices plus an explicit verdict of RELEASE CANDIDATE ONLY with concrete next steps before 1.0.

Committed on the current branch with commit 8a763ba, and created a PR entry via the required tool.

Testing

✅ find .. -name AGENTS.md -print

✅ find . -maxdepth 3 -type d | sort

✅ cat Cargo.toml

✅ python - <<'PY' ... (module/dependency/ECS/config/test inventory scripts used during audit generation)

⚠️ cargo test -q (long-running compile/test process in this environment; no final completion output was returned within the interactive polling window)

⚠️ cargo check -q --bin engene_headless --no-default-features --features headless && echo CHECK_OK (same limitation: long-running build with no terminal completion signal observed during polling)


файл (1)
смотри такой план исправления всех ошибок пойдет или както расширить? Да. Ниже — чистый, приземлённый roadmap, уже не “всё обо всём”, а именно как довести ENGENE из current RC-state до честного single-player production 1.0.

Я специально выкину лишнюю романтику и оставлю:

что чинить,

в каком порядке,

что является выходным критерием,

что не трогать до 1.1.

ENGENE → Production 1.0 Hardening Roadmap
Главная цель

Довести движок до состояния:

single-player only

sequential shipping-safe by default

без фальшивых claims

с реальными release gates

с предсказуемой persistence/config/runtime behavior

Главный принцип

Сейчас не время “улучшать движок”.
Сейчас время убирать ложную зрелость и закрывать реальные блокеры.

Что считаем блокерами прямо сейчас

По аудиту у тебя реальные стопперы такие:

P0 — без этого не выпускать

Direct ECS storage access

Silent config fallback for required configs

Silent persistence load failures

Content pipeline placeholder logic в 1.0 scope

Отсутствие реального CI release gating

Unwrap/expect в критических путях

Нечёткая truth-синхронизация между roadmap / doctor / code

P1 — можно выпустить только с documented exception

Parallel tick не настоящий

Metrics partially wired

Некоторые release ops могут быть manual

Часть тестового покрытия больше integration-heavy, чем boundary-focused

Порядок работ

Я бы делал 5 жёстких фаз, без расползания.

PHASE 0 — Freeze & Truth Alignment
Цель

Зафиксировать реальность. Прекратить спор документации с кодом.

Что сделать
0.1 Зафиксировать 1.0 scope письменно

Создать или обновить документ:

docs/canonical/V1_SCOPE_LOCK.md

Там написать:

1.0 = single-player

shipping default = sequential tick

networking = out of scope

deferred/forward+ = out of scope

advanced animation = out of scope

content pipeline: либо в scope, либо explicitly demoted

cook/prefab если не готовы — вывести за 1.0

0.2 Синхронизировать roadmap и реальность по config set

Исправить строку:

не “19 .ron”, если реально top-level их 16

Создать один canonical список:
src/core/game_config.rs или src/core/data_loader.rs

Doctor должен брать required configs оттуда, а не из своего ручного массива.

0.3 Завести release blocker register как живой файл

Создать:
docs/canonical/RELEASE_BLOCKERS.md

С текущими блокерами:

RB-01 ECS direct access

RB-02 config truth ambiguity

RB-03 persistence silent fail

RB-04 unresolved placeholders in 1.0 path

RB-05 CI gates missing

RB-06 critical unwraps

RB-07 fake parallel claim

RB-08 metrics partial wiring

Done criteria

scope lock зафиксирован

canonical config set определён

roadmap/doc/doctor больше не противоречат друг другу

release blockers перечислены в одном месте

PHASE 1 — ECS Boundary Enforcement
Цель

Убрать главный архитектурный дефект: прямой доступ к ECS storage.

Почему это первое

Пока все системы лезут в ecs.transforms, ecs.ai_states и прочее, ты не можешь честно заявлять:

query-based architecture,

controlled access,

safe evolution,

meaningful parallel scheduling.

Что сделать
1.1 Закрыть публичные storage fields

В src/core/ecs.rs:

storage fields перестают быть публичными для runtime systems

доступ остаётся только:

внутри ECS

внутри query layer

в низкоуровневых ECS tests, если нужно

1.2 Довести query API до usable состояния

Если текущий API неудобен, не надо героически терпеть. Доведите его до нормального вида.

Цель:

single component read/write

tuple queries

with/without filters

iteration without awkward ceremony

1.3 Мигрировать системы по приоритету

Порядок:

src/ai/*

src/physics/*

src/simulation/*

src/economy/*

src/world/*, где есть runtime logic

src/gameplay/*

src/tools/*, если они мутируют ECS напрямую

1.4 Ввести временный fail-check

Добавить скрипт или grep-check в CI/doctor:

искать ecs\.[a-zA-Z_]+\.(get|get_mut|insert|remove)

исключить только query-layer и ECS internals

Что не делать

не пытаться одновременно сделать “идеальный generic ECS”

не переписывать полдвижка на новую философию

цель тут: enforced boundary, а не диссертация по ECS

Done criteria

0 direct storage accesses в runtime systems

query layer — единственный sanctioned path

Ecs storages не торчат наружу

тесты ECS/query зелёные

PHASE 2 — Config & Runtime Truth Hardening
Цель

Сделать required configs настоящим source of truth, а не “можно не загрузить и тихо жить дальше”.

Проблема сейчас

Аудит показывает:

loader есть

fallback defaults есть

ошибки могут маскироваться

Для dev это удобно. Для production — опасно.

Что сделать
2.1 Разделить config policy на dev и strict

Нужны 2 режима:

Dev mode

можно fallback-нуться

warning/log

удобно для локальной разработки

Strict / shipping mode

required config missing/invalid = hard failure

никакой тихой подмены дефолтом

2.2 Для каждого из 16 canonical .ron сделать mapping table

Создать документ:
docs/generated/CONFIG_WIRING_MATRIX.md

Колонки:

file

exists

parsed

bound in GameConfig

runtime consumer

required/optional

fallback allowed? yes/no

2.3 Удалить hardcoded duplicates

Особенно проверить:

population limits

simulation radii

jobs/economy values

goals/thresholds

perception radii

biome values

materials/surfaces bridge

2.4 Починить doctor

Doctor должен проверять:

all required configs exist

all required configs parse

strict mode fails on required config parse/load error

no stale expected filenames like game.ron

Done criteria

все required configs реально wired

в strict mode required config failure = fail

hardcoded duplicates removed or documented as fallback-only in dev

doctor uses canonical config source, not hand-maintained fantasy list

PHASE 3 — Persistence Hardening
Цель

Сделать save/load честным, диагностируемым и не молчащим.

Проблема сейчас

Аудит пишет:

silent failure on chunk load (return 0)

migration infra есть, но цепочки слабые

torture evidence не до конца подтверждено

Что сделать
3.1 Убрать silent failure

В:

src/world/chunk_persistence.rs

src/memory/save_chunks.rs

Запретить паттерны:

“не смогли прочитать/декодировать → вернуть 0 и притвориться, что всё ок”

Нужно:

structured error

log with context

metric increment

optional quarantine/bad file marker

3.2 Разделить error classes

Нужны как минимум:

FileMissing

DecodeFailed

VersionMismatch

MigrationFailed

PartialLoad

AtomicWriteFailed

BackupRestoreFailed

3.3 Довести atomic write policy

Для всех save-critical путей:

write temp

flush if applicable

rename

backup previous

recovery path

3.4 Подтвердить migration path

Сделать минимум:

одна реальная тестовая миграция

test save on old schema → load on new schema

documented save compatibility matrix

3.5 Save/load torture tests должны реально run в CI

Не просто файл существует.
Они должны быть частью release gating.

Done criteria

0 silent persistence failures

corrupted save/load produces explicit diagnostics

atomic save confirmed

migration path tested

save torture tests green in CI

PHASE 4 — Cut Fake Completeness
Цель

Убрать всё, что создаёт видимость готовности, но по факту не готово.

Это один из самых важных этапов
4.1 Parallel tick: принять правду

Сейчас есть 2 честных варианта:

Вариант A

Реально доделать parallel tick
— но это дорого и рискованно.

Вариант B

Для 1.0:

shipping default = sequential

parallel = experimental

claims в документации исправить

release blocker снять как documented exception

Для single-player 1.0 я бы выбрал Вариант B, если цель — реально выпустить.

4.2 Content pipeline: либо добить, либо выкинуть из 1.0 claims

Если:

cook phase placeholder

prefab phase placeholder

то есть 2 пути:

Путь 1

Доделать pipeline до рабочей формы

Путь 2

Сказать честно:

authored/static content only

cook pipeline not required for 1.0 shipping

pipeline stays partial/post-1.0

Но не надо оставлять “ну вроде есть”.

4.3 Metrics: подтвердить producers

Надо пройтись по metrics_registry.rs и runtime модулям и сделать таблицу:

metric registered

metric updated

metric exported

metric consumed by doctor/dashboard

Убрать “пустые” метрики или довести их до живого состояния.

4.4 Tooling claims

Если есть editor/tool that exists but not really wired, пометить честно:

partial

planned

internal-only

Done criteria

parallel tick truth aligned

content pipeline либо real, либо out-of-scope

metrics no longer lie

tooling maturity labels соответствуют коду

PHASE 5 — Release Engineering & CI
Цель

Перевести релиз из устной традиции в автоматизированный процесс.

Проблема сейчас

Аудит прямо говорит:

.github/workflows not found

Это серьёзный production blocker.

Что сделать
5.1 Добавить реальные CI workflows

Минимальный набор:

build-dev

build-release

build-headless

build-sdk

test-unit

test-integration

doctor-strict

persistence/save tests

determinism/sequential baseline tests

benchmark regression

soak test

5.2 Сделать release gates executable, а не документальными

То, что написано в RELEASE_CHECKLIST.md, должно быть:

либо CI job

либо script

либо manual validation with artifact

5.3 First-run validation

Нужен реальный сценарий:

clean machine / VM

no Rust tools

unpack package

run --version

run headless

run game

check config/assets pathing

5.4 Crash symbol flow

Если full symbol server не нужен, минимально надо:

archive symbols

tie symbol archive to build hash

symbolicate script

5.5 Versioning artifacts

Release package должен содержать:

semver

git hash

build timestamp

compatibility docs

Done criteria

CI workflows in repo

release gates reproducible

clean machine validation done

symbol archive produced

release package buildable without шаманство

PHASE 6 — Final Production Gate Review
Цель

Перед релизом не писать код, а провести жёсткую приёмку.

Что проверить руками и автоматически
Functional completeness

configs wired

ECS boundary enforced

no fake stubs in 1.0 path

sequential shipping path stable

Architectural integrity

doctor strict green

trust boundaries implemented

no contradiction between docs and code

content/save schema policy enforced

Operational reliability

tests green

save/load torture green

soak green

metrics report sane

no silent corruption paths

Release readiness

CI green

package valid

symbols archived

compatibility docs present

first-run verified

Итоговые статусы

Только один из трёх:

1. NOT READY

если есть любой незакрытый P0 blocker

2. RELEASE CANDIDATE

если P0 закрыты, но есть documented P1 exceptions

3. READY FOR SINGLE-PLAYER 1.0

если:

P0 = 0

P1 = либо 0, либо accepted with written mitigation

release ops подтверждены артефактами

Что конкретно я бы выкинул из 1.0 claims уже сейчас

Чтобы не мучить себя лишним пафосом:

Из 1.0 не обещать:

real parallel execution, если его нет

fully production content cook pipeline, если он placeholder

broader engine-generic purity

networking/deferred/advanced animation

“full production platform quality”

Для 1.0 обещать:

single-player stable runtime

sequential shipping-safe execution

data-driven required configs

save/load integrity

release automation

observability baseline

editor/tooling partial but usable

documented limits

Самый практичный краткий план по неделям

Если делать без цирка и распухания:

Sprint 1

scope lock

roadmap/doc/doctor truth alignment

canonical config set

release blockers file

Sprint 2–3

ECS direct access removal

query enforcement

direct access grep gate

Sprint 4

config strict mode

hardcoded duplicate cleanup

config wiring matrix

Sprint 5

persistence hardening

no silent load failure

atomic save verification

migration test

Sprint 6

parallel truth alignment

mark sequential shipping default

metrics producer audit

content pipeline scope decision

Sprint 7

CI workflows

doctor/test/build gates

benchmark + soak jobs

Sprint 8

package release

clean machine validation

symbol archive

final blocker review

Честный итог

Если делать по-чистому, а не “давайте ещё добавим SSAO, раз уж сели”, то ENGENE можно довести до:

single-player production 1.0

через:

закрытие ECS boundary,

жёсткую config/persistence правду,

отказ от fake-complete claims,

реальный CI/release pipeline.

Если в одной фразе

Не надо сейчас “улучшать движок”. Надо убрать ложь, закрыть границы, сделать релиз воспроизводимым и заставить код, документацию и tooling говорить одно и то же.

Если хочешь, следующим сообщением я могу превратить это в таблицу task-by-task:
задача → файл → что менять → критерий приёмки → приоритет.
Копировать
Summary

Добавил отдельный практический документ docs/canonical/ENGENE_HARDENING_ROADMAP_1_0.md с жёсткой структурой: P0/P1 blocker model, 6 фаз исполнения, done criteria, что не обещать в 1.0, sprint map и task-by-task матрица (задача → файлы → критерий приёмки).

Обновил исходный аудит docs/canonical/ENGENE_PRODUCTION_AUDIT_2026.md, добавив post-review примечание и прямую привязку к новому execution roadmap, чтобы закрыть комментарий “нужен более прикладной план, а не только аудит”.

Изменения закоммичены на текущей ветке (6da7b26) и PR-описание зафиксировано через make_pr.

Testing

✅ git status --short && nl -ba docs/canonical/ENGENE_HARDENING_ROADMAP_1_0.md | sed -n '1,260p' && nl -ba docs/canonical/ENGENE_PRODUCTION_AUDIT_2026.md | tail -n 20

✅ git add docs/canonical/ENGENE_HARDENING_ROADMAP_1_0.md docs/canonical/ENGENE_PRODUCTION_AUDIT_2026.md && git commit -m "docs: add actionable 1.0 hardening roadmap and task matrix"


файла (2)

ENGENE_HARDENING_ROADMAP_1_0.md
Новый

ENGENE_PRODUCTION_AUDIT_2026.md
Новый



Разница
Журналы

docs/canonical/ENGENE_HARDENING_ROADMAP_1_0.md
docs/canonical/ENGENE_HARDENING_ROADMAP_1_0.md
Новый
+220
-0

# ENGENE → Production 1.0 Hardening Roadmap

Статус: post-audit execution plan (после `ENGENE_PRODUCTION_AUDIT_2026.md`)
Цель: довести ENGENE до **честного single-player production 1.0** без ложных claims.

---

## 1. Главная цель и принципы

### Target state for 1.0
- single-player only;
- shipping default = sequential tick;
- required configs fail-fast в strict/shipping режиме;
- persistence без silent failure;
- release gates автоматизированы в CI.

### Главный принцип
Сейчас не “улучшение всего движка”, а закрытие P0-блокеров и синхронизация truth между code/docs/tooling.

---

## 2. Release blocker model

### P0 (без этого не выпускать)
1. Direct ECS storage access в runtime-системах.
2. Silent config fallback для required config.
3. Silent persistence load failures.
4. Placeholder-логика в 1.0 content path.
5. Отсутствие реального CI release gating.
6. `unwrap/expect` в критических путях.
7. Расхождение roadmap/doctor/code truth.

### P1 (можно только с documented exception)
1. Parallel tick как experimental path (если не завершён).
2. Partial metrics wiring.
3. Часть release operations manual (временно).
4. Integration-heavy coverage без достаточных boundary checks.

---

## 3. Phase plan (execution order)

## PHASE 0 — Freeze & Truth Alignment

### Что делаем
- Ввести/обновить `docs/canonical/V1_SCOPE_LOCK.md`:
  - single-player only;
  - sequential default;
  - networking, advanced anim, и прочее — explicit out-of-scope для 1.0 (или documented exception).
- Завести `docs/canonical/RELEASE_BLOCKERS.md` как живой реестр RB-01..RB-08.
- Устранить расхождение “16 vs 19 configs”: единый canonical list в коде и переиспользование этого списка в doctor.

### Done criteria
- scope lock зафиксирован;
- один canonical config source;
- roadmap/doc/doctor формулируют одинаковую реальность.

---

## PHASE 1 — ECS Boundary Enforcement

### Что делаем
- Закрыть `pub`-доступ к component storages в `src/core/ecs.rs`.
- Расширить/дошлифовать query API до пригодного для систем.
- Мигрировать runtime-системы на sanctioned ECS access path:
  - `src/ai/*`, `src/physics/*`, `src/simulation/*`, `src/economy/*`, `src/gameplay/*`, runtime-части `src/world/*`.
- Добавить gate-проверку (doctor/CI script) на прямой доступ `ecs.<storage>.<get|get_mut|insert|remove>` вне whitelist.

### Done criteria
- 0 прямых storage access в runtime systems;
- storages больше не торчат наружу;
- query path — единственный sanctioned.

---

## PHASE 2 — Config & Runtime Truth Hardening

### Что делаем
- Ввести dual-mode policy:
  - Dev mode: controlled fallback + warning;
  - Strict/Shipping mode: required config error = hard fail.
- Сгенерировать `docs/generated/CONFIG_WIRING_MATRIX.md` для всех canonical `.ron`.
- Удалить hardcoded duplicates (или явно пометить как dev-only fallback).
- Перевести doctor на canonical config list из runtime/config слоя.

### Done criteria
- required configs реально wired;
- strict mode падает на missing/invalid required config;
- doctor не содержит hand-maintained stale list.

---

## PHASE 3 — Persistence Hardening

### Что делаем
- Убрать silent return-paths в load (например “decode failed -> return 0”).
- Ввести typed error classes:
  - `FileMissing`, `DecodeFailed`, `VersionMismatch`, `MigrationFailed`, `PartialLoad`, `AtomicWriteFailed`, `BackupRestoreFailed`.
- Довести atomic/backup/recovery policy для save-critical paths.
- Подтвердить минимум 1 реальную migration chain test-case.
- Включить persistence torture tests в release CI gates.

### Done criteria
- 0 silent failures;
- corruption выдает явную диагностику;
- migration path и save/load torture подтверждены в CI.

---

## PHASE 4 — Cut Fake Completeness

### Что делаем
- Parallel tick truth alignment:
  - если не production-ready — явно `experimental`,
  - shipping default остаётся sequential.
- Content pipeline decision:
  - либо реально добить cook/prefab path,
  - либо честно вывести из 1.0 claims (documented scope demotion).
- Metrics producer audit: “registered vs updated vs consumed”.
- Tooling maturity labels привести к фактическому integration status.

### Done criteria
- нет misleading claims;
- partial/stub зоны формально помечены и не маскируются как production-ready.

---

## PHASE 5 — Release Engineering & CI

### Что делаем
- Добавить `.github/workflows/*` с минимумом:
  - build (dev/release/headless/sdk),
  - unit+integration tests,
  - doctor strict,
  - persistence/determinism checks,
  - benchmark regression,
  - soak job.
- Превратить release checklist в executable gates (job/script/artifact).
- First-run clean-machine validation сценарий.
- Symbol archive + symbolication flow, привязанный к build hash.

### Done criteria
- CI workflow присутствует в репо;
- release gates воспроизводимы;
- first-run validation и symbol artifacts подтверждены.

---

## PHASE 6 — Final Production Gate Review

### Gate categories
1. Functional completeness;
2. Architectural integrity;
3. Operational reliability;
4. Release readiness.

### Final status options
1. NOT READY (любой незакрытый P0);
2. RELEASE CANDIDATE (P0 закрыты, есть P1 exceptions);
3. READY FOR SINGLE-PLAYER 1.0 (P0=0, P1=0 или formal accepted with mitigation).

---

## 4. Что НЕ обещать в 1.0

Не включать в 1.0 claims, если не доведено до production качества:
- real parallel execution;
- full production content cook pipeline;
- networking/deferred/advanced animation;
- full platform-grade production statement.

Обещать в 1.0:
- single-player stable runtime;
- sequential shipping-safe execution;
- data-driven required config discipline;
- save/load integrity;
- reproducible release pipeline;
- baseline observability.

---

## 5. Sprint map (8-week practical cadence)

1. Sprint 1: scope lock + truth alignment + blockers register.
2. Sprint 2–3: ECS boundary enforcement.
3. Sprint 4: config strict mode + wiring matrix.
4. Sprint 5: persistence hardening + migration test.
5. Sprint 6: fake completeness cleanup (parallel/content/metrics truth).
6. Sprint 7: CI workflows + gates.
7. Sprint 8: packaging + first-run + symbol archive + final gate review.

---

## 6. Task-by-task matrix (задача → файл → критерий)

| ID | Priority | Task | Primary files | Acceptance criteria |
|---|---|---|---|---|
| T-001 | P0 | Scope lock doc | `docs/canonical/V1_SCOPE_LOCK.md` | 1.0 scope утверждён, out-of-scope explicit |
| T-002 | P0 | Release blockers register | `docs/canonical/RELEASE_BLOCKERS.md` | RB-01..RB-08 задокументированы, владельцы/статусы назначены |
| T-003 | P0 | Canonical config source | `src/core/game_config.rs`, `src/tools/doctor.rs` | doctor читает canonical list, без ручного дублирования |
| T-004 | P0 | ECS storage privacy | `src/core/ecs.rs` | runtime code не может обращаться к storages напрямую |
| T-005 | P0 | Query API migration | `src/core/query.rs`, runtime systems | 0 forbidden direct access hits по CI grep gate |
| T-006 | P0 | Strict config mode | `src/core/game_config.rs`, runtime startup | required parse/load fail => hard fail in shipping/strict |
| T-007 | P0 | Config wiring matrix | `docs/generated/CONFIG_WIRING_MATRIX.md` | все canonical `.ron` имеют runtime consumer mapping |
| T-008 | P0 | Remove silent load fail | `src/world/chunk_persistence.rs`, `src/memory/save_chunks.rs` | decode/read/version errors возвращают typed errors + diagnostics |
| T-009 | P0 | Save atomic policy verification | `src/memory/atomic_saved.rs`, persistence callers | save-critical paths проходят atomic+backup policy checks |
| T-010 | P0 | Migration proof test | persistence tests | old schema save successfully migrates/loads on current build |
| T-011 | P1 | Parallel truth alignment | `src/core/engine.rs`, docs | shipping default sequential, parallel clearly experimental |
| T-012 | P0/P1 | Content pipeline scope decision | `src/content/pipeline.rs`, scope docs | либо functional cook/prefab, либо explicit out-of-scope |
| T-013 | P1 | Metrics producer audit | `src/core/metrics_registry.rs`, producers, dashboards | no registered-but-dead critical metrics |
| T-014 | P0 | CI workflows | `.github/workflows/*` | build/test/doctor/persistence/determinism jobs green |
| T-015 | P1 | Benchmark/soak gates | workflows + benches/tests | regression threshold and soak stability tracked |
| T-016 | P1 | First-run validation | release scripts/docs | clean-machine runbook + artifact evidence |
| T-017 | P1 | Symbol archive flow | release scripts/docs | symbols archived and tied to build hash |

---

## 7. Executive one-liner

**Не расширять scope; убрать ложные claims, закрыть ECS/config/persistence/CI границы и выпустить воспроизводимый single-player 1.0.**
docs/canonical/ENGENE_PRODUCTION_AUDIT_2026.md
docs/canonical/ENGENE_PRODUCTION_AUDIT_2026.md
