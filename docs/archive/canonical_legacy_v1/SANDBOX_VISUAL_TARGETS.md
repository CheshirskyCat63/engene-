# Sandbox Visual Targets

Visual quality targets for the Destruction Sandbox 50×50 proving ground. These define the expected look and rendering behavior under specific test conditions.

---

## Test Scenarios

### Daylight Outdoor Test
- **Zones:** A, B, D
- **Conditions:** Direct sunlight
- **Targets:**
  - Dynamic cascaded shadows clearly visible
  - Clear material distinction (wood, metal, concrete, tile, glass, soil, cloth)
  - Shadows and lighting support readability of destruction state

### Dusty Explosion Test
- **Zone:** B (Ground)
- **Action:** Grenade detonation
- **Targets:**
  - Dust and debris particles visible against sky
  - Explosion flash visible
  - Particles readable and not lost in background

### Indoor Basement Shadow Test
- **Zone:** C (Basement)
- **Conditions:** Low interior light
- **Targets:**
  - Dynamic shadows stable (no shimmer, no temporal noise)
  - Contact shadows under chairs, tables, crates
  - Muzzle flash illuminates scene briefly
  - Specular noise control on tile, metal, pipes — no excessive flicker
  - TAA stability in low-light (no flickering on small objects)

### Muzzle Flash Readability
- Weapon fire must briefly illuminate the scene
- Visible in both outdoor and indoor contexts
- Light contribution should be noticeable but not overblown

### Debris Readability
- Flying debris must be visually trackable
- Debris should not be lost against background
- Motion blur and contrast must support readability

---

## Rendering Stack (Required Order)

1. Shadows
2. G-buffer
3. SSAO
4. Deferred lighting
5. IBL (image-based lighting)
6. Atmosphere/fog
7. Contact shadows
8. TAA (temporal anti-aliasing)
9. Bloom
10. Tonemap
11. egui overlay

---

## Dynamic Shadows Only

- **No lightmaps** — Scene lit entirely by dynamic lights
- **No baked shadows** — All shadows computed at runtime
- **Cascaded Shadow Map:**
  - `CASCADE_SPLITS`: `[0.05, 0.15, 0.4, 1.0]`
- **ContactShadowPass:** For close-range contact shadows

---

## Low-Spec Acceptable Degradation

On low-spec hardware, the following reductions are acceptable:

- Reduced shadow resolution
- Fewer particle sprites
- Lower post-process quality (SSAO, bloom, etc.)

**Constraints:**

- Scene must remain readable
- All materials must remain distinguishable
- Destruction state must remain visible
- Truth state must never be degraded (see master spec)

---

## TAA Stability

- **Low-light interior:** No flickering on small objects
- TAA must not introduce temporal artifacts on debris, furniture, or fine geometry
- Stability is critical in Zone C (Basement) under low light
