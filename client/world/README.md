# World

The renderer-independent [resource contracts](resource-contracts.md) define stable
raw-material identities, the initial catalog, validated deposits, and physical
fragments for the planetary collection work. Per-body resource distribution
profiles expose validated weights, spacing, mass ranges, deterministic generation
inputs, and optional biome overrides. Bounded nearby deposit materialization
uses deterministic cube-sphere cells with body-local geometry and physical bounds.
The world-owned mining session journals modified/depleted deposit mass across
local streaming independently of runtime presentation.

`salimon-world` owns the portable compressed Solar System, large-scale
coordinates, camera tour, and body geometry math. Its immutable catalog contains
exactly Sun, Mercury, Venus, Earth, Moon, and Mars in authoritative `f64` meters.
The Sun is visual-only; the other five bodies expose solid landing volumes with
outer radius `1.15R`. No orbital simulation, backend, networking, or disk persistence is involved.
Resource deltas and fragments are retained in memory for the current session.

Portable helpers select the nearest nominal body surface inside an inclusive
distance threshold and project point velocity onto the center-to-point radial
axis. Signed radial speed is negative while approaching, positive while receding,
and zero for stationary or tangential motion; selection includes the visual Sun.

A host advances `CameraPrototype` with a monotonic duration, translates
`CameraCommand` values from platform input, and reads a borrowed `WorldSnapshot`.
The snapshot exposes the static body catalog, the engineering camera, and three
noncanonical meter-scale precision markers. The runtime maps bodies to renderer
sphere/material DTOs and the Sun to a point light, preserving `f64` centers/radii.
`InspectBody` selects each catalog body's positive-Z surface for the same
120 Mm-to-12 m validation tour. Earth remains the initial target.

```text
world absolute f64 bodies + camera + markers
    -> runtime type mapping (absolute f64 preserved)
    -> renderer subtracts f64 camera origin
    -> renderer casts local result to GPU f32
    -> renderer-owned view/projection and reverse-Z depth
```

`WorldPosition::camera_relative_f32` remains a portable numeric helper for
tests and precision reporting; it is not the runtime-to-renderer handoff.

See [solar-system-layout.md](solar-system-layout.md) for the exact compressed
catalog and travel/landing math, [coordinate-strategy.md](coordinate-strategy.md)
for measured numeric limits, [architecture.md](architecture.md) for dependency
boundaries, and [invariants.md](invariants.md) before changing world behavior.

### Physical carrying

`carrying` owns the permanent one-world-object identity slot, independent of gear,
and exact cube aiming with supplied terrain/hull occlusion. `MiningSession` owns
the carried fragment ID, guarded pickup, transform changes, and release, keeping
the physical entity in the same session collection throughout. Collected pieces
are sealed against later extraction growth. Runtime composes character-relative
poses and validates surface and ship-deck placement. Runtime retains ship-local anchors
for loose interior pieces and releases those anchors on pickup; world owns
identity, raw material, mass, and the same session entities through every transfer.
