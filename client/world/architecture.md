# World Architecture

## Responsibility

`salimon-world` owns portable absolute coordinates, the deterministic Task 4
camera tour, validation primitive definitions, and numeric precision reporting.
It does not own platform input, frame scheduling, GPU buffers, projection
matrices, depth textures, or presentation policy.

```text
runtime monotonic delta + typed CameraCommand
    -> salimon-world CameraPrototype
        -> borrowed PrototypeSnapshot (f64 camera + absolute primitives)
    -> runtime maps domain structs into renderer frame DTOs
        -> absolute f64 positions remain unchanged
    -> renderer subtracts camera in f64 and casts relative values to f32
        -> renderer-owned projection, depth, and drawing
```

The runtime is the composition root and translates `winit` events into
`CameraCommand`. The renderer does not depend on this crate. It receives typed
renderer DTOs produced by field-by-field runtime mapping, not world domain
types. That mapping preserves absolute coordinates; the renderer performs the
double-precision subtraction before narrowing relative values for the GPU.

`WorldPosition::camera_relative_f32` documents and tests the required numeric
order inside this portable crate, but does not move presentation conversion into
the runtime or make the world crate responsible for GPU preparation. Projection
matrices, clip-space conventions, depth targets, and reverse-Z policy remain
renderer concerns.

## Evolution

Task 4 data is deliberately small and static. Task 5 may replace validation
primitives with real compressed Solar System data while preserving `f64` meters,
explicit renderer-facing conversion boundaries, and the world-to-presentation
dependency direction.
