# Renderer AI Maintenance Guide

## Purpose

`salimon-renderer` owns GPU presentation through `wgpu`. It initializes a native
surface and pipelines, responds to valid size changes, renders the WGSL
bootstrap triangle, composites generic RGBA overlay input, and exposes
renderer-owned measurements.

## Read before changing

1. Fetch the active Notion task, Phase 0 specification, and technical
   architecture as required by the root [AGENTS.md](../../AGENTS.md).
2. Read [architecture.md](architecture.md) and [invariants.md](invariants.md).
3. Read the runtime's [maintenance guide](../runtime/README.ai.md) before changing
   the integration contract.

## Public boundary

Keep `Renderer::new`, `Renderer::resize`, and `Renderer::render` plus their typed
image/result values as the narrow host-facing operations unless a task
explicitly requires a contract change.
Window/surface handles needed during initialization are integration inputs; they
do not transfer native lifecycle policy to the renderer.

The renderer owns `wgpu` state, shaders, pipelines, command encoding, overlay
composition, GPU timestamp readback, allocator reporting, and presentation. It
must not own frame scheduling, CPU timing aggregation, input, authoritative
domain state, diagnostics text, or toggle policy.

## Change checklist

- Preserve renderer invariants and runtime error/recovery expectations.
- Keep bootstrap presentation data separate from future world state.
- Prefer typed, renderer-facing frame data over direct domain dependencies.
- Capability-gate optional GPU features and preserve explicit
  pending/unsupported results.
- Keep timestamp readback asynchronous; never wait for profiling data in the
  render loop.
- Add tests for pure configuration/selection logic where practical.
- Run all root gates and the native smoke check.
- Update the component docs whenever ownership, API, or recovery behavior
  intentionally changes.
