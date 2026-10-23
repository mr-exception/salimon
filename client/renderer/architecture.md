# Renderer Architecture

## Responsibility

The renderer is a library boundary around native `wgpu` presentation. It owns
the surface, adapter/device/queue state, surface configuration, shader modules,
render pipelines, command encoding, camera-relative GPU conversion,
renderer-owned projection and depth, generic overlay composition, and frame
presentation for textured spheres, authored ship/resource/item meshes and instrumentation.

The temporary [wgpu-hal backport](../../vendor/README.md) permits the first Metal
drawable before AppKit has reported window visibility. Once the window has been
visible, the dependency retains its normal occlusion protection. This fixes
native startup; it changes no shader, render setting or timing observation.

```text
salimon-world absolute f64 snapshot
    -> runtime maps domain types to renderer DTOs (no rebasing)
    -> Renderer::render(SceneFrame, optional RGBA overlay)
        -> validate absolute f64 camera, cuboids, spheres, and light inputs
        -> load the checked-in ship GLB once, split opaque/glass vertices, update its pose/door uniform, and refresh the instrument atlas only when displayed values change
        -> subtract camera in f64, then cast relative values to f32
        -> conservatively cull cuboid and projected sphere bounds
        -> build renderer-owned view + infinite reverse-Z projection
        -> acquire surface texture
        -> encode depth-tested cuboids + analytic spheres + terrain patches + opaque ship + cockpit glass
        -> store scene depth, then composite overlays in a separate color-load pass
        -> resolve optional timestamp queries asynchronously
        -> submit command buffer
        -> present
        -> return renderer measurements
```

The initialization seam may accept a native window/surface handle, but window
creation, event dispatch, redraw policy, frame timing, and error policy remain in
the runtime. Surface acquisition outcomes flow back to the runtime for delayed
retry, recovery, or shutdown.

## Resize and frame flow

Only nonzero physical sizes configure the surface. A successful render acquires
the current surface texture, prepares and uploads the borrowed scene, records
the validation pass, submits it to the queue, and presents it once. The runtime
owns mapping from portable world structs into `CameraFrame` and `SceneInstance`,
but preserves their absolute `f64` positions. The renderer subtracts camera
position from instance centers and the camera target in `f64`, converts the
relative results to `f32`, and builds the right-handed infinite reverse-Z
projection and depth state.

Runtime emits six `SphereInstance` values with absolute `f64` center/radius,
three marker cuboids, a generic `ShipMeshInstance`, and the Sun mapped into `PointLight`. Generic material
styles identify presentation choices. `SceneFrame` conveys no celestial IDs,
landing volumes, world catalog ownership, or simulation behavior. The GLB loader
expands the checked-in ship mesh once at renderer initialization; material colors
and a door-vertex flag are retained in the GPU vertex stream. Opaque ship
geometry writes reverse-Z depth first. One `Monitor_Wide` quad carries top-left UVs and a
flat display flag. Its fragment branch samples a linearly filtered
2560 × 256 sRGB atlas (three console regions plus Core and Fabricator regions) without a separate draw or light pass.
`CockpitInstruments` carries presentation-only speed, shared power, Core energy,
and optional nearby-body surface-distance/radial values;
`cockpit_instruments.rs` caches rounded display keys before formatting or
rasterizing. Unchanged frames allocate no instrument images and upload no
instrument pixels. One atlas occupies 2.5 MiB on the CPU and GPU each.
The screen remains attached to the ship and obey normal occlusion/free-look.
The double-sided cockpit glass then uses
one alpha-blended draw with depth testing but no depth writes, shadows, sorting,
or post effects.

The scene uses a conservative bounding-sphere frustum test before upload.
Spheres use conservative projected bounds. The combined retained set defines
visible/rendered counters; there is no occlusion culling.

## Precision boundary

Camera-relative conversion preserves local detail around a large absolute origin.
For a sphere, the renderer additionally computes `distance - radius` in CPU `f64`
and uses that altitude in a rationalized near-intersection root. Surface depth
therefore does not reconstruct meters by subtracting two large GPU values.
Body-local texture detail likewise receives a wrapped CPU `f64` origin.
See [sphere-rendering.md](sphere-rendering.md) for equations and limits.

