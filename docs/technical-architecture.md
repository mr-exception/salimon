# Technical architecture and AI maintenance

## Implemented architecture

Salimon uses a custom Rust runtime with low-level `winit` and `wgpu`, without a
full game engine. The current prototype extends the Phase 0 baseline and remains native and
client-only. Local build scripts stage macOS, Windows and Debian-based Linux
executables. PR CI compiles the workspace once on Ubuntu. Separate nightly
verification and weekly benchmarks/platform builds publish reports; all local
commands remain available. Native macOS is the reference playable/performance target. Web/server reuse remains a future capability, not a shipped target.

The [feature maintenance map](maintenance-map.md) routes work to source, tests,
invariants and scenarios. The [client guide](../client/README.md) defines package
dependencies; each crate's architecture/invariants defines its local contracts.

| Boundary | Implemented ownership |
| --- | --- |
| `client/runtime/` | Native shell/lifecycle/evidence IO plus portable GameplaySession commands, snapshots, stepping, resource/cargo adapters and presentation mapping |
| `client/world/` | Static six-body compressed catalog, `f64` coordinates/camera tour, resource contracts, generation, extraction/session deltas, fragments and one-object carrying |
| `client/ship/` | Pose, cockpit authority, inertial force-based flight, uncancellable landing/takeoff, airlock interlocks and live telemetry |
| `client/physics/` | Portable small-object gravity, ejection/release velocity, authored convex contacts and angular motion, restitution and deterministic substeps; caller-supplied surfaces/geometry |
| `client/math/` | Dependency-free `f64` vector component arithmetic; no domain/frame ownership |
| `client/character/` | Portable movement, collision, gravity, cockpit transitions and open-space/nearby-body EVA |
| `client/renderer/` | GPU resources, GLB loading, sphere/ship/image pipelines, reverse-Z, instruments and measurements through renderer-neutral DTOs |
| `client/diagnostics/` | Public DTO/facade plus private aggregation, formatting and CPU rasterization modules; see [source owners](../client/diagnostics/architecture.md#private-source-owners) |
| `models/` and `client/assets/` | Offline authored sources/contracts and checked-in validated runtime exports respectively |
| `client/platform/` and `core/` | Reserved adapter/backend boundaries; no backend implementation |

Runtime depends on supporting crates and maps snapshots into presentation DTOs;
supporting crates do not call runtime. Character has no ship-crate dependency;
ship may consume world contracts. Renderer does not depend on authoritative
world, character or ship state. Native window/input types stay at the composition
boundary; GPU types stay in renderer. Typed Rust contracts currently provide
module boundaries, not dynamically loaded WASM components.

Implemented gameplay includes surface walking, ship flight/assists, airlock/EVA,
resource mining, one-fragment carrying, planet/ship transfer and custom fragment
motion/contact. Mining session deltas and physical fragments survive nearby
streaming **in memory for the current session**. This is not disk/backend
persistence. The canonical [reload verification scope](../client/world/resource-contracts.md#persistence-and-reload-verification-scope-268)
defines in-session restoration and the proposed #236 scope clarification awaiting
owner approval; relaunching starts a fresh session. No orbital simulation,
multiplayer or production survival is implemented.
Core joule storage, watt consumers and power authority are implemented; assist reservations are implemented; physical Core intake/conversion and a distinct Core-local status display are implemented.

## Accepted constraints

- No full game engine. Keep low-level libraries and explicit domain ownership.
- Portable domain source uses typed commands, validated state and snapshots;
  platform capabilities and presentation do not leak into domain contracts.
- CPU absolute coordinates remain `f64` meters. Subtract camera origin in `f64`
  before narrowing to GPU `f32`; never subtract universe-scale `f32` positions
  in WGSL. Renderer uses infinite-far reverse-Z `Depth32Float`, clear 0 and
  greater-depth comparison with an explicit near plane. See the
  [coordinate strategy](../client/world/coordinate-strategy.md).
- World/runtime axes are right-handed +Y up. Ship local +X is forward and -Z
  starboard. Surface gravity uses the selected body's local radial direction,
  not global +Y. Blender source is +Z up; export converts once. See
  [asset contracts](../models/contracts.md).
- The static catalog and tour are compressed validation fixtures, not realistic
  orbital simulation. Follow [catalog invariants](../client/world/invariants.md)
  rather than treating tour tuning as a product requirement.
- Preserve deterministic identities/update semantics, one-object carrying and
  uncancellable assists. Changes must update owning invariants and tests.
- Profile before optimizing or adding shared-memory coupling. Linux software
  rendering does not establish the reference Apple M1 iMac's hardware budget.

## Shared math decision (#115)

Adopt a narrow dependency-free `salimon-math` leaf crate, with only `add`, `sub`,
`scale`, `dot`, `cross` and sum-of-squares `length` on `[f64; 3]`. Actual repeated
code supports sharing these primitives; a generic vector/quaternion framework
or shared frame/normalization policy would obscure existing domain contracts.

| Inventory at this decision | Compatibility and action |
| --- | --- |
| character vector consumers; ship `src/orientation.rs`, `controller/{flight,assist}.rs` | Identical add/sub/scale/dot/right-handed cross; import shared primitives. Ship's length has the same X/Y/Z sum-of-squares order and is imported under its existing name. |
| physics `src/lib.rs` (extracted from runtime `fragment_physics.rs`); runtime `carrying.rs` | Fragment component arithmetic/dot/cross/length has the same order and semantics; import it. Carrying imports the identical cross product. |
| runtime `session/interaction.rs` | Subtract/dot/length also compatible, but leave these composition helpers outside the limited first migration. |
| world `src/coordinates.rs`, `geometry.rs`, `mining.rs`, `resource_fragments.rs` | Length/dot/cross are compatible candidates, left local for now to avoid extending the first migration into world ownership. `WorldPosition`, subtraction/rebasing and resource pose validation remain world-owned. |
| world `resource_generation.rs` | Chained `hypot` length deliberately differs from naive sum of squares; retain overflow-safe deterministic generation math. |
| character/ship/fragment normalization | Current magnitude cutoff is `> 1e-12`, but fallbacks belong to the caller (character/fragment +Y; ship caller-selected). Keep local functions and their exact comparison/arithmetic, including existing NaN/overflow behavior. Character rejection/interpolation/tangent selection stay local. |
| ship quaternion helpers and `ShipPose::axes` | `[x,y,z,w]` local-to-world unit orientation, Hamilton composition, shortest-arc slerp and identity fallback. No identical second simulation implementation warrants sharing them. Keep in ship and test composition/handedness there. |
| world `ResourceTransform::new` | Reject invalid/non-unit orientations, then canonicalize accepted rounding error (`1e-9` squared-norm tolerance); incompatible with ship's identity fallback. Keep validation in world. |
| renderer `lib.rs`, `ship_mesh.rs`, `held_item.rs`, `resource_mesh.rs` | `f32` arithmetic, finite-input/scaled normalization, GPU matrix/quaternion axes and camera basis preparation have separate precision/layout contracts. Retain locally, including all subtract-before-cast paths. |
| character/runtime ship frame conversion | Point translation and axis projection depend on environmental frame contracts. Keep with frame owners rather than adding generic frame types. |

The first migration changes no public gameplay DTOs, normalization rules,
arithmetic ordering, axes or coordinate conversion. Math adds no dependencies;
character, ship and runtime depend on it, while world and renderer remain
unchanged. Shared tests cover right-handed cross products, distant `f64`
subtraction and IEEE edge behavior. Owning-crate tests cover normalization
boundaries and quaternion composition alongside existing movement, landing,
fragment/contact and renderer precision regressions. See
[math maintenance](../client/math/README.ai.md).

## Authored asset boundary

The scout migration (#79–#83) is complete. Blender
[`source.blend`](../models/assets/ships/salimon-scout/README.md) owns editable
geometry/materials; the ship category adapter uses shared export/validation.
The GLB is the runtime artifact, with matching glTF/bin interchange, generated
spatial sidecar and character layouts. Required proxy/marker nodes are nonvisual
metadata, while gameplay policy remains in Rust. Preserve named groups, axes,
monitor UVs, door metadata, budgets and licenses. Read the
[scout regeneration guide](../client/assets/ship/README.md#source-and-regeneration)
for commands and generated files; normal builds never require Blender.

The current scout removes the dedicated cargo module, shortens the nose/glazing
and uses one wide multifunction monitor on the center console with walking routes on both sides.
Loose fragments use the cabin deck. Historical cargo and migration evidence in
[reports/](../reports/README.md) describes earlier revisions. The mining tool now renders a Blender-authored GLB with a camera-local grip
and status material. Iron, silicate and water-ice fragments each use two Blender-authored variants selected by stable identity. All three deposit materials use four authored variants; see [renderer architecture](../client/renderer/architecture.md) for selection and bounds contracts.

## Physical-object simulation decision (#114)

Extract now into `salimon-physics`, a narrow portable crate depending only on
`salimon-math` and the standard library. The existing gravity, ejection/release,
contact, restitution, friction and substep rules already form a reusable unit;
keeping them in native composition would make the next loose-object feature
extend runtime with unrelated rules. A broad `salimon-simulation` crate or full
physics engine is not warranted by this extraction.

Runtime's `fragment_physics.rs` selects loose nearby fragments in session order,
excludes the carried piece, supplies mass-derived radius and selected spherical
body or ship-local floor geometry, projects release directions and maps results
back through the ship frame into validated world poses. World still owns IDs,
material, mass, orientation and session lifetime. Character's authored collision
layout still supplies the pure floor-containment query. Physics owns the response
to that geometry; it never reads the catalog, resource sessions or ship state.

Future cargo, loose equipment, debris and other independently moving physical
objects should use this boundary for compatible motion/contact. Their feature
owners adapt identity/state and environment here, extending physics with focused
contracts/tests when new rules are actually required. Do not add unrelated
physical rules to runtime. Character locomotion and ship flight retain their
existing domain controllers. The #148 solver uses authored convex envelopes, mass-weighted contacts and angular response with approximate isotropic inertia. The stable-ID activation cache sleeps supported quiet contact islands and wakes
connected supports; see the [sleep contract](../client/physics/architecture.md#persistent-contact-island-sleep-156).
Active contacts use deterministic conservative spatial candidates and sparse SAT
geometry; see [the broad-phase contract](../client/physics/architecture.md#deterministic-spatial-candidates-157).
Dense piles retain quadratic worst-case work; this remains a small-object custom solver.
Multiple ship frames must be stepped separately; the current single-ship adapter
preserves frame grouping and deterministic order; convex substeps adapt to scale and speed.
See [physics architecture](../client/physics/architecture.md) and
[invariants](../client/physics/invariants.md) before extending it.

## Future candidates and deferred capabilities

These directions require their own issue, explicit contracts and validation;
they are not dependencies or scaffolding requirements for current work:

| Direction | Status and decision boundary |
| --- | --- |
| Web/WASM and WIT Component Model | Portable Rust reuse and versioned capability interfaces where useful; no component host or WIT boundary currently shipped |
| ECS | Standalone `bevy_ecs` may be evaluated; no canonical ECS or Bevy engine adoption |
| Physics | [#161 decision](../reports/issue-161/README.md): keep the custom solver for accepted settled 500 cargo; defer Rapier comparison/adoption. Active budgets and full stress acceptance remain unresolved; [#165 disposition](../reports/issue-165/README.md) records the user-waived remaining M1 validation, not a full-matrix pass. Rapier is the first alternative if replacement is justified |
| Workers/scheduler/event bus | Evaluate for measured simulation needs; no worker infrastructure required by current native composition |
| Streaming/storage | Larger spatial hierarchies and storage interfaces remain future work; IndexedDB is the browser cache direction, native storage needs a platform implementation |
| Networking/server | Authoritative multiplayer server and shared deterministic domain reuse are future architecture |
| Survival/economy/NPCs | Product directions, not implemented crates or interfaces |

Future public component boundaries should declare capabilities, errors,
compatibility and versioning. Shared buffers need profiling and documented
ownership; unrestricted cross-module shared memory is not the default. Do not
add a framework or runtime service solely because it appears in this table.

## Maintenance and history

Use [AGENTS.md](../AGENTS.md) for GitHub issue/blocker workflow and precedence,
[coding conventions](coding-conventions.md) for implementation/report rules, and
[validation](validation.md) for current checks. Each major implemented crate
maintains its public Rust contract, `README.ai.md`, `architecture.md`, invariants
and focused tests. Update affected contracts, consumers and guides together.

Project specifications and [Project Q&A](project-qa.md) capture product decisions.
[Phase 0 evaluation](phase-0-evaluation.md) retains the dated go-with-revisions
assessment and machine-specific benchmark limits. Older task reports and
[legacy hub notes](legacy-salimon-hub.md) are historical context; GitHub issues
replace the former Notion task database and worker-selection instructions.
The maintenance goal is a focused change whose owners, contracts and useful
checks can be identified without reading unrelated domains.

## Authored water-ice deposits (#88)

Four opaque Blender-authored spire/crown/ridge/shelf exports use the shared
resource mesh batch. Runtime selects `DepositId.local % 4` in that order and
emits no water-ice cuboid. Depleted deposits emit no authored visual. Baked
coordinates fit ±0.48 m; uniform scaling by `bounds_radius_meters /
(0.48 * sqrt(3))` inscribes the visual cube in the authoritative spherical bound
at its absolute deposit center, at every body/latitude. Variant choice ignores
query order, camera, streaming and remaining mass. World generation, mining,
mass, session persistence and streaming rules are unchanged. See the
[asset guide](../models/assets/resources/water-ice-deposit-spire/README.md).

## Scout spatial geometry (#111)

The linked Blender components own 20 scout boxes and four spatial markers.
The offline adapter generates one character `spatial_contracts.rs` alongside
the sidecar/embedded contract. Dedicated cabin envelopes exclude appendages
and account for sills/fixtures; chair and cockpit-side proxies preserve the
previous conservative walking footprints. Character layout composes clearance,
gate/shoulder proxies and query policy from generated raw bounds. Player size,
movement, permissions, ranges, gravity/timing and camera offsets stay in Rust.
See the [authored geometry/policy map](../models/assets/ships/salimon-scout/README.md#geometry-and-gameplay-policy).
Runtime builds require neither Blender nor asset generation.

## Object-relative prompt presentation (#127)

Runtime selects applicable cockpit/door/resource/carrying prompts and supplies
borrowed text pixels plus an absolute `f64` object anchor. Ship markers and
selected physical poses remain the canonical position sources. Renderer owns
camera-relative projection, viewport fitting and reverse-Z anchor visibility
through a separate overlay pass. Global guidance and transient feedback retain
screen placement. See [runtime flow](../client/runtime/architecture.md#contextual-action-bar-flow)
and [renderer contract](../client/renderer/architecture.md#world-anchored-prompts-127).

## Equipment toolbar HUD (#129)

Runtime field-maps five-slot equipment state to renderer-neutral toolbar DTOs
for gameplay; precision tour omits it. Renderer rasterizes/caches numbered slots,
mining-tool silhouette and optional highlight, and stacks global/transient
messages above the fitted bottom-center toolbar. DPI follows bounded native
scale-factor handling. See the [renderer contract](../client/renderer/architecture.md#equipment-toolbar-hud-129).

## Integrated terrain (#170, #179–#181)

World owns the immutable versioned [terrain definition/query/codec](../client/world/terrain-contract.md).
Runtime composes shared ground queries, authored landing footprints and resource
anchoring with bounded cube-sphere patch generation. Renderer owns immutable
patch buffers, a uniform arena and near/far presentation through renderer-neutral
DTOs, with no world dependency. Normal native gameplay enables Earth/Moon/Mars;
legacy flat scenarios and F2 precision markers retain their fixtures. Read
[the integration guide](terrain-validation.md) for budgets, approximation limits,
physical policy and native/reference validation. Networking, persistence and
volumetric terrain remain deferred.

## Flight update and validation decisions (#197)

Accepted review decisions **42 and 78** require deterministic bounded fixed ticks
and explicit accumulated-time handling. Runtime admits elapsed time and drains
16 ms ticks with a wall-clock catch-up budget; remaining backlog is retained.
E2E explicit stepping drains admitted ticks without that wall-clock budget.
Ship direct API updates split at energy/conversion/assist boundaries so force
never extends beyond available power and takeoff consumes leftover elapsed time.
No per-frame cargo scan determines propulsion mass: physical mutation/boundary
events feed the cached ledger; explicit reconciliation is diagnostic only (77).

Ship owns mass/force, fractional joules, single-fragment conversion, isolated
assistance budgets and full-vector observational telemetry. Runtime supplies
physical intake, containment and power snapshots to character/fragment adapters.
Renderer receives observation DTOs only; it neither estimates stop feasibility
nor commands thrust. See [ship contracts](../client/ship/architecture.md),
[accepted gameplay decisions](spaceship-building-and-survival.md#accepted-propulsion-and-core-review-2026-10-14),
[portable E2E](../scripts/README.md#realistic-flight-regression-195) and
[500-cargo CPU profiling](../scripts/FRAGMENT_BENCHMARKS.md#flight-mass-and-cabin-power-cpu-profiles-196).

Issue owners: #201 inertial force, #186 throttle, #202 swept crash, #187 cached
mass, #188 Core, #189 intake, #203 reservations, #190 safe assist/recovery,
#191 interior gravity, #192 blackout controls, #193 navigation, #194 Core display,
#195 regressions, #196 profiling, #197 documentation. Hardware-only #198–#200
remain native tasks; passing cloud checks does not imply M1/60 FPS validation.
Future thruster/landing upgrades, extra module loads, full cabin inertial physics,
ship gravity/orbits and manual power management are explicitly OUT OF SCOPE.

## Coherent render-only gameplay interpolation (#310)

The runtime session retains previous/completed ship and character observations
beside loose-fragment history. Its [presentation contract](../client/runtime/architecture.md#bounded-gameplay-ticks-and-fragment-interpolation-159)
uses one clamped backlog factor and consistent world/ship-local frames. Live look
and all gameplay authority remain current; automation renders exact snapshots.
Lifecycle and frame transitions invalidate history. Native smoothness/latency
verification remains separate in #314.

## Position boundaries and resource dimensions (#311)

The high-risk frame inventory is deliberately limited to concrete boundaries;
this does not introduce a generic vector library or move domain policies to math.

| Boundary / owner | Contract and treatment |
| --- | --- |
| Persistent loose ship cargo → world poses (`runtime/carrying.rs`, `fragment_physics.rs`, `session.rs`) | Anchor map values are `salimon_character::ShipLocalPoint`, not interchangeable with raw or absolute positions. Insertions explicitly assign the solver/fixture frame; pose-derived insertions call `ShipFrame::project_world`. Synchronization calls `unproject_local`. |
| Historical/live character eyes → presented ship frame (`runtime/session/presentation.rs`) | `WorldPoint` → `ShipLocalPoint`; local interpolation accepts/returns only ship-local points before typed unprojection. Attached fragment remapping uses the same typed conversion pair. |
| Runtime → portable physics (`fragment_physics.rs`) | Unwrap local points only when building `ObjectState`; floor/unpowered-floor snapshots use ship coordinates, other surfaces use absolute coordinates. Floor output explicitly constructs a local point before storage. This existing surface-tagged solver DTO is retained; separate ship frames still require separate solver calls. |
| Body-relative terrain → absolute ground / renderer patches (`world/terrain`, `runtime/terrain.rs`) | Existing `TerrainSample.body_local_position_meters`, body-center `WorldPosition` and patch-specific DTOs name the frame explicitly. Retained as frame-specific DTOs, without changing the terrain codec or normal/query policy. |
| Absolute CPU → camera-relative GPU (`world/coordinates.rs`, renderer preparation) | Existing `WorldPosition` and private prepared renderer records retain subtract-in-f64-before-f32 conversion. Renderer remains domain-independent; no world/character dependency is added. |

`WorldPoint` is a narrow character-boundary DTO, not a replacement for the
world domain's canonical `WorldPosition`. The DTOs provide explicit `new` and
`meters` methods, with no implicit conversions, dereference, arithmetic operators
or serialization. Raw array `ShipFrame` adapters remain available for existing
character-local geometry. Both APIs share the exact previous f64 operations and
axis order. Constructors assign a frame, not finite-input validation; quaternion,
handedness (+X forward, +Y up, +Z port), normalization and domain fallback policies
are unchanged. Compile-fail examples reject reversed typed conversion arguments.

Analytic fragment size remains owned by `resource_size`: contact radius in meters
and `resource_fragments::diameter_meters` / `fragment_diameter_meters` for diameter.
Renderer `ResourceMeshInstance.authored_mesh_scale` and physics
`ObjectState.authored_mesh_scale` are dimensionless multipliers of authored mesh /
hull coordinates, never an analytic diameter. For the shared fragment GLB, visual
scale is radius divided by `SPHERE_AUTHORED_RADIUS_METERS` (0.48 m). Analytic contacts ignore
that scale when `hull` is absent; sphere support/contact uses `radius`. Runtime
history invalidation compares analytic radii, not a legacy size alias.

External automation/benchmark JSON stays compatible: fragment `side_meters`
continues to serialize diameter; `authored_visual.side_meters` and legacy solver
snapshot `side_meters` keep their previous numeric meanings. The solver snapshot's
unused mesh-scale slot retains its historical diameter value for analytic spheres;
it is not consumed as analytic contact size. No asset, terrain codec, scenario,
material-volume, mass or session persistence contract changes.


Fabricator crafting is newly authorized by the [accepted five-round contracts](spaceship-building-and-survival.md#accepted-fabricator-contracts-2026-10-19-336). Implementation is staged: #336 contracts, #337 raw materials, #338 portable domain, then separate runtime/UI integration. Survival effects, disk persistence and playable progression remain deferred.
