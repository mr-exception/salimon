# World Architecture

## Responsibility

`salimon-world` owns portable absolute coordinates, the immutable Task 5 body
catalog and geometry helpers, the deterministic Task 4 camera tour, three
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
