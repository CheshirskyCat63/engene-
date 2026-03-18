# ENGENE 2.0 Dependency Admission Law

## Purpose
Prevent accidental boundary erosion when adding new third-party libraries.

## Admission checklist (all required)
Any new dependency MUST provide:
1. Exact problem statement and measurable pain without it.
2. Why current std/internal stack is insufficient.
3. Owning runtime role (**kernel**, **tools/sdk**, **game**, or **projection**).
4. Whether kernel can compile/run without it.
5. Whether tools can compile/run without it.
6. Whether game can compile/run without it.
7. Boundary risk introduced (upward pull risk, content coupling risk, UI coupling risk, determinism risk).
8. Replaceability analysis (std/custom alternative and migration cost).
9. Exit strategy (how to remove/replace later if needed).

If any item is missing, admission is denied.

## Kernel admission bar (stricter)
A dependency may enter kernel only if all are true:
- It directly serves deterministic state computation core.
- It does not pull authored content, renderer truth, or tool UI ownership.
- It does not force game or tools bootstrap in kernel paths.
- It keeps fixed-tick deterministic behavior auditable.

## Tools/game/projection admission
- Tools-only deps must stay out of kernel and game bootstrap.
- Game-only deps must stay out of kernel and tools bootstrap.
- Projection deps (renderer/audio visualization) must not become kernel truth owners.

## Forbidden admission patterns
- “Needed for demo only.”
- “Might be useful later.”
- Hidden fallback architecture via transitive dependency.
- Pulling UI/runtime authoring libs into kernel.

## Documentation requirement
Every admitted dependency must be classified in the canonical minimal-stack inventory with owner role and boundary impact.
