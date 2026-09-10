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
    -> runtime maps domain structs into renderer SceneInstance values
       and diagnostics BodyDistance values
    -> renderer subtracts camera in f64 and casts relative values to f32
```

The catalog is a static ordered slice. `CelestialBody` carries stable identity,
name, absolute center, compressed radius, display color, and `BodyRole`.
Geometry helpers calculate center distance, signed nominal-surface separation,
optional solid landing-volume separation, and nonnegative point-to-surface
distance. `VisualOnly` deliberately makes the Sun's landing radius absent.

The runtime is the composition root and translates `winit` events into
`CameraCommand`. The renderer does not depend on this crate. It receives generic
renderer DTOs produced by field-by-field runtime mapping: a body becomes a
radius-scaled colored cuboid for Task 5, while precision markers remain separate
instances. Task 6 owns textured/scalable spheres and Sun lighting.

`WorldPosition::camera_relative_f32` documents and tests the numeric conversion
order but does not move presentation conversion into world. Projection matrices,
clip-space conventions, depth targets, and reverse-Z policy remain renderer
concerns.