## Diagnostics contract

Diagnostics supplies already-rasterized RGBA pixels and a revision number. The
renderer validates and caches the image, uploads it only when the revision or
size changes, uniformly scales oversized panels to stay inside 16-pixel drawable
margins, and composites them with alpha blending. It never formats metrics or
decides when the overlay is visible.

The scene and overlay passes share one capability-gated timestamp interval. A three-slot
readback ring is polled without waiting, so GPU timing does not synchronously
stall presentation. Scene object/draw counts exclude the engineering overlay;
the total draw count includes its one compositing draw. Optional allocator
reports expose GPU allocated/reserved bytes without claiming process memory.

## Evolution

Future terrain can add small body-local patches with absolute `f64` anchors,
explicit coverage, shared materials, and the same depth convention, retaining
analytic spheres for distant presentation. Preserve the dependency direction: domain
modules produce absolute snapshots, the runtime maps them into presentation
DTOs without doing precision conversion, and the renderer remains a GPU
consumer rather than an authoritative state owner.

## Held item presentation

`held_item.rs` loads the checked-in mining-tool GLB once into an immutable
vertex buffer with positions/normals, base factor, status flag, authored UVs and
metallic/roughness factors (15 `f32`s per vertex). It decodes the single embedded
opaque 256×256 PNG using the bounded `item_image.rs` decoder and uploads an sRGB
atlas with nine linear-color-averaged mip levels once. Unsupported maps/material
contracts fail loading. The fragment shader samples the authored atlas and uses
camera-local fill/highlights with authored metallic/roughness; steel, matte
rubber/polymer and status remain distinct without a new scene pass.
`HeldItemInstance` carries active feedback only; absence stows
the visual. The identity grip and baked vertices rotate into a camera-local
frame through the single `GRIP_TO_VIEW` transform in `held_item.rs`. CPU
preparation composes it with the active camera projection; `held_item.wgsl`
consumes that model-to-clip matrix and uses ordinary reverse-Z depth. Pitch/gravity/world origins do not affect screen placement. Status
material regions switch amber/teal; runtime retains mining gates and the
separate screen-space reticle. The visual adds one scene object and one draw. See the
[mining-tool contract](../../models/assets/items/mining-tool/README.md).

## Centered gameplay overlay

`Renderer::render` accepts an independent borrowed reticle image in addition to
its diagnostics and action-bar images. `OverlayPlacement::Center` maps the image
midpoint to drawable center (NDC zero) using the current surface configuration on
every prepare, including after resize/reconstruction. The existing alpha image
pipeline draws it after the scene with depth comparison Always and no depth
writes. Cached pixels change only with the supplied revision; gameplay equipment
policy stays in runtime. Total draw calls include the visible reticle draw.

## Authored resource fragments

`resource_mesh.rs` validates all 18 checked-in fragment/deposit GLBs once
(baked identity transforms, opaque materials, ±0.48 m local bounds), deduplicates
position/color vertices and retains immutable GPU vertex/index buffers. Authored
material factors and local facet fill remain baked in the vertex colors. The
immutable CPU triangle positions used by collision/support queries are unchanged.

Runtime supplies stable mesh choice, linear tint, absolute center, unit quaternion and
uniform side through `ResourceMeshInstance`. Each frame the renderer validates
poses, subtracts the camera origin in `f64`, then narrows centers to `f32`.
A rotation-independent enclosing sphere (±0.481 cube, slightly inflated for
rounding) conservatively rejects resources outside the near/side frustum planes
before staging. Near-plane crossings and carried fragments use the same path;
culling changes presentation counters only, never picking, physics or session data.

Visible resources are grouped by mesh variant; material roles remain in that
variant's resident geometry. A 48-byte record contains center/scale, quaternion
and opaque linear tint as three `vec4<f32>` instance attributes. WGSL rotates/scales local vertices and
adds the relative center before the existing view/infinite reverse-Z projection.
There is one indexed instanced draw per nonempty mesh: up to 19 including
legacy probes; live fragment-only cargo uses one draw for all resource colors. Draw counts
and triangles reflect the submitted work, not the full gameplay population.

