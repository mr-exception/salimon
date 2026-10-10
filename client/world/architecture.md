# World Architecture

## Source map

`src/lib.rs` is the public facade: existing root imports stay stable, while
non-resource implementation modules remain private.

| Source | Responsibility and existing tests |
| --- | --- |
| `src/coordinates.rs` | `WorldPosition`, absolute `f64` arithmetic and subtract-before-cast regression |
| `src/catalog.rs` | Body IDs/roles/definitions, ordered static catalog, anchor/reference speed, membership and travel tuning tests |
| `src/geometry.rs` | Body surface/landing distances, radial speed, sphere separation and landing-volume tests |
| `src/proximity.rs` | Nearest-surface and nearby-solid selection, inclusive threshold and influence tests |
| `src/precision.rs` | Three immutable noncanonical markers, scalar spacing reports and numeric precision regressions |
| `src/camera.rs` | Camera commands/state/snapshots, timeline sampling and deterministic tour tests |

Resource modules keep their existing ownership and public module paths.

## Responsibility

The `resources` module owns the [planetary resource contracts](resource-contracts.md):
stable material/entity identities, SI mass/volume properties, bounded deposit
state, and physical fragment poses. It has no rendering or backend dependencies.

`salimon-world` owns portable absolute coordinates, the immutable six-body
catalog and geometry helpers, the deterministic engineering camera tour, three
noncanonical precision markers, and numeric precision reporting. It does not own
platform input, frame scheduling, GPU buffers, projection matrices, depth
textures, presentation policy, orbital simulation, or backend behavior.

```text
runtime monotonic delta + typed CameraCommand
    -> salimon-world CameraPrototype
        -> borrowed WorldSnapshot
           (f64 camera + six bodies + three precision markers)
    -> runtime maps domain structs into renderer sphere/light/cuboid DTOs
       and diagnostics BodyDistance values
    -> renderer subtracts camera in f64 and casts relative values to f32
```

The catalog is a static ordered slice. `CelestialBody` carries stable identity,
name, absolute center, compressed radius, display color, and `BodyRole`.
Geometry helpers calculate center distance, signed nominal-surface separation,
optional solid landing-volume separation, and nonnegative point-to-surface
distance. They also provide deterministic nearest-surface selection and signed
radial point velocity without assuming a ship or renderer. `VisualOnly`
deliberately makes the Sun's landing radius absent but not absent from proximity.

The runtime is the composition root and translates `winit` events into
`CameraCommand`. The renderer does not depend on this crate. It receives generic
renderer DTOs produced by field-by-field runtime mapping: a body becomes a
sphere with an absolute `f64` radius, while precision markers remain separate
cuboids. Generic material selection and Sun-to-point-light mapping belong to
runtime; textured sphere drawing and lighting belong to renderer.

The camera retains a selected catalog identity. `InspectBody` restarts the same
validation timeline at that body's positive-Z surface; default is Earth, and
restart retains selection. No catalog positions or gameplay behavior change.

`WorldPosition::camera_relative_f32` documents and tests the numeric conversion
order but does not move presentation conversion into world. Projection matrices,
clip-space conventions, depth targets, and reverse-Z policy remain renderer
concerns.

The portable `mining` module owns normalized aimed rays, first-deposit selection,
solid-body obstruction, bounded time-based extraction, and modified-deposit
session state. Runtime provides a character ray and ship obstruction distance,
then applies session deltas to freshly generated candidates for inspection and
presentation. No renderer types, input events, or abstract inventory enter world.

`resource_fragments` receives the actual extracted mass inside `MiningSession`
and owns physical output, stable session IDs and bounded pieces. Its optional
placement callback accepts a new finite pose before debiting output mass; blocked
placement preserves unconsumed mass and identity. Growth never invokes placement. Fragment queries are read-only and independent of deposit
streaming. Runtime maps these domain objects into nearby inspection and authored mesh DTOs.

## Carrying and session lifetime

`carrying.rs` owns the one-object slot and occluded fragment aiming.
`MiningSession` guards pickup/release and validated pose mutation, sealing output
on pickup. Runtime composes hand poses, ship support and fragment motion/contact
with character collision geometry. Generated deposits can stream out and back
in without losing modified mass; fragments remain the same physical entities.
All retention is in memory for the current session, with no disk/backend save.
