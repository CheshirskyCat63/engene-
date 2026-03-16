# ART DIRECTION BIBLE

Visual language reference for ENGENE's 2x2 km world slice.
All lighting, color, and atmosphere decisions flow from this document.

## Time of Day

### Dawn (05:00–07:00)
- Warm orange/pink horizon, cool blue overhead
- Long shadows, low sun angle
- Fog lifts gradually — visibility starts at ~200m, clears to 800m
- Ambient: birds, distant camp activity

### Noon (11:00–14:00)
- Harsh overhead light, minimal shadows
- Washed-out colors in open areas
- Maximum visibility (~1km+)
- Heat shimmer on roads and open plains

### Dusk (18:00–20:00)
- Deep amber/red horizon, purple overhead
- Dramatic long shadows returning
- Camp fires becoming visible
- Fog descends — visibility drops to ~400m

### Night (22:00–04:00)
- Deep blue moonlight, desaturated world
- Strong contrast: lit areas (camps) vs pitch-dark wilderness
- Monster activity peaks — red anomaly glows visible
- Visibility: ~100m without light source, ~50m in fog

## Weather

### Rain
- Desaturated palette, grey sky
- Wet surface reflections on terrain and props
- Rain particle density: heavy=2000, light=500
- Reduced draw distance (~300m)
- Puddle accumulation on flat surfaces

### Fog
- White/grey atmospheric volume
- Exponential falloff: density peak in swamps/valleys
- Reduces effective LOD distance by 50%
- Anomaly glow pierces fog (visible at 2x normal range)

### Clear
- Full color saturation
- Maximum draw distance
- Crisp shadows

## Biome Visual Identity

### Plains
- Yellow-green grass, sparse bushes
- Wide open sightlines
- Road paths clearly visible
- Warm earth tones

### Forest
- Dense canopy, dappled light
- Dark green dominant, brown undergrowth
- Reduced visibility through trees (~80m)
- Moss and fern ground cover

### Swamp
- Murky green/brown water
- Dead trees, exposed roots
- Permanent low fog layer
- Bioluminescent anomaly effects

### Hills / Rocky
- Grey-brown rock faces
- Sparse vegetation
- Wind-exposed — stronger wind particles
- Clear sightlines from elevation

## Location Archetypes

### Camp (Warmth)
- Warm point lights (fire, lanterns)
- Orange/amber color temperature
- Smoke particles rising
- Sense of safety: higher ambient brightness within perimeter

### Ruins
- Desaturated, cold tones
- Broken geometry, exposed interiors
- Dust particles in light shafts
- Sense of abandonment: lower ambient, more shadow

### Anomaly Zone
- Unnatural color shifts (green/purple glow)
- Particle distortion effects
- Spatial warping hints (heat-haze-like)
- Dangerous: pulsing light, audible hum

### Hostile Exterior
- Cold blue/grey palette
- Limited cover, exposed terrain
- Threat visibility: monster silhouettes at distance
- Wind and weather more pronounced

## Material Guidelines

- **Terrain**: 4-layer splat map (grass, dirt, rock, mud)
- **Props**: PBR with roughness 0.4–0.9, minimal pure metals
- **Characters**: Subsurface-hinted skin, cloth with anisotropic hints
- **Vegetation**: Two-sided leaves, wind vertex animation
- **Water**: Screen-space reflections, depth-based color shift

## Low-Spec Adaptations

- TAA off: use FXAA fallback
- SSAO off: bake ambient into lightmaps where possible
- Vegetation density: 50% reduction, no wind animation
- Particles: 50% count reduction
- Shadow resolution: halved
- No contact shadows
- Impostor LOD for vegetation at 50% normal distance

All low-spec changes are purely visual — no truth-affecting state changes.
