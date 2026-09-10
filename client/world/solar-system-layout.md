# Phase 0 Compressed Solar System Layout

## Catalog

All centers below are offsets from Earth's center. The absolute Earth center is
`SURFACE_ANCHOR + (0, 0, -6,000,000) m`, where `SURFACE_ANCHOR` is
`(1,000,000,000,000, -750,000,000,000, 250,000,000,000) m`.

| Order | Body | Center offset (m) | Radius (m) | Role |
| ---: | --- | ---: | ---: | --- |
| 1 | Sun | `(-90,000,000, 18,000,000, 0)` | 15,000,000 | Visual only |
| 2 | Mercury | `(-58,000,000, -22,000,000, 0)` | 2,400,000 | Solid |
| 3 | Venus | `(-30,000,000, 20,000,000, 0)` | 5,500,000 | Solid |
| 4 | Earth | `(0, 0, 0)` | 6,000,000 | Solid |
| 5 | Moon | `(20,000,000, -12,000,000, 0)` | 1,600,000 | Solid |
| 6 | Mars | `(69,500,000, 0, 0)` | 3,500,000 | Solid |

This is intentionally compressed, static Phase 0 data rather than astronomical
ephemeris data. All radii are distinct. The catalog excludes Jupiter, Saturn,
Uranus, Neptune, dwarf planets, asteroids, and every other body.

## Travel tuning

The Earth-to-Mars center distance is `69,500,000 m`. Subtracting Earth's
`6,000,000 m` radius and Mars's `3,500,000 m` radius leaves a nominal-surface gap
of `60,000,000 m`. At the exported Phase 0 reference maximum speed:

```text
60,000,000 m / 500,000 m/s = 120 s
```

## Landing volumes

Every solid body has an outer landing radius of `1.15R`: its surface radius plus
the exported `0.15R` landing-altitude factor. Pairwise separation is computed as
center distance minus both landing radii and must remain strictly positive. The
Sun is `VisualOnly`, so it returns no landing radius and does not participate in
solid landing-volume checks.

## Presentation

Runtime maps each catalog body to an analytic textured sphere at its unchanged
compressed radius. The Sun uses emissive material and supplies a generic point
light; the five solid bodies use lit generic materials. Three Task 4 meter-scale
precision-marker cuboids remain separate and are not catalog members. The
renderer has no celestial IDs, landing rules, or world dependency.
