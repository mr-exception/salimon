# Renderer AI Maintenance Guide

## Purpose

`salimon-renderer` owns GPU presentation through `wgpu`. It initializes a native
surface and pipelines, responds to valid size changes, renders borrowed
renderer-facing scenes, composites generic RGBA overlay input, and
exposes renderer-owned measurements.

## Read before changing

1. Fetch the active Notion task, Phase 0 specification, and technical
   architecture as required by the root [AGENTS.md](../../AGENTS.md).
2. Read [architecture.md](architecture.md) and [invariants.md](invariants.md).
3. Read the runtime's [maintenance guide](../runtime/README.ai.md) before changing
   the integration contract.

## Public boundary

Keep `Renderer::new`, `Renderer::resize`, and `Renderer::render` plus
`CameraFrame`, `SceneInstance`, `SphereInstance`, `PointLight`, `SceneFrame`, and the typed image/result values
as the narrow host-facing contract unless a task explicitly requires a change.
Window/surface handles needed during initialization are integration inputs; they
do not transfer native lifecycle policy to the renderer.

World snapshots remain absolute `f64` meters. The runtime maps their domain
types into renderer DTOs without rebasing them. During scene preparation, the
renderer subtracts the camera position from instance centers and the camera
target while the operands are still `f64`, then casts the relative results to
GPU-facing `f32`. The renderer also owns the view/projection transform,
reverse-Z depth target, and depth policy.

The renderer owns `wgpu` state, shaders, pipelines, command encoding, overlay
composition, GPU timestamp readback, allocator reporting, and presentation. It
must not own frame scheduling, CPU timing aggregation, input, authoritative
domain state, diagnostics text, or toggle policy.

## Change checklist

- Preserve renderer invariants and runtime error/recovery expectations.
- Keep validation presentation data separate from authoritative world state.
- Prefer typed, renderer-facing frame data over direct domain dependencies.
- Preserve the absolute-`f64` DTO boundary and subtract before casting to
  camera-relative `f32`.
- Capability-gate optional GPU features and preserve explicit
  pending/unsupported results.
- Keep timestamp readback asynchronous; never wait for profiling data in the
  render loop.
- Add tests for pure configuration/selection logic where practical.
- Run all root gates and the native smoke check.
- Update the component docs whenever ownership, API, or recovery behavior
  intentionally changes.

Read [sphere-rendering.md](sphere-rendering.md) before changing spherical
presentation. `spheres.rs` owns conservative screen bounds, CPU `f64` altitude,
material resources, and one instanced analytic draw. `spheres.wgsl` owns stable
ray intersections, surface depth, body-local texture LOD/detail, and lighting.
`surface_textures.rs` is original generated texture source. Keep celestial
identity and landing rules out of all three. Preserve rationalized near-root
depth and the wrapped `f64` detail origin; large `f32` center/radius subtraction
would reintroduce the former proxy's sub-meter surface-position error.
