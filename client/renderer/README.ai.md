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
as the host-facing contract. TerrainPatch/Key/Vertex/Mask extend it for immutable
streamed geometry; explicit capture and cache observations support native evidence.
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
lighting, and the inexpensive depth-tested cockpit-glass blend pass. The single
`Monitor_Wide` quad retains top-left UVs and a display flag in the immutable vertex stream.
`cockpit_instruments.rs` formats typed speed/power, Core energy, and optional
nearby-body DTO values into one cached
2560 × 256 RGBA atlas (three console regions plus Core and Fabricator regions); upload it only when the displayed values change. The screen
is a self-lit surface in the existing opaque draw and stay live when unseated.
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

`resource_mesh.rs` / `resource_mesh.wgsl` own immutable authored resource geometry,
per-variant indexed instancing, pose validation, conservative resource culling and
retained compact upload caches. CPU unit triangles remain available independently
for collision/support queries. See [resource contract](architecture.md#authored-resource-fragments)
before changing instance layout, material colors, precision or counters.

## Terrain integration (#179–#181)

Follow the canonical [integration/budgets/validation guide](../../docs/terrain-validation.md)
and [world definition contract](../world/terrain-contract.md). Keep local rules
and their consumers aligned; flat deterministic fixtures are explicit.

## Core-local monitor (#194)

`Core_Monitor` is an authored two-triangle, starboard-facing screen in the
energy-core component, distinct from `Monitor_Wide`. The existing opaque draw
samples a cached 2560 × 256 atlas: the console occupies the first three 512 px
regions, the Core the fourth and Fabricator the fifth. No extra draws/materials/pipelines. Both
screens blank once on explicit bus power loss and redraw automatically at
restoration. The Core region independently caches rounded kW, energy, seconds
and warning/conversion state. Runtime maps domain status without deriving rules.


`toolbar_message_inset(surface, dpi)` is the pure shared bottom-band layout seam
for runtime HUD producers (#373). Keep it consistent with `ToolbarRaster` message
stacking; it introduces no renderer dependency on survival or other domains.


`overlay_available_size(surface)` (#374) exposes the existing physical screen
margin, shared by overlay fitting and bounded host-produced terminal images.
A host image that exactly fits this size stays unscaled, keeping physical pointer
hit rectangles aligned. Runtime still owns all terminal state, text and input.

Scout machinery #464: read the [canonical feedback boundary](../../docs/scout-presentation.md#machinery-feedback-boundary-464). Runtime maps snapshots in `scout_feedback.rs`; renderer owns `machinery.rs`/ship roles/shader. Preserve pause/restore and power/work gating tests; run the opt-in offscreen GPU evidence check for shader changes.


Scout UI #466: `ui_style.rs` owns the shared font/palette; runtime prompts reuse
it. Read the [canonical matrix](../../docs/scout-presentation.md#scout-ui-readability-contract-466)
before changing text, key cues, line fitting or physical screen orientation.
Preserve SI case, non-color warning text and all three screen-orientation tests.


## Archive terminal (#467)

`archive_terminal.rs` adds a flush +Z-facing cabin panel and screen (12 vertices)
to the existing ship draw. The runtime uses its published physical center for
interaction. UVs are viewer-right/top-left; the sixth 512x256 region extends the
instrument atlas to 3072x256 without changing the first five region dimensions.
The shader uses section indices, blanks all six regions on power loss and restores
them from the cached DTO. No new draw/pipeline, collision or simulation authority.
See [runtime contract](../runtime/architecture.md#scout-archive-467).

## Recovery component tray presentation (#468)

`ShipMeshInstance::fabricator_output_component` is an observational flag for the
existing anchored tray package. Its existing uniform lane encodes component=2,
medical=1 or food=0; `ship_mesh.wgsl` draws grey/cyan circuit traces for components.
No buffers/draws/colliders, installation authority or resource state are added.
Archive work/progress text remains runtime-owned. Native appearance/input/budget
acceptance is #515; CPU shader/layout contracts cannot establish it.
