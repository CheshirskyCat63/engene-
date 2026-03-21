# ENGENE 2.0 Scope Lock

## Scope lock statements
1. ENGENE 2.0 is a **hybrid simulation program**; literal full-fidelity everywhere is prohibited.
2. All flagship systems must support **L0-L3 simulation levels**.
3. No feature merges without budget telemetry and debug visibility.
4. SDK tooling is required deliverable, not optional polish.
5. Showcase scene is mandatory release gate for 2.0 readiness.
6. Performance-first/hot-path laws are mandatory architecture constraints, not optional optimization tasks.

## Hard acceptance gates
- Dependency direction checks pass on every phase integration.
- ECS boundary remains query/API-based with no storage leakage.
- Scene can demonstrate all flagship interactions in one executable.
- Population target (10k enemies + 5k allies) runs through multi-level abstraction with bounded frame cost.
- Audio, destruction, fire/water/weather each have debug overlays + recordable metrics.
- Concurrency, CPU, and memory contracts are defined and measured for hotspot subsystems.

## Excluded in 2.0 lock
- Full-world rigid body simulation.
- Unlimited debris persistence.
- Unbounded voxel destruction across whole map.
- Real-time global fluid simulation.

## Change-control policy
A scope change requires:
- Written impact on perf budgets.
- Updated acceptance criteria.
- Updated docs in `docs/canonical/`.
- Explicit downgrade/deferral list if schedule slips.
