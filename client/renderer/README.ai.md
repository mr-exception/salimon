# Renderer AI Maintenance Guide

## Purpose

`salimon-renderer` owns GPU presentation through `wgpu`. Task 2 intentionally
keeps it small: initialize a native surface and pipeline, respond to valid size
changes, and render a WGSL bootstrap triangle.

## Read before changing

1. Fetch the active Notion task, Phase 0 specification, and technical
   architecture as required by the root [AGENTS.md](../../AGENTS.md).
2. Read [architecture.md](architecture.md) and [invariants.md](invariants.md).
3. Read the runtime's [maintenance guide](../runtime/README.ai.md) before changing
   the integration contract.

## Public boundary

Keep `Renderer::new`, `Renderer::resize`, and `Renderer::render` as the narrow
host-facing operations unless a task explicitly requires a contract change.
Window/surface handles needed during initialization are integration inputs; they
do not transfer native lifecycle policy to the renderer.

The renderer owns `wgpu` state, shaders, pipelines, command encoding, and
presentation. It must not own frame scheduling, simulation timing, input,
authoritative domain state, or diagnostics policy.

## Change checklist

- Preserve renderer invariants and runtime error/recovery expectations.
- Keep bootstrap presentation data separate from future world state.
- Prefer typed, renderer-facing frame data over direct domain dependencies.
- Add tests for pure configuration/selection logic where practical.
- Run all root gates and the native smoke check.
- Update the component docs whenever ownership, API, or recovery behavior
  intentionally changes.
