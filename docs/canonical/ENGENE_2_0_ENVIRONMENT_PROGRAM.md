# ENGENE 2.0 Environment Simulation Program

## Simulation granularity
- Active cells: 1-2m grid for fire/wetness in L0.
- Near-field: coarser 4-8m grid for L1.
- Far-field: region scalar models for L2/L3.

## Sky and Weather are distributed subsystems with split ownership
Sky/weather must not be implemented as one monolithic "sky subsystem".

### Ownership split
- **Truth (authoritative state)**: `engine_world`
  - `SkyTime`
  - `WeatherController` / `WeatherState`
  - `CloudCoverageField`
  - `PrecipitationField`
  - `FogDensityField`
  - `WetnessState`
  - local exposure/shelter truth
- **Orchestration**: `engine_runtime`
  - update order
  - phase scheduling
  - snapshot build/distribution
  - event routing
- **Presentation**: `engine_render`
  - sky LUT
  - cloud rendering
  - fog rendering
  - rain/snow particles
  - moon/stars/postprocess sky effects
- **Authoring**: `engine_content`
  - weather presets
  - cloud profiles
  - fog configs
  - day/night curves
  - storm archetypes
  - precipitation response curves
  - climate zone configs
- **Debug/SDK**: `engine_tools` + `sdk_app`
  - weather debugger
  - sky overlays
  - storm visualizer
  - wetness heatmap
- **Game**: `game_framework`
  - triggers only (e.g. mission/event starts storm or requests preset transition)

### Critical invariant
- WORLD knows weather.
- RENDER shows weather.
- GAME does not own weather truth.

## Frame contract (must-have)
All runtime consumers read a frame snapshot contract; they must not read mutable world weather truth directly.

```rust
pub struct FrameSkyWeatherSnapshot {
    pub time_of_day: f32,
    pub sun_dir: Vec3,
    pub moon_dir: Vec3,

    pub weather_state: WeatherState,
    pub wind: Vec3,

    pub cloud_coverage: f32,
    pub precipitation_intensity: f32,
    pub precipitation_type: PrecipitationType,

    pub fog_density: f32,
    pub wetness_avg: f32,
}
```

### Snapshot publication order (required)
1. `engine_world` finishes weather progression for the tick.
2. `engine_runtime` materializes exactly one `FrameSkyWeatherSnapshot`.
3. Snapshot is published to render/audio/AI/game/debug readers.
4. Readers consume; no weather truth mutation is allowed until next tick.

### Immutable snapshot law
- `FrameSkyWeatherSnapshot` is built once per simulation tick/frame phase.
- After publication it is immutable for that frame.
- Render/audio/AI/debug/gameplay readers consume the same published snapshot.
- No subsystem may mutate canonical sky/weather values after snapshot publication.
- No subsystem may bypass snapshot to query mutable weather internals in hot frame path.

## Weather event policy (edge notifications only)
Minimum weather event taxonomy:
- `WeatherStateChanged`
- `RainStarted`
- `RainStopped`
- `StormSpawned`
- `StormDissipated`
- `FogEnteredThreshold`
- `FogExitedThreshold`

Event truth rule:
- Weather events are notifications, not authoritative truth.
- Authoritative weather truth lives in `engine_world` + published frame snapshots.
- Gameplay/runtime must not infer durable weather truth from a single event alone.
- When event delivery is dropped/aggregated, consumers still rely on snapshot truth.

## Sky/Weather invariants
- Weather truth exists independently of renderer presence.
- Render may lag by frame cadence, but must not invent truth.
- Game triggers may request weather changes, but cannot set render sky state directly.
- Visible weather state must be explainable from canonical snapshot.
- Precipitation indoors requires explicit exposure path; never default spawn.
- Wetness accumulation in loaded cells is authoritative.
- Far-field weather may be approximated but must transition coherently into local truth.

## Coupling rules
- Fire spread requires burnable material + ignition temp + oxygen proxy.
- Wetness reduces ignition probability and burn rate.
- Wind biases spread direction and ember jump probability.
- Rain increases wetness, fills containers, and can extinguish low-intensity fires.
- Punctured containers create leak source terms feeding local wetness map.

## Sky/Weather degradation ladder
### Tier A — never fake (authoritative)
- `SkyTime`
- `WeatherController` / weather truth state
- precipitation truth
- wetness truth in loaded cells
- local fog truth drivers
- shelter/exposure classification

### Tier B — degradable presentation/cadence
- volumetric cloud quality
- cloud shadow update cadence
- fog rendering resolution
- precipitation occlusion refresh cadence
- lightning visual richness

### Tier C — cosmetic polish
- star richness
- moon detail
- cloud fine noise detail
- postprocess mood polish
- dense precipitation cosmetic particles

Law:
- Sky/weather truth may not be degraded into contradiction.
- Only presentation fidelity and update cadence may degrade under stress.

## Program phases

### D1: Sky truth baseline (gate)
- Establish single authoritative `SkyTime`.
- Establish single authoritative `WeatherController` / `WeatherState`.
- Implement immutable `FrameSkyWeatherSnapshot` contract + runtime distribution.
- Implement basic weather edge events.
- Validate single-source-of-truth and snapshot consistency across render/audio/AI/debug readers.
- **No sky visual expansion before this gate is complete.**

### D2: Atmosphere + sky lighting
- Sun/moon direction from `SkyTime`.
- Sky lighting + atmosphere LUT policy.
- Sunrise/sunset continuity from truth-driven state.

### D3: Clouds + precipitation
- Cloud coverage truth + wind-driven motion.
- Rain/snow/hail visuals from weather truth.
- Precipitation occlusion/shelter path rules.
- Weather events used as edges only.

### D4: Wetness + hydrology + interiors
- Wetness accumulation and decay.
- Runoff/puddle behavior.
- Container fill/leak coupling.
- Interior weather volumes / shelter classes.

## Performance budgets
- Max active fire cells per region.
- Max wetness updates per tick via dirty-region scheduler.
- Weather updates on fixed coarse cadence outside active bubble.

## Local activation rules
- High-frequency simulation only in active + nearby combat zones.
- Dormant sectors switched to event-summarized state transitions.

## Far-field visual model
- Volumetric skybox layers + front billboards + lightning sheets.
- No full far-field fluid or combustion simulation.

## Acceptance criteria
- Fire only propagates over burnable networks.
- Rain/wetness measurably suppresses fire spread.
- Punctured containers produce visible leaks and wetness growth.
- Distant fronts are visible and synchronized with eventual local weather shift.
- Sky/weather frame snapshot is singular, immutable per-frame, and consistent across render/audio/AI consumers.
- Indoor precipitation appears only with explicit exposure path.