Per-variant CPU staging/last-upload caches and GPU instance buffers retain capacity;
GPU buffers grow geometrically. Only changed visible variant records upload.
Turning the camera changes the uniform/culling; translation or ship motion may
refresh compact instance records but never reuploads mesh geometry. An unchanged
visible batch uploads only the 64-byte camera uniform; a fully changed batch
uploads at most `64 + 48 * visible_count` bytes. The existing scratch counter now
reports retained staging plus last-uploaded byte capacities, not temporary
expanded vertices or total allocator traffic. See the
[benchmark guide](../../scripts/FRAGMENT_BENCHMARKS.md) and
[issue #158 evidence](../../reports/issue-158/README.md).

## Authored water-ice deposits (#88)

Four opaque Blender-authored spire/crown/ridge/shelf exports use the shared
resource mesh batch. Runtime selects `DepositId.local % 4` in that order and
emits no water-ice cuboid. Depleted deposits emit no authored visual. Baked
coordinates fit ±0.48 m; uniform scaling by `bounds_radius_meters /
(0.48 * sqrt(3))` inscribes the visual cube in the authoritative spherical bound
at its absolute deposit center, at every body/latitude. Variant choice ignores
query order, camera, streaming and remaining mass. World generation, mining,
mass, session persistence and streaming rules are unchanged. See the
[asset guide](../../models/assets/resources/water-ice-deposit-spire/README.md).

## Authored silicate deposits (#87)

Four opaque Blender-authored boulder/slab/ridge/scree exports share the resource mesh
batch. Runtime selects `DepositId.local % 4` in that order and emits no silicate
cuboid. Depletion hides the mesh. The centered ±0.48 m baked cube uses uniform
`bounds_radius_meters / (0.48 * sqrt(3))` scaling inside the authoritative sphere.
Identity, mining, mass, session persistence and streaming stay world-owned.
See the [asset guide](../../models/assets/resources/silicate-deposit-boulder/README.md).

## Authored iron deposits (#86)

Four opaque Blender-authored nodule/vein/ledge/rubble exports complete deposit
migration to the shared resource mesh batch. Runtime chooses `DepositId.local % 4`
in that order. All resources now emit authored meshes; no deposit cuboid path
remains. Depletion hides the visual. The centered ±0.48 m baked cube uses uniform
`bounds_radius_meters / (0.48 * sqrt(3))` scaling inside the authoritative sphere.
Identity, targeting, mining, mass, session persistence and streaming stay world-owned.
Automation retains the legacy `visual` field as null and reports mesh/center/scale
through `authored_visual`. See the [asset guide](../../models/assets/resources/iron-deposit-nodule/README.md).

## World-anchored prompts (#127)

`OverlayPlacement::World` carries an absolute `f64` object center and conservative
camera-facing visibility radius in metres. `overlay.rs` subtracts the camera in
`f64` and uses the same prepared view/projection as the scene. Nonfinite,
behind-camera, near-clipped and off-viewport anchors are hidden. The label is
centered 12 physical pixels above the projected anchor, uniformly limited to
60% of drawable width; a rectangle that cannot fit is hidden rather than clamped.
Movement/rotation/resize reprojects each frame without rerasterizing unchanged text.

Scene depth is stored and sampled in a separate color-load overlay pass (no
attached depth target). `overlay.wgsl` compares reverse-Z depth at the anchor
against its camera-facing bound, hiding the entire label behind closer geometry.
The bound avoids self-occlusion at centers inside target meshes; it is a
conservative visibility approximation, not a new interaction range. Transparent
cockpit glass does not write depth. Global overlays skip the depth comparison.
Cached texture pixels/revisions remain independent of placement; depth bindings
are refreshed when surface reconstruction replaces the depth view. Gameplay
eligibility, text, priority and transient expiry remain runtime-owned.

## Equipment toolbar HUD (#129)

Runtime supplies `EquipmentToolbar` (five optional `EquipmentIcon` values plus
an optional typed `EquipmentSlot`) to `Renderer::render`; absence hides the HUD
in precision tour. `equipment_toolbar.rs` owns the compact five-slot raster,
number glyphs, source-rendered PNG tool artwork and selected border/background.
The shared bounded PNG decoder loads the icon once. Visible alpha bounds, rather
than canvas dimensions/fixed silhouette offsets, determine a uniform fit centered
in the slot with four logical pixels of padding. Straight-alpha artwork composites
onto the slot, then numbers draw last. The regression pads/offsets the image
asymmetrically and checks centered artwork at all three raster densities. It reads no
gameplay domains and never derives usability, selection or equipped state.
Raster pixels and GPU uploads are cached by contents, selection and bounded
finite DPI density (1x–2x, matching the existing HUD scale-factor convention).

The toolbar uses its own generic RGBA overlay cache/draw, bottom-centered in the
current physical viewport with uniform fitting on resize. It occupies the lowest
HUD band. Global/transient `BottomCenter` action images are converted to
`BottomCenterInset(fitted toolbar height + scaled gap)` and fit into the remaining
height. At tiny sizes an unfittable message is hidden; it never covers the toolbar.
World prompts keep object-relative placement and depth visibility. Total draw
counts include the toolbar. No inventory/tool behavior lives in this raster.

Resource mesh `support_meters(up)` queries cached validated authored vertices on
the CPU; runtime uses the result for terrain/deck placement. Mesh scale comes
from the world gameplay size policy; material solid volume remains separate.

## Resource batch measurements (#155)

`RenderStats.resource_batch` reports culled instance preparation time, CPU
encoding/queue-enqueue time, changed instance plus projection-uniform bytes,
submitted resource triangles and retained instance staging/upload capacity. These
are resource-batch measurements, not total GPU traffic/allocator totals. Instance
buffer growth is included in preparation; triangles include only visible deposits
and fragments after conservative frustum rejection. Existing GPU
pass readback stays asynchronous and reports the latest completed duration,
which can be older than the current frame. No timing readback waits are added.

The opt-in [#160 geometry probes](../../scripts/FRAGMENT_BENCHMARKS.md#resource-geometry-evaluation-160)
inventory every authored variant and isolate resource-only GPU pass timestamps.
These are evaluation tools; no resource mesh LOD or collision proxy replacement
is enabled by them.

## Bottom-hinged boarding ramp (#169)

The [authored ramp contract](../../models/assets/ships/salimon-scout/README.md#boarding-ramp-169)
owns geometry and generated dimensions. Runtime enables traversal only when the
700 ms animation is fully deployed, supplies movement-envelope collision and
rejects E while the player's body occupies the sweep. Ship's
`toggle_door_with_clearance` rejects an obstructed toggle without reversing the
target state; takeoff retains its closed-state/zero-fraction interlock.
Open-space EVA retains the same flight permission and inherited drift rules;
there is no planetary support on the lowered door in space. Renderer tags the
leaf and tray separately, using one uniform for phased lowering/extension in
the existing opaque batch. Character contact follows the complete sloping
surface, supports a stopped or carrying walker, and joins radial ground at its tip.

## Terrain integration (#179–#181)

Follow the canonical [integration/budgets/validation guide](../../docs/terrain-validation.md)
and [world definition contract](../world/terrain-contract.md). Keep local rules
and their consumers aligned; flat deterministic fixtures are explicit.

Cockpit throttle DTO/cache percentages are signed i8, clamped to −100..+100
for display. The label preserves the sign and the bar shows absolute magnitude
for reverse/forward selection; rasterization never controls ship authority.

## Core power integration (#188)

`CockpitInstruments::power_off` blanks the cached physical display to opaque
black once. Values changing while off cause no uploads; restoration redraws all
three regions. The renderer consumes a power DTO, never derives gameplay power
from rounded stored-joule telemetry.

## Core-local monitor (#194)

`Core_Monitor` is an authored two-triangle, starboard-facing screen in the
energy-core component, distinct from `Monitor_Wide`. The existing opaque draw
samples a cached 2560 × 256 atlas: the console occupies the first three 512 px
regions, the Core the fourth and Fabricator the fifth. No extra draws/materials/pipelines. Both
screens blank once on explicit bus power loss and redraw automatically at
restoration. The Core region independently caches rounded kW, energy, seconds
and warning/conversion state. Runtime maps domain status without deriving rules.

## Vector navigation and braking (#193)

Ship owns the [navigation feasibility contract](../ship/architecture.md#vector-navigation-and-braking-193).
Runtime maps it without deriving rules; renderer formats observation DTOs only.

The existing wide console shows velocity XYZ in ship-local axes (+X forward,
+Y up, -Z starboard), angle, signed throttle/acceleration, kg mass, nearby total
speed and reverse duration/distance/GJ. Unavailable readings use --; insufficient
energy and alignment have explicit status. Rounded strings independently cache
flight/braking/navigation regions. Core-local screen ownership is unchanged.

## Shared spherical fragment (#232)

`ResourceMesh::Sphere` loads one checked-in 80-triangle neutral GLB. A linear
opaque tint is the third vec4 in each 48-byte resource instance record; the
shader multiplies baked facet color by tint. Deposits/legacy probes use white.
All live fragment colors/radii share one resident mesh and one visible batch.
`authored_mesh_scale` is dimensionless uniform asset scale,
radius / `SPHERE_AUTHORED_RADIUS_METERS` (0.48 m) for the shared sphere.
It is independent of analytic gameplay diameter. The legacy inspection JSON key
`authored_visual.side_meters` retains its previous mesh-scale meaning. Runtime
maps mass/density and current camera/tool attachment, never renderer gameplay.

## Procedural Option A style (#278)

`planet_style.rs` holds compact, immutable version/seed/palette recipes indexed by
generic `SurfaceMaterial`; runtime retains celestial mapping. `surface_textures.rs`
generates the shared broad albedo swatches. Terrain's existing triangle positions
supply derivative face normals, smoothly faded with camera distance and existing
geometry morph. GPU patch vertex/uniform layout, depth, caches and world physics
contracts do not change. See the [art bible](../../docs/visual-style.md#procedural-body-and-terrain-recipes-278)
for exact palettes, fades and future external-body configuration constraints.

Terrain upload observations include CPU preparation duration and peak separately
from upload bytes and asynchronous GPU pass timings. This covers validation,
buffer creation and queue writes; it does not measure GPU completion (#307).

`RendererInfo::driver` retains the selected adapter's driver/driver-info strings
for versioned visual-corpus provenance, alongside its name/backend (#308).

## Authored Fabricator module and tray presentation (#342)

The cabin component owns the wall chassis, monitor, input throat and tray, plus
a stationary generic output package. Runtime reads the generated input marker.
The renderer receives only a tray-occupied boolean; the existing ship uniform's
spare component gates the authored output surface in the opaque shader. The
package appears at completion and disappears on collection, with no animation,
new draw or physical fragment identity. It follows the presented ship pose and
remains visible during blackout. Monitor artwork/crafting UI are owned by #343.

Fabricator monitor (#343) uses atlas section 4 (five 512x256 regions total).
The wide cockpit retains sections 0–2 and Core retains section 3. The authored
`Fabricator_Monitor` UV surface samples section 4; its renderer-neutral DTO carries
status, job count and active seconds only. Enlarged recipe UI is runtime RGBA.

Tray category styling (#344) uses the fourth spare ship-uniform component;
occupied state remains the third. The single authored output submesh stays
stationary in ship coordinates, with ration stripes or a medical cross, including
blackout. No loose physics, extra draw call or inventory authority is added.

### Consumable HUD (#346)

The bottom HUD raster reserves two distinct rows: five equipment slots and
three food/two medical slots. Empty category markers stay visible. Amber ration
stripes and a pale package/red cross match the existing low-poly tray styling.
Message inset covers both rows; DPI fitting and caching use the full image.

## Radial consumable presentation (#377)

`consumable_wheel.rs` accepts renderer-neutral slot icons, names, hover index and
effect text. Its cached 480px RGBA donut and native physical-pixel hit test share
sector gaps/radii and uniform center fitting, including resized small windows.
The inner hole/gaps/outside are noninteractive. Empty segments are visibly dim;
original ration stripes/medical crosses match the dedicated HUD and authored
tray. Renderer owns no inventory, consumption, input or survival rules.

## Shared toolbar reservation (#373)

`toolbar_message_inset(surface, dpi)` exposes the same pure fitted physical bottom
band used internally for action text. Runtime uses it to fit the independent
survival overlay without copying inventory raster dimensions or changing existing
slots/art. The renderer still receives only generic borrowed RGBA and placement;
all survival state, effect descriptions, HUD raster and expiry remain runtime-owned.


`overlay_available_size(surface)` (#374) exposes the existing physical screen
margin, shared by overlay fitting and bounded host-produced terminal images.
A host image that exactly fits this size stays unscaled, keeping physical pointer
hit rectangles aligned. Runtime still owns all terminal state, text and input.

## Scout light zoning (#463)

The existing 160-byte ship uniform uses `center.w` for the explicit bus-on bit
from `!instruments.power_off`; energy rounding never determines presentation.
`ship_mesh.wgsl` retains per-vertex diffuse fill, now cool outside and warm in
ship-local cockpit/workshop zones. On bus loss electrical emission extinguishes,
screens retain the existing blank-atlas policy, and a cool non-electrical fill
keeps silhouettes visible. This is an artistic visibility floor, not simulated
emergency energy. No light lists, shadows, new textures, geometry, draws or GPU
allocations are added. Glass alpha, reverse-Z and material base factors remain.
Ordinary metallic/roughness, normal maps, reflections and occlusion/PBR transport
are still unsupported. Engine emission remains a power-only accent; throttle
animation belongs to E02 (#464).

The shared camera contract and native/offline capture commands are documented in
[Scout presentation](../../docs/scout-presentation.md). New images are candidates;
existing reviewed baseline images are never automatically replaced.

## Scout machinery surfaces (#464)

`machinery.rs` owns typed presentation DTOs and named-surface tagging. The ship
vertex stride is 76 bytes; its uniform is now 192 bytes. Power, role, progress,
charge, ramped rudder and active phase drive the existing shader/draws without
new geometry or gameplay dependencies. See the [canonical feedback contract](../../docs/scout-presentation.md#machinery-feedback-boundary-464).

## Engine switch instrument (#493)

`CockpitInstruments::engines_off` is a renderer-neutral observation. The existing
throttle row shows signed selection plus ENG ON/OFF and B hint. Its display cache
invalidates the flight region when the switch changes even at zero throttle;
Core blackout still blanks physical displays. Native readability review is #498.

## Unified Scout UI (#466)

Read the [shared UI contract](../../docs/scout-presentation.md#scout-ui-readability-contract-466)
for the physical screen/prompt/toolbar matrix. `ui_style.rs` is the sole bitmap
face/palette; the runtime reuses it without handing gameplay policy to renderer.
`cockpit_instruments.rs` fits lines within their atlas section at integer density
>=2, makes truncation visible, and renders Fabricator textual warning/action rows
only when that section's key changes. `ship_mesh.rs` explicitly interprets the
current Fabricator U as viewer-right; CPU geometry tests cover all three screens.
No GLB, normal, collider, pipeline, atlas dimension or power authority changes.


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

### Core blackout verification (#451)

The current power-aware shader remains owned by the
[Scout lighting/contrast contract](../../docs/scout-presentation.md#core-contrast-verification-condition-451).
The [dated native report](../../reports/issue-451/README.md) binds current M1
captures to the shader/binary; it does not promote image baselines or change GPU
work, authored materials or simulation authority.

## Oriented marker instances (#542)

Generic `SceneInstance` cuboids now carry an explicitly validated finite unit
quaternion. CPU rebasing still subtracts absolute positions in f64 first; the
64-byte instance stream adds orientation after center/half-extents/color. The
bootstrap shader rotates local scaled vertices before camera-relative projection.
Frustum bounding spheres remain conservative at every rotation. Existing world
markers use identity rotation. Runtime supplies twelve thin cuboids per blueprint
volume, leaving its interior transparent; renderer owns neither reservation nor
completion. No new pipeline or game-engine dependency is introduced.


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
