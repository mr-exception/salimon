# Renderer Invariants

1. The surface is configured and rendered only with a nonzero physical width and
   height.
2. The active surface configuration matches the latest accepted drawable size.
3. `wgpu` device, queue, surface, pipeline, shader, and presentation resources
   remain owned by the renderer boundary.
4. One successful `render` call submits and presents one validation-scene frame
   and returns measurements for that presented work.
5. Surface acquisition outcomes are returned to the runtime; the renderer does
   not exit the application or own lifecycle recovery or retry policy.
6. Rendering never advances simulation, mutates authoritative world/game state,
   or consumes raw gameplay-domain internals.
7. `CameraFrame`, `SceneInstance`, `SphereInstance`, `ShipMeshInstance`, and `PointLight` are renderer DTOs, not authoritative world
   state. The runtime maps domain snapshots into them while preserving absolute
   `f64` positions; it does not rebase those positions.
8. The renderer subtracts camera position from instance centers and camera
   target in `f64` before converting relative values to GPU-facing `f32`.
9. View/projection construction and depth policy are renderer-owned. The scene
   path uses a right-handed infinite reverse-Z projection and matching depth
   target/comparison state.
10. Cuboids, analytic spheres, generic materials, and point lighting exercise
    presentation only, with no celestial identity, landing rules, or world
    dependency. Sphere surface altitude is subtracted in `f64` before GPU
    conversion; its fragment depth uses the rationalized ray-intersection root.
11. The renderer uses low-level `wgpu` directly and must not introduce a full
    game engine.
12. Overlay inputs are validated, borrowed RGBA images with typed screen-space
    or absolute-world placement. The renderer may cache, uniformly fit, and composite them but
    never owns diagnostics aggregation, action text, expiry/toggle policy, or
    gameplay data.
13. GPU timestamps are requested only when the selected adapter supports them;
    readback must not block the presentation loop. Unsupported or pending data
    is reported explicitly.
14. Scene object counts describe instances retained by conservative frustum
    culling, and scene draw counts exclude overlay presentation. Total draw calls
    include each diagnostics/action/reticle overlay when it is visible.
15. Texture LOD varies continuously with projected footprint. Local detail uses
    a body-relative origin wrapped in CPU `f64`; no universe-scale `f32` texture
    coordinates, atmosphere, clouds, shadows, or post-effect dependency is used.
16. The checked-in ship GLB is validated and loaded by the renderer without a
    dependency on ship or character behavior. Pose and door state cross only as
    generic presentation data and remain camera-relative on the GPU.
17. Cockpit glass is the ship's only blended material. It renders after opaque
    ship geometry with reverse-Z depth testing and no depth writes, shadows, or
    post effects, preserving exterior visibility from either side.
18. Ship vertices use the authored scale baked into the checked-in GLB; no
    compensating runtime mesh scale is applied. Door lowering and tray extension
    use the authored hinge, angle and travel exported into `ramp_geometry.rs`.

19. Cockpit instruments consume only renderer-facing speed/power, Core-energy,
    and optional nearby-body DTO values. One wide `Monitor_Wide` quad shows
    flight, Core energy and navigation simultaneously using one cached texture
    in the existing opaque ship draw. The shared thruster command appears once.
    Only changed displayed values rebuild their regions and upload the texture;
    no target renders an explicit out-of-range state. UVs use top-left (0, 0);
    the one named surface is exactly two triangles.

20. Center-placed overlays use the current drawable dimensions, place their
    midpoint at NDC (0, 0), and ignore scene occlusion. The renderer consumes
    image/revision/placement only, never equipment or targeting state.

21. Held mining-tool geometry is the checked-in authored GLB, grip-centered and
    baked with identity node transforms. Camera-local presentation preserves
    orientation through look/gravity changes and never supplies a gameplay ray.
    Its embedded surface atlas/UVs and metallic/roughness factors reach the
    fragment shader; unsupported texture/material contracts fail asset loading.
    Active feedback changes only status regions; normal reverse-Z depth applies.

