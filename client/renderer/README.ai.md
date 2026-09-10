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
`CameraFrame`, `SceneInstance`, `SceneFrame`, and the typed image/result values
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

Task 5 body proxies are generic cuboids produced by runtime; do not add body
identity, landing rules, or Solar System dependencies here. Task 6 owns the
explicit sphere and Sun-lighting presentation path.

The radius-scaled Earth proxy is an intentional stress case, not evidence of
sub-meter precision across one huge mesh. Reconstructing its near face from a
center and half-extent near `6 Mm` in GPU `f32` uses `0.5 m` representable steps,
so rounding can contribute up to about `0.25 m` of face-position error. Small
markers near the camera still retain their local camera-relative precision.
