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

The Scout starter layout retains one opaque plus one glass draw. Its embedded
asset-budget regression decodes the canonical
[ship manifest](../../models/assets/ships/salimon-scout/manifest.json), rather
than duplicating triangle/byte limits in Rust. Starter cargo and habitation
views belong to the opt-in `scout_feedback_gpu_evidence` capture fixture; keep
its source/binary provenance and frozen-repeat checks with the originals.

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
Archive work/progress text remains runtime-owned. Cloud route #515 includes
original GPU frames and an import-only package UV correction: `Fabricator_Output`
uses normalized per-face coordinates selected by its flat normal and transformed
bounds. Authored geometry/exports stay unchanged. World-X/Z UVs previously missed
all normalized circuit/food/medical artwork and collapsed vertical faces. The
loaded-geometry regression requires [0,1] coordinates and noncollapsed full-face
triangles. Human appearance is #561, physical input/DPI #557 and budgets #563.


## Base habitation (#540)

The bunk's two authored panels use the generated
`habitation_geometry.rs` hinge/swept bounds, sourced from
[the category definition](../../models/assets/ships/salimon-scout/habitation-geometry.json).
The renderer maps a finite stowed/deployed fraction to rigid rotation and normals;
runtime admits manual motion only after full player/belonging clearance. Static
poses cost no repeated clearance scan. Reserved walking clearance is unchanged.
Galley E opens the existing exact-identity consumable wheel, with no generated
food or additional inventory; hygiene remains an inspectable base fixture.
No water/hygiene meter, sleep-time advancement or additional survival benefit is
introduced. Sitting/lying, privacy mechanisms and expanded tactile presentation
remain #577. Schema 10 saves source motion; see
[the save contract](../../docs/save-contract.md#habitation-schema-10-540).


## Modular interior fixture library (#575)

Renderer `fixtures.rs` loads immutable named templates once, batches by mesh,
rebases world positions in f64 before narrowing and validates identities, finite
unit poses and bounded tint. `FixtureNode` resolves root/room/furnishing/attachment
parents in deterministic input order using a linear queue; missing parents,
cycles and duplicates fail. Physics/collision and persistent condition remain
caller-owned. The 16,384-record renderer staging guard fails explicitly; it
neither limits physical populations nor silently drops submitted instances.

The existing berth migrates through
[interior layout v1](../assets/ship/interior-layout-v1.json) without changing
schema-10 gameplay source, identities, anchors, materials, mass or animation state.
The original whole-ship path remains available for legacy/reference DTOs. Native
composition selects independent presentation and hides only the old paired berth
surfaces; opaque fixture batches draw before cockpit glass. Template instances
share vertex/index buffers and compact 48-byte pose/tint/power records. Per-batch
counts, allocation capacities and CPU preparation are read-only observations.
The prototype Shelf/Seat/Panel/PanelDamaged templates demonstrate architecture;
they do not install furniture or invent physical damage, construction or aging.
See [fixture ownership/budgets](../../docs/modular-interior-fixtures.md).


Persistent finishes #579: [canonical ownership, schema 13, bounded render factors and controls](../../docs/room-finishes.md). Ship owns room/item cosmetic identity; runtime owns nearby physical control admission; renderer applies the explicit nonfunctional surface allowlist. Cosmetic changes never edit physical material/age/damage.


Room and fixture lights #580: [canonical controls, source, effective output and schema-16 contract](../../docs/room-lighting.md). Master/blackout/inoperable lamps suppress output without changing preferences; critical machine displays remain independent.


## Cosmetic surface age (#581)

Follow the [persistent exposure, schema-23 and clean-repair contract](../../docs/surface-age.md). Ship owns bounded integer exposure; runtime admits physical identity/use and verified offline time; renderer consumes observations only. Recolor/repair retain age, replacements use fresh IDs, and no cosmetic state grants damage or collision authority.


Pipeline ownership (#643): use `src/initialize.rs` for device/pipeline creation,
`src/surface.rs` for resize/configuration and `src/frame.rs` for ordered draw,
capture, submission and observations. Keep facade/resource ownership in
`src/lib.rs`. Follow [pipeline invariants](architecture.md#pipeline-and-frame-owners-643).


Resource geometry (#644): pure GLB validation and stable index packing live in
`src/resource_mesh/geometry.rs`. Keep GPU staging/upload/culling in the parent;
see [resource ownership](architecture.md#resource-geometry-owner-644).


## Canonical terrain masks (#424)

Terrain preparation passes the caller's validated mask radius to the existing
shader instead of hard-coding the 4,096 m fixture cap. Canonical whole-planet
meshes retain published relief at orbital LOD; old fixture mask radii and shader
layouts are unchanged. See [canonical terrain](../../docs/universe-terrain.md).


## Bounded navigation prediction (#421)

Follow the [prediction/telemetry contract](../../docs/universe-navigation.md): ship owns cloned real-scale coasting and current-controller previews, runtime owns disposable observation caching, renderer owns instrument DTO/rasterization only. Preserve explicit model assumptions, bounded horizon, sampled contact/arrival semantics, empirical convergence (not a certified error bound), visible validity/age and nonmutation of controls, power, source saves and simulation clocks.


## Scanner delivery (#430)

Renderer `scanner_hologram.rs` owns bounded depth-tested ship-local 3D projection geometry; runtime `session/scanner/presentation.rs` maps only fresh same-frame discoveries to ship pose. The physical console projector shows logarithmic distance, true 3D direction/elevation and selected contacts. M cycles contacts only in an admitted canonical cockpit; power loss suppresses projection, not history.
See the [scanner contract](../../docs/universe-scanner.md) for source/test mapping, units and limitations.


## Scanner delivery (#469)

Universe `scanner/chart.rs` owns player-known observations, inferred/transmitted leads with explicit uncertainty, and immutable voluntary route annotations. Runtime `session/chart.rs` owns cockpit P/M/Enter controls and schema-26 pair validation with scanner history. Catalog truth never seeds the chart; observed knowledge outlives contact TTL and source frames are preserved without guessed transforms.
See the [scanner contract](../../docs/universe-scanner.md) for source/test mapping, units and limitations.


## Geological surface materials (#613)

Follow the [geological material contract](../../docs/geological-materials.md) for terrain-bound
profile mapping, immutable material cache keys, the 52-byte vertex layout,
filtered stratification and original software-Vulkan evidence. Runtime owns
geology-to-DTO mapping; renderer owns shading only. Vesper and resource palettes,
physical queries, geometry and save identities are preserved.


## Hybrid astronomical sky (#423)

See the [sky contract](../../docs/universe-sky.md) for bounded presentation,
split-origin/source validation, explicit frame/epoch limits and production
verification. Sky visibility grants no scanner/chart knowledge or simulation authority.


## Optical atmosphere (#614)

The [atmosphere contract](../../docs/planetary-atmosphere.md) maps immutable climate
optics, runtime light inputs and depth-limited production scattering. Vacuum
remains empty; presentation never changes gas physics, weather or canonical identity.
