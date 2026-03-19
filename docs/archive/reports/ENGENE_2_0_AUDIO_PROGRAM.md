# ENGENE 2.0 Audio Program

## Runtime pillars
1. Event-driven sound system with strict category budgets.
2. Obstruction/occlusion model using geometry + material acoustic coefficients.
3. Material-aware impact and destruction sound selection.
4. Reverb zones with blending and environmental sends.
5. Ambience layering linked to biome/weather/time.
6. AI hearing hooks fed by post-occlusion loudness estimates.

## Audio data ownership contract
- Sound event taxonomy is owned by `engine_audio` public contracts.
- Material acoustic profiles are defined in canonical content schema (`engine_content`).
- Environment routing/reverb zones are authored content, consumed by `engine_audio`.
- AI hearing interface contract is owned by `engine_audio` + `engine_runtime` event APIs.
- SDK audio debug panels are owned by `engine_tools`/`sdk_app`, consuming public audio APIs only.

## Algorithm strategy
- Near field: ray-cone obstruction + diffraction approximation.
- Mid field: simplified LOS + material class attenuation.
- Far field: ambience-only representation.

## Tooling required in SDK
- Audio event timeline inspector.
- Propagation rays and obstruction debug view.
- Reverb zone editor and live audition.
- Material acoustic profile editor.
- AI hearing monitor (heard events, confidence, source position error).
- Voice budget dashboard with stealing diagnostics.

## Integration requirements
- Weather affects ambience layers and high-frequency damping.
- Fire/destruction emitters auto-register with priority classes.
- Ballistics pipeline emits impact events with surface metadata.

## Acceptance criteria
- Obstructed shots audibly differ by material path.
- Indoor/outdoor transitions blend reverb reliably.
- AI hearing reaction correlates with audible loudness under occlusion.
- Voice budget remains stable in showcase stress case.
