# Renderer AI Maintenance Guide

## Purpose

`salimon-renderer` owns GPU presentation through `wgpu`. It initializes a native
surface and pipelines, responds to valid size changes, renders borrowed
renderer-facing scenes, composites generic placed RGBA overlay inputs, and
exposes renderer-owned measurements. World-placed overlays use the scene camera
and stored scene depth; keep projection/occlusion out of runtime gameplay policy.

## Read before changing

1. Fetch the active GitHub issue, affected project specification, and technical
   architecture as required by the root [AGENTS.md](../../AGENTS.md).
2. Read [architecture.md](architecture.md) and [invariants.md](invariants.md).
3. Read the runtime's [maintenance guide](../runtime/README.ai.md) before changing
   the integration contract.

## Public boundary

Keep `Renderer::new`, `Renderer::resize`, and `Renderer::render` plus
`CameraFrame`, `SceneInstance`, `SphereInstance`, `ShipMeshInstance`, `HeldItemInstance`, `CockpitInstruments`, `PointLight`, `SceneFrame`, and the typed image/result values
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
domain state, diagnostics/action text, expiry, or toggle policy.

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

`ship_mesh.rs` owns the checked-in ship GLB parser, immutable opaque/glass vertex
buffers, camera-relative pose uniform, door visual offset, basic material
lighting, and the inexpensive depth-tested cockpit-glass blend pass. The three
named monitor quads retain UVs and a panel index in the immutable vertex stream.
`cockpit_instruments.rs` formats typed speed/power, Core energy, and optional
nearby-body DTO values into one cached
512 × 768 RGBA atlas; upload it only when the displayed values change. Screens
are self-lit surfaces in the existing opaque draw and stay live when unseated.
Keep gameplay interaction and ship state out of these modules.

## Shared maintenance rules

Follow the root [coding conventions](../../docs/coding-conventions.md),
[validation matrix](../../docs/validation.md) and
[feature map](../../docs/maintenance-map.md). Update affected contracts/guides
with behavior changes and record completion evidence under root `reports/`.

`held_item.rs` / `held_item.wgsl` own static authored mining-tool loading,
camera-local grip placement, embedded PNG surface/UV sampling, material shading
and status-region feedback. `item_image.rs` bounds PNG decoding for the held
atlas and derived toolbar icon. Runtime owns equip
and active gates; the asset README defines the baked grip contract.

`equipment_toolbar.rs` owns HUD rasterization/cache from `EquipmentToolbar`,
`EquipmentIcon` and `EquipmentSlot`. Runtime supplies visibility/state/DPI;
renderer owns bottom-band layout and message stacking through `overlay.rs`.
See [toolbar contract](architecture.md#equipment-toolbar-hud-129) and CPU tests.