22. World prompt anchors are rebased in `f64` and projected with the scene camera.
    Invalid, behind-camera, clipped or non-fitting labels are hidden, never
    clamped to screen edges. The separate overlay pass samples stored reverse-Z
    scene depth at the anchor to hide an entire occluded prompt. Its conservative
    radius avoids target self-occlusion and never changes gameplay eligibility.

23. Equipment toolbar contents/selection cross as renderer-neutral DTOs. Renderer
    only rasterizes/composites them; five slots remain visible even when all are
    empty. Absence hides the toolbar. Resize/DPI recomputes readable fitting, and
    bottom-center global/transient messages stack above its reserved band.
    Mining icon placement uses visible alpha bounds so asymmetric canvas padding
    cannot shift artwork from the slot center; glyphs/borders remain legible.

24. Authored resource vertex/index geometry is immutable on the GPU. Resources
    upload camera-relative 48-byte pose/tint records grouped by stable mesh variant;
    draw calls scale with visible variants, not fragments. Conservative enclosing
    sphere culling affects rendering only. Material colors/facet fill, unit-pose
    validation, subtract-before-cast precision and reverse-Z remain intact.

## Terrain integration (#179–#181)

Follow the canonical [integration/budgets/validation guide](../../docs/terrain-validation.md)
and [world definition contract](../world/terrain-contract.md). Keep local rules
and their consumers aligned; flat deterministic fixtures are explicit.

- Explicit Core power loss blanks the cached cockpit atlas; restoration redraws
  every region. Rounded energy telemetry is not a power authority.

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

- Shared sphere resource instances use finite [0,1] linear RGB and opaque alpha;
  the 48-byte CPU upload layout matches three WGSL vec4 instance attributes.
  Resource identity/color changes never duplicate the shared sphere geometry.

## Authored Fabricator module and tray presentation (#342)

The cabin component owns the wall chassis, monitor, input throat and tray, plus
a stationary generic output package. Runtime reads the generated input marker.
The renderer receives only a tray-occupied boolean; the existing ship uniform's
spare component gates the authored output surface in the opaque shader. The
package appears at completion and disappears on collection, with no animation,
new draw or physical fragment identity. It follows the presented ship pose and
remains visible during blackout. Monitor artwork/crafting UI are owned by #343.

The Fabricator display occupies only atlas region 4. Cockpit regions 0–2 and
Core region 3 retain their independent cache/UV mappings. Machine observation
never changes recipe, queue, power or inventory authority.

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

The exported toolbar message inset (#373) and internal message placement use the
same scale/fitting implementation. Runtime HUDs may reserve this band, but renderer
must never inspect their gameplay state or own their animation/expiry.


`overlay_available_size(surface)` (#374) exposes the existing physical screen
margin, shared by overlay fitting and bounded host-produced terminal images.
A host image that exactly fits this size stays unscaled, keeping physical pointer
hit rectangles aligned. Runtime still owns all terminal state, text and input.

Scout lighting (#463) consumes the same explicit bus state as the physical
screens. Blackout suppresses electrical emission and warm fill, preserving a
cool visibility floor without adding power/gameplay authority. Glass blending,
depth, immutable geometry and the 160-byte uniform size remain unchanged.

Scout machinery (#464) consumes finite normalized immutable feedback only; no
simulation/clock/save authority. The current ship uniform is 192 bytes and vertex
stride 76 bytes. Named role tags cannot animate structural supports, hull, ramp
or colliders. Electrical cues obey explicit bus state; tray output remains
independent of power. The original #463 layout statement above describes its
foundation; #464 extends that layout.

Engine ON/OFF observation (#493) is independent of selected throttle and Core
blackout. A switch change invalidates the cockpit flight display cache without
mutating gameplay; native appearance acceptance remains #498.


Scout UI #466: one presentation-only font/palette, case-preserving SI readings,
explicit textual warning and truncation, and viewer-right/top-left physical screen
orientation. No display text may spill into adjacent atlas sections. The current
Fabricator import-only U correction is deliberate and must be reconciled with
any future authored UV change. Native acceptance remains #502; future scanner
integration remains #503 after #430.


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

## Physical mounts (#576)

Physical rail, clamp and shelf instances retain stable caller IDs. Secured state is supplied by runtime; the renderer cannot create inventory or locks. See [the canonical contract](../../docs/physical-mounts.md).
