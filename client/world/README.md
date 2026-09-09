# World

`salimon-world` owns the portable Phase 0 large-scale coordinate and camera
prototype. It keeps authoritative positions in `f64` meters around a validation
anchor near `1e12` meters, drives a deterministic camera tour from 120,000 km to
12 m, and exposes immutable cuboids that make precision and depth failures easy
to see. The range and content are validation-only; Task 5 owns real Solar System
data and scene content.

The crate has no windowing or GPU dependency. A host advances `CameraPrototype`
with a monotonic duration, translates `CameraCommand` values from platform input,
and reads a borrowed `PrototypeSnapshot`. That snapshot exposes the camera and
primitive centers as absolute `f64`-meter values. The runtime maps those domain
structs into renderer DTOs without rebasing them; the renderer owns the
subtract-before-cast conversion as well as projection and depth.

The portable commands can pause/resume the tour, restart it at the far endpoint,
or jump to the exact 12 m near-surface dwell and pause for deterministic visual
inspection.

```text
world absolute f64 snapshot
    -> runtime type mapping (absolute f64 preserved)
    -> renderer subtracts f64 camera origin
    -> renderer casts local result to GPU f32
    -> renderer-owned view/projection and reverse-Z depth
```

`WorldPosition::camera_relative_f32` remains a portable numeric helper for
tests and precision reporting; it is not the runtime-to-renderer handoff.

See [coordinate-strategy.md](coordinate-strategy.md) for measured numeric limits,
[architecture.md](architecture.md) for dependency boundaries, and
[invariants.md](invariants.md) before changing coordinate behavior.
