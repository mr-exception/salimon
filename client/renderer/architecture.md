# Renderer Architecture

## Responsibility

The renderer is a library boundary around native `wgpu` presentation. It owns
the surface, adapter/device/queue state, surface configuration, shader modules,
render pipelines, command encoding, camera-relative GPU conversion,
renderer-owned projection and depth, generic overlay composition, and frame
presentation for textured spheres, authored ship/resource/item meshes and instrumentation.

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
        -> encode depth-tested cuboids + analytic spheres + opaque ship + cockpit glass
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
geometry writes reverse-Z depth first. Three monitor quads carry UVs and a
flat panel index. Their fragment branch samples a shared, linearly filtered
512 × 768 sRGB atlas without a separate draw or light pass.
`CockpitInstruments` carries presentation-only speed, shared power, Core energy,
and optional nearby-body surface-distance/radial values;
`cockpit_instruments.rs` caches rounded display keys before formatting or
rasterizing. Unchanged frames allocate no instrument images and upload no
instrument pixels. One atlas occupies 1.5 MiB on the CPU and GPU each.
The screens remain attached to the ship and obey normal occlusion/free-look.
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

`resource_mesh.rs` loads checked-in water-ice, silicate and iron fragment GLBs at initialization and
validates their baked identity transforms and ±0.48 m local bounds. Runtime maps
stable variant choice, absolute center and authoritative side into
`ResourceMeshInstance`; the renderer subtracts camera position in `f64` before
narrowing scaled vertices. All authored variants share one opaque reverse-Z draw with
flat material colors and inexpensive local facet shading. No transparency,
textures or gameplay-domain dependency is added. Geometry buffers grow only
when needed; diagnostics count individual resource instances and the single batch draw.

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
