# Technical architecture and AI maintenance

[Architecture baseline and ownership audit](architecture-baseline.md) records measurable refactoring priorities (#638).

## Implemented architecture

Salimon uses a custom Rust runtime with low-level `winit` and `wgpu`, without a
full game engine. The current prototype extends the Phase 0 baseline and remains native and
client-only. Local build scripts stage macOS, Windows and Debian-based Linux
executables. PR CI compiles and runs portable workspace tests on Ubuntu. Separate nightly
verification and weekly benchmarks/platform builds publish reports; all local
commands remain available. Native macOS is the reference playable/performance target. Web/server reuse remains a future capability, not a shipped target.

The [feature maintenance map](maintenance-map.md) routes work to source, tests,
invariants and scenarios. The [client guide](../client/README.md) defines package
dependencies; each crate's architecture/invariants defines its local contracts.

| Boundary | Implemented ownership |
| --- | --- |
| `client/runtime/` | Native shell/lifecycle/evidence IO plus portable GameplaySession commands, snapshots, stepping, resource/cargo adapters and presentation mapping |
| `client/fabrication/` | Portable validated recipes, integer raw storage/reservation, FIFO jobs, single tray and typed consumable slots; runtime adapters remain separate |
| `client/world/` | Static six-body compressed catalog, `f64` coordinates/camera tour, resource contracts, generation, extraction/session deltas, fragments and one-object carrying |
| `client/ship/` | Pose, cockpit authority, inertial force-based flight, uncancellable landing/takeoff, airlock interlocks, live telemetry and independent portable structural topology |
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
streaming and normal-play process restarts through versioned whole-session
checkpoints (#363–#366). The canonical [reload verification scope](../client/world/resource-contracts.md#persistence-and-reload-verification-scope-268)
defines in-session restoration and the proposed #236 scope clarification awaiting
owner approval; normal play restores its active run, while E2E/benchmarks start isolated sessions. No orbital simulation,
multiplayer is implemented. Survival meters/status/recovery, active clocks,
whole-game persistence and terminal/restart lifecycle are implemented through
#371; HUD, Journey Ended and starter supplies are implemented by #373/#374/#379; native Q consumption is integrated by #377.
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

The current Scout retains its compact outer hull and precise ramp/cockpit/station
anchors. The #538 cabin envelope adds a separate aft cargo compartment, a
full-height bulkhead with a 3 m internal opening, open storage shells and
wall-integrated stowed bunk, galley and hygiene fixtures. Character movement,
sight and floor placement consume the same generated bounds. Operable internal
door/bin mechanics remain #539; habitation interactions remain #540. Historical
cargo-removal reports describe earlier revisions. The mining tool now renders a Blender-authored GLB with a camera-local grip
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

**Future canonical planet source contract:** [multi-seed planet identity](canonical-planet-generation.md) separates the versioned immutable planet definition (offline package now, matching backend catalog later), independent but **explicitly versioned interdependent** procedural domains (geology input → one authoritative terrain query; geology/biome inputs → optional resource distribution; visuals remain physically inert), mutable shared-world deltas and local renderer caches. Published fictional planets may change only via controlled migration; all real observed solid bodies freeze their physical surfaces from first authoritative real-scale publication, not from prototype fixtures. The present six-body terrain codecs and existing save/runtime seeds are not that future catalog; keep legacy invariants until the dedicated universe issues implement their replacement.

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

The #338 portable Fabricator domain is implemented in [salimon-fabrication](../client/fabrication/architecture.md). It depends only on world resource contracts; native/playable integration remains #339–#347.

## Whole-game persistence (#363)

The [canonical save contract](save-contract.md) inventories authoritative state,
restore order, clocks, validation and schema evolution. `salimon-persistence` owns
the platform-independent versioned envelope and deterministic stable-ID map codec.
Domain adapters follow in #364 and session/Fabricator composition in #365; disk
checkpoints remain #366. Renderer caches are never save authority.

The #364/#365 adapters now implement the typed whole-session in-memory save
boundary: runtime `session/persistence.rs`, domain character/ship persistence
modules, world `mining_save.rs`, physics `SleepSaveState` and fabrication
`save.rs`. Machine/inventory/physical identities validate before atomic session
publication; derived mass, power request and caches regenerate. Persistence
regressions live in runtime session/persistence and fabrication_scenarios.
Filesystem durability and native lifecycle checkpoints remain #366.

## Durable active run (#366)

Persistence `storage.rs` owns the two-generation verified storage protocol and
native filesystem lease/adapter. Runtime `checkpoints.rs` composes whole-session
restore, 60-second/event capture, bounded background IO, visible recovery/failure
warnings and verified clean exit. See [canonical contract](save-contract.md#durable-active-run-366).
Survival/death/restart/archive integration is implemented by #371/#372; #374 owns
Journey Ended presentation. No offline
world simulation or multi-save UI is implied.

## Survival meters (#367)

Portable meter/active-clock/offline-fatigue authority lives in
[`salimon-survival`](../client/survival/architecture.md). Native/session persistence
integration is implemented by #371; the playable session advances and saves survival.

Damage, regeneration, treatment-only wound debt and data-driven timed/persistent/
exposure status effects (#368) use the survival crate's [single pipeline](../client/survival/architecture.md#damage-and-status-pipeline-368).


## Journey statistics (#372)

Runtime `session/journey.rs` owns portable travel samples/counter and final named
location; survival owns active duration and typed death cause. Schema-3 saves and
version-2 death reports compose these with physical authority, without reading
render-relative coordinates. Background occlusion/zero drawable keeps bounded
simulation active. See the [canonical metric/save contract](save-contract.md#journey-statistics-372)
and maintenance map for sources, frame policy and regression ownership.


## Survival HUD (#373)

Runtime `survival_hud.rs` owns typed observational mapping and presentation
feedback/cache, with bounded raster/layout in `survival_hud/raster.rs`; `app.rs`
owns H/[ / ] focus-safe dispatch and overlay priority. Survival's immutable
`effect_snapshots` supplies captured rules/stacks/timers. Renderer exposes its
existing fitted toolbar message band; existing inventory/equipment art is unchanged.
See the [canonical HUD contract](../client/runtime/architecture.md#survival-hud-and-status-details-373).
Tests cover actual consumables, exact penalties, visual timing/cache, all supported
viewport/DPI layouts, full effect pagination and native input policy. #376 retains
native visual signoff; #381/#377 integrate menu/wheel admission.


## Journey Ended (#374)

Runtime `journey_ended.rs` + `journey_ended/raster.rs` own the report screen and
physical pointer layout. `session/lifecycle.rs::ended_run` is shared with durable
archives; `app.rs::restart_terminal` composes the existing verified restart for
keyboard/mouse/native automation. Renderer only exposes its generic screen margin.
See the [terminal contract](save-contract.md#journey-ended-presentation-374).
Tests: `journey_ended/tests.rs`, native-host input/restart regressions and existing
checkpoint/restart fault injection. Reference native visual/input acceptance is #376.

Four-food definitions and continuous food modifiers are implemented by #378 in
survival, with locked advanced recipes in fabrication. Native wheel input is integrated by
#377; no progression acquisition is implemented. See the
[canonical food contract](../client/survival/architecture.md#four-food-catalog-378)
for captured settings, active/offline clocks and exact drain arithmetic.

Scout light zoning and reproducible native/offline cameras (#463) are defined in
[Scout presentation](scout-presentation.md); CPU/Blender results do not grant native visual signoff.

Scout audio (#465) is a runtime-only presentation adapter with bounded stereo
PCM and shared optional OS playback, independent of run saves and all domain
authority. [Soundscape contract](scout-presentation.md#soundscape-boundary-465)
records spatial/mute/quiet-zone rules and the unverified native playback limits.

Scout machinery (#464) uses a stateless runtime snapshot mapper and renderer-only
normalized feedback, sharing authoritative power/progress with existing monitors
and audio. It adds no gameplay controller or saved state. See the
[feedback contract](scout-presentation.md#machinery-feedback-boundary-464).

Pause saves (#382) use the existing worker/store and verified receipts; see
[manual checkpoint contract](save-contract.md#pause-checkpoint-controls-382).

## Future sub-light voyage, Core impulse and Spatial Jump Drive boundaries (2026-10-21)

**Implemented engine / future Jump Drive:** #493 implements fictional 1 N·s/J from funded Core propulsion, 100/50 MW default draws, explicit engine on/off and schema 5. The Core remains 60 GJ. The [spatial Jump Drive and shared clock](jump-drive-and-interstellar-travel.md) remain future; native engine handling review is #498.

- **Ship/Core portable domain:** switch the engine to **actually funded energy × 1 N·s/J = vector impulse**, applied through current total mass, orientation and exact substep power boundaries. Document and test the significant acceleration change; don't bill conventional kinetic-energy differences. **No change to deployed flight** is implied by this document.
- **Future Jump Drive portable domain:** validate mass, finite direction and distance, N=20% initially, speed 0<=v<c, one-frame epoch and destination safety; energy = light_seconds * mass_kg * (N/100) * (c-v_mps) / kappa. Preserve real velocity and orientation. Refuse engine-on, including idle zero throttle, through a **new explicit propulsion switched-off authority bit**; reject incompatible assists and insufficient unreserved Core. Other speed regimes remain undecided and must not create negative/free jumps.
- **Fabricator/world:** version physical component and upgrade recipes/IDs using existing queue/raw-bin/tray/material ownership. The first module is permanent, not a disposable item or food/medical slot; installed mass contributes to the ship ledger. The old 20 kg raw bin and one-object carrying constraints still apply.
- **Runtime/render:** owner-authorized **six-field** light-distance panel quotes energy and errors without mutating simulation; verified single-debit quantum-transition transaction, atomic onboard contents, staging/loading scene and full position/frame change only after the relevant world region loads. Renderer uses immutable observation DTOs; no GPU-owned movement, power or chronology.
- **Persistence/future backend:** shared 1:1 real elapsed universe time for offline/online users (no relativistic per-player clocks), single authoritative jump position/energy event, idempotent restart/reconnect and passengers, physics catch-up. Offline engines off; offline coasting continues and jumps are not automatic. The precise multiplayer backend does not currently exist.
- **Gameplay/economy:** one-to-twelve-month real-time long voyages and approximately 6–12 months of dedicated galaxy exploration are **aspirational balance**, not evidence that the current 60 GJ Core funds even a 1-light-second jump at N=20%. Explicit balance review required.

Survival onboarding (#380) is implemented as portable per-run objective/skip
state in runtime, persisted by schema 4. Events come from existing successful
Q/Fabricator/status actions; prompts use the existing action band and do not
alter input or simulation authority. See the [owning contract](../client/runtime/architecture.md#persistent-survival-onboarding-380).

## Portable persistence/survival scenario boundary (#375)

Test-only runtime modules compose production full-session saves, the two-slot
filesystem manager and normal `Checkpoints::open_at` launch transaction, plus
worker/restart transactions over fault-injected `CheckpointStore`. The
[versioned portable DSL](../scenarios/portable/README.md) admits checkpoint/reload
and full-source comparison without adding filesystem writes to the live E2E
protocol. Fixture placement and supplied UTC are explicit; clocks and domain
rules remain production-owned. Long meter partition tests run directly against
survival, while separate fixed-tick session tests prove clock/death routing.
Bounded coverage runs in PR workspace tests; extended coverage runs nightly.

## Unified Scout UI presentation (#466)

The [Scout UI contract and surface matrix](scout-presentation.md#scout-ui-readability-contract-466)
connects existing physical monitors, contextual prompts, crafting, toolbar, Q/H
and reticle owners. Renderer `ui_style` provides shared bitmap glyphs/functional
colors, instruments own bounded formatting/caching and screen UV interpretation;
runtime owns text selection, eligibility, focus and typed commands. Existing
domain power/simulation/mass/persistence ownership is unchanged. Delivery evidence
is [reports/issue-466](../reports/issue-466/README.md); M1 signoff is #502, and
scanner integration #503 now formats authoritative #430 observations through
renderer `ScannerReadout` and the existing shared-face runtime action bar.
Readout rows and 3D functional inks follow [Scout presentation](scout-presentation.md#scanner-readability-503).


## Scout Archive (#467)

Runtime owns local Archive records and modal input; the existing save manager
persists validated identities/read progress in schema 6. Renderer adds a flush
panel to its existing ship draw and one static instrument atlas region. No backend,
chart or recovery authority is added. See [ownership](../client/runtime/architecture.md#scout-archive-467).

## Canonical universe schema foundation (#402)

`client/universe/` (`salimon-universe`) now owns immutable validated catalog
schema, exact source IDs, systems/stars/individual bodies/statistical populations,
per-field provenance/uncertainty/validity and versioned canonical definition/seed
references. It is a portable leaf, not a runtime dependency. The six-body playable
`salimon-world`, terrain/resource-v1 and current save interpretation are unchanged.
See [schema ownership and bounds](../client/universe/architecture.md). Catalog
truth grants no player chart knowledge; frames, storage/import and native world
migration remain U02 onward.

## Universe documentation maintenance (#440)

Every U01–U34 PR applies the [same-PR documentation gate](documentation-maintenance.md)
with owning architecture/invariants links and checked implementation status.
Root/source maps and domain contracts change alongside behavior, or a precise
no-impact exception is reviewed. This is contributor/reviewer policy; automated
CI lint #442 and final semantic audit #441 remain separate scoped work.

## Universe coordinates and epoch (#403)

The portable universe leaf also owns split galactic cells, typed system/body/local
frame trees, rotating-frame velocities and validated calendar/scale arithmetic.
See [U02 contract](../client/universe/architecture.md#hierarchical-frames-and-epoch--u02-403).
Runtime and compressed playable-world coordinates remain unchanged; ephemerides,
UTC leap-table conversion and production astrometry import are downstream work.

## Universe new-run saves (#406)

Owner: `client/persistence/src/universe.rs`; tests: `src/universe/tests.rs`.
Persistence now depends on the portable universe leaf for validated IDs, split
positions and epochs. Existing `SaveManager` protects catalog incompatibility as
a fatal recovery error. Runtime retains its separate runtime envelope (now schema 26). See
[save contract](save-contract.md#future-universe-new-run-contract--u05-406),
[persistence architecture](../client/persistence/architecture.md) and
[invariants](../client/persistence/invariants.md).

## Universe source inputs (#407)

`client/universe/src/sources.rs` owns typed source metadata;
`scripts/import_universe_sources.py` verifies frozen local raw bytes and stages
a deterministic bundle. `catalogs/sources/manifest-v1.json` separates synthetic
Gaia fixtures from actual JPL/exoplanet snapshots. [Source contract](universe-sources.md),
[architecture](../client/universe/architecture.md#source-manifests--u06-407) and
[invariants](../client/universe/invariants.md) govern reuse/precision boundaries.
Production reuse/full snapshots wait for #513; science ingestion remains U07/U08.

## Data Recovery Module (#468)

Runtime `session/recovery.rs` owns Archive work; Fabricator data provides the
controller recipe and typed tray output; ship funds low-priority surplus joules
and accounts installed mass; renderer only styles the existing package. Runtime
schema 7 persists source and rejects duplicate ownership. See [canonical contract](data-recovery-module.md),
[runtime architecture](../client/runtime/architecture.md#data-recovery-module-468)
and [invariants](../client/runtime/invariants.md). Tests: `session/recovery/tests.rs`
and ship auxiliary energy tests; native acceptance #515, future leads #470.

## Universe binary partitions (#405)

Owner: `client/universe/src/partitions.rs`; tests: `partitions/tests.rs`, independent
wire fixture `fixtures/partition-v1.bin`; workload: `examples/partition_bench.rs`
(all relative to `client/universe/`). Read the [storage contract](universe-storage.md),
[owning architecture](../client/universe/architecture.md#binary-partitions--u04-405)
and [invariants](../client/universe/invariants.md). Bounded prototype only: no GB
builder/cross-cell references/runtime cache or production reuse approval. U12/U13
retain those boundaries. Run universe/workspace gates and the release workload.

## Universe scientific reference fixtures (#404)

Owner: `client/universe/src/reference.rs`, tests: `reference/tests.rs`, goldens:
`fixtures/references-v1.json`, examples: `reference_compare.rs`, `reference_bench.rs`
with `support/reference_inputs.rs` (paths within `client/universe/`/its examples).
See [reference wire/budget/workload guide](universe-reference-fixtures.md),
[architecture](../client/universe/architecture.md#scientific-reference-harness--u03-404)
and [invariants](../client/universe/invariants.md). Strict SI/epoch/frame comparison
and CPU diagnostics only; no solver/world migration or scientific model approval.
Cloud reference measurements remain #521; full-game cloud gates remain #435–#439.
Human presentation/platform/budget criteria are #561/#557/#563 under the
[verification workflow](verification-workflow.md); no M1 result is inferred.

## Universe Gaia ingestion (#408)

Owner: `client/universe/src/ingestion/gaia.rs`, shared `ingestion/csv.rs`; tests:
`ingestion/gaia/tests.rs`; workload: `client/universe/examples/gaia_bench.rs`.
See the [Gaia selection/units/cross-match contract](universe-gaia-ingestion.md),
[architecture](../client/universe/architecture.md#gaia-ingestion--u07-408) and
[invariants](../client/universe/invariants.md). Synthetic local ingestion and
bounded canonical batches are implemented; production approval/full snapshots
remain #513, inference U09, astrometry pack integration U12. No playable migration.

## Universe observed ingestion (#409)

Owners: `client/universe/src/ingestion/horizons.rs`, `exoplanets.rs`; tests in their
sibling module directories; fixture example `client/universe/examples/observed_import.rs`.
See [units/identity/validity contract](universe-observed-ingestion.md),
[architecture](../client/universe/architecture.md#observed-solar-system-and-exoplanets--u08-409)
and [invariants](../client/universe/invariants.md). CI compares two offline builds.
Actual frozen Earth/Proxima and synthetic matching work; full reviewed release
coverage remains #526/#513, packaging U12, arbitrary-date models U14. No runtime
world, scanner knowledge, save or rendering migration is implied.

## Universe missing-data inference (#410)

The portable universe leaf now owns offline, model-classified single-star
completion of missing physical quantities, age and partial orbits, with versioned
proposal/confidence evidence and whole-system physical screens. U08 source limits
and conflicts reject before inference. See [contract](universe-inference.md) and
[owning architecture](../client/universe/architecture.md#missing-data-inference--u09-410).
This does not replace runtime/world state, fit empirical population frequencies,
prove long-term stability or package the sidecars for production (U12).

## Canonical fictional systems (#411)

The portable universe leaf now owns a bounded offline fictional-system builder
using explicit residual density, age and metallicity inputs, reserved stable
IDs, SHA-256 lanes, individually materialized bodies and aggregate minor
populations. Observed source records remain intact. Complete generation recipes
and partial orbital/stellar sidecars are hashed with the catalog. U11 below owns
the selected world's full definition/surface query; other worlds' definitions
remain downstream and U12 owns production batching/packaging. See the
[builder contract](universe-fiction-builder.md) and [owning architecture](../client/universe/architecture.md#canonical-fiction-synthesis--u10-411).
Synthetic distribution/storage evidence does not certify a real Milky Way pack
or playable/native integration.

## Permanent awakening publication (#412)

The portable universe leaf now freezes Nacre Reach / Vesper from a pinned U10
cohort. `starting_system.rs` owns stable IDs/names, >=30 ly distance proof,
physical screening, full selected-world parameters, baseline outcrops/arrival
routes, independent domain digests and a single surface evaluator. See
[contract](universe-starting-system.md) and
[owning architecture](../client/universe/architecture.md#permanent-starting-world--u11-412).
U01 recipe references remain unchanged; U12 packages the expanded U11 sidecar.
The runtime still starts in the compressed prototype. U14 states, U23 LOD and
U24 arrival/mining integration retain their scopes, with separate native gates.

## Portable modular structure (#541)

Ship's independent `structure` module owns the 3 m Core-rooted sparse graph,
structural/room/equipment IDs and envelopes, safe removal, damage component
handoffs, explicit baseline/additional mass and domain schema 1. See the
[owned boundary](../client/ship/architecture.md#portable-structural-foundation-541).
Runtime composition, current authored hull/colliders and whole-game schema are
unchanged. This foundation does not claim player construction or the redesigned
Scout is playable; #538–#550 retain their integration scopes.

## Physical mounts (#576)

Physical mounts compose ship-owned attachments, the world-owned one-object slot and runtime physical/presentation adapters. Schema 11 preserves exact source and safe legacy layouts. See [the canonical contract](physical-mounts.md).


## Expanded cabin mechanisms (#577)

The optional schema-12 cabin source, manual reversible fixtures, supported postures, visible restraints and native controls are implemented in [the maintained cabin contract](cabin-mechanisms.md). Historical schema-10 habitation remains the migration baseline; lighting preferences (#580), sound (#583), human comfort (#561) and hardware budgets (#563) retain their separate owners.


## Persistent cosmetic groups (#579)

Five room groups and exact furniture overrides now use the [bounded finish contract](room-finishes.md). Schema 13 preserves histories independently of divider topology; physical material replacement, future constructed room geometry (#545) and actual damage (#549) retain their owners. Shared instance tint factors are now bounded to [0,4] and cosmetic RGB is clamped before lighting; warning/functional surfaces retain authored values.


## Instant direct-material Core upgrades (#548)

The owner-authorized capacity tiers, exact receipt accounting, separate fuel/material controls and schema-14 save behavior are implemented in [Core upgrades](core-upgrades.md). Ordinary construction/commissioning and blackout throttle rules remain separate. No free charge or upgrade power/timer is introduced.


## Construction material ownership (#543)

See [construction material transactions](construction-materials.md) for exact original receipts, atomic physical refunds, schema 15, mass and the owner-approved construction economy. Paid work and completed geometry follow [construction progress](construction-progress.md) and [modular structure](modular-construction.md).


## Room and fixture lighting (#580)

The [maintained lighting contract](room-lighting.md) describes implemented presets, independent overrides, effective bus/condition gates, shutter/Sun contribution and schema 16. Actual future damage/room creation remain #549/#545; human approval #561 and hardware budgets #563.


## Tactile audio and physical partitions (#583)

The [maintained acoustics contract](scout-acoustics.md) describes actual mechanism/attachment events, finite cargo/privacy separation, differentiated floor contact and bounded continuous CPAL filtering. E2E mapping stays silent; actual output captures and human listening remain separately identified.


## Automatic construction (#544)

See [automatic construction contract](construction-progress.md) for active Core-funded timers, safety/pause admission, simple interior completion, schema 17, exact save/refund behavior, production evidence completed [modular structure](modular-construction.md) and [optional module commissioning](non-core-modules.md).


## Completed modular structure (#545)

See the [canonical completed construction contract](modular-construction.md) for the ship assembly/shell and runtime installed/work owners, character traversal, per-cell/face identity, material mass transfer, schema-18 reconstruction/migration, partition editing and the production room evidence scenario. Gameplay damage/salvage remains #549; offline construction remains #550.


Optional module composition (#547): runtime construction owns tier/maintenance/commissioning source, existing ship Funding/Work retain material/time arithmetic, Recovery requests actual Core-funded throughput and renderer consumes physical case geometry. See [the canonical owner and save/test mapping](non-core-modules.md).


## Construction damage and salvage (#549)

See the [canonical damage and schema-21 contract](construction-damage.md) for runtime `session/construction/damage.rs`, ship lost-cell geometry, world sealed salvage allocation, controls, exact source/mass/identity boundaries and focused tests/evidence routes. Run workspace tests/build/Clippy/fmt plus documentation validation.


## Offline construction (#550)

See [the maintained chronology and schema-22 contract](construction-offline.md): runtime `session/offline.rs`, ship `controller/offline.rs`, launch/checkpoint staging and existing work owners. Run offline/checkpoint/Recovery and ship tests, workspace build/tests/Clippy/fmt, the two documented wgpu routes and docs/base-impact validation. Future #433 supplies verified universe events; no scanner or orbit implementation is duplicated here.

Runtime domain save adapters preserve the explicit transactional layout; see
[ownership and byte compatibility](save-contract.md#runtime-domain-mapping-ownership-641) (#641).


## Session domain composition (#642)

Runtime retains explicit typed domain fields; input owns mutable held state and
host execution policy is a separate nonserialized value. Initialization, command
admission, interactions and ordered tick stages have focused owners. The
[runtime ownership table](../client/runtime/architecture.md#internal-composition-modules)
and [stage contract](../client/runtime/invariants.md#session-composition-owners-642)
identify the update order and source files. Save/restart/offline publication copies
host policy and resets held input; schema 23 remains unchanged. No new dynamic
registry, event bus, dependency cycle or hidden scheduler is introduced.


## Presentation owners (#643)

Renderer resource ownership and public DTOs remain in its facade. Private
initialization, surface and frame modules isolate GPU construction, drawable
resize and ordered submission. Runtime native presentation and contextual
prompts have focused app owners, while Scout visuals map through the scene
adapter. See [renderer ownership](../client/renderer/architecture.md#pipeline-and-frame-owners-643)
and [runtime composition](../client/runtime/architecture.md#native-presentation-composition-643).
No domain dependency, API, shader layout, draw order or simulation authority changes.


## Contact and resource geometry owners (#644)

Physics separates substep/broad-phase scheduling from exact sampled contact and
impulse response in its private `response.rs`. Renderer separates pure resource
GLB validation/index packing from GPU batch state. This is source organization
within the existing domains: numerical behavior, public contracts, serialized
state, assets/shaders and allocation policy are preserved. Convex/SAT and
ship-assist owners remain cohesive; see the [maintenance map](maintenance-map.md#contact-and-resource-geometry-review-644).


## Incremental private offline packs (#413)

See the [implemented contract](universe-pack.md) for ownership, inputs, bounds, validation and remaining production/runtime limitations.


## Bounded read-only pack queries (#414)

See the [implemented contract](universe-reader.md) for ownership, inputs, bounds, validation and remaining production/runtime limitations.


## Time-addressable ephemeris models (#415)

See the [implemented contract](universe-ephemerides.md) for ownership, inputs, bounds, validation and remaining production/runtime limitations.


## Dominant-source celestial gravity (#416)

See the [implemented contract](universe-gravity.md) for ownership, inputs, bounds, validation and remaining production/runtime limitations.


## Gravity-aware ship trajectory (#417)

See the [implemented contract](universe-trajectory.md) for ownership, inputs, bounds, validation and remaining production/runtime limitations.


## Real-scale surface flight (#418)

See the [surface-flight contract](universe-surface-flight.md) for rotating/orbiting support,
Core-funded physically bounded landing/takeoff, production source/tests and
remaining playable-world adoption. Current compressed gameplay is unchanged.


## Real-scale atmospheric drag (#419)

See the [atmosphere contract](universe-atmosphere.md) for planet-specific density,
co-rotating gas/wind, atomic online/offline drag integration and physically paid
assist compensation. Model validity, tests and playable-adoption limits are explicit.


## Real-scale moving contact (#420)

See the [continuous-contact contract](universe-contact.md) for moving solids/terrain,
certified earliest-event queries, atomic gravity/drag interception, body-relative
consequences, funded-work receipts, cloud tests and adoption/precision limits.


## Predictive spatial streaming (#422)

See the [streaming contract](universe-streaming.md) for worker ownership,
nonblocking runtime polling, explicit residency limits, stale-result rejection
and immutable quality-independent content. Startup uses `SALIMON_UNIVERSE_PACK`;
loaded catalog truth does not grant player discovery or replace gameplay yet.


## Canonical terrain streaming (#424)

See the [canonical terrain contract](universe-terrain.md) for the exact U11
evaluator, separate frozen identity/codec, planet-wide quadtree, stitching and
CPU/GPU bounds. Compressed six-body fixtures remain unchanged. Gameplay adoption
and normal alien-world startup belong to #425/#426.


## Canonical surface integration (#425)

The [canonical gameplay contract](universe-gameplay.md) maps current body/frame,
resource/physics/Fabricator integration and schema-24 definition binding to owners
and tests. It preserves the frozen terrain and explicit historical fixtures.

Canonical new journeys (#426) use `session/canonical.rs::new_canonical_run`: frozen Vesper ID, published traveler offset [14,0], conservative authored-hull clearance and a view facing the landed ship. Fresh terminal restarts preserve run serial/seed policy and grant existing supplies once; restore never grants them. Flat fixtures remain explicit; normal launch rejects prototype saves under #427. Oxygen depletion stays deferred. Tests and native evidence: `reports/issue-426/README.md`; setup `canonical-journey`.


## Navigation forecasts (#421)

See the [bounded prediction contract](universe-navigation.md): `ship/src/prediction.rs` (split-coordinate coasting gravity) and `ship/src/preview.rs` (actual playable power/contact model), `runtime/src/session/navigation.rs` disposable cache, `app/scene.rs` / `automation/inspection.rs` observation mapping, and renderer `cockpit_instruments.rs`. Run ship/runtime navigation and renderer tests, all-target Clippy/fmt/docs, runner tests and `scenarios/evidence/universe-navigation.json` in production software-Vulkan. No autopilot, source-energy mutation, chart reveal or certified arrival/error bound is introduced.


## Scanner contacts (#428)

`client/universe/src/scanner.rs` owns bounded player discovery, exact sampled
celestial states, three-hour freshness and durable encounter history. Unknown
catalog entries are never revealed implicitly; batches validate before mutation.
See the [scanner contract](universe-scanner.md) for ownership, clock semantics, tests and downstream integration.


## Scanner delivery (#429)

Universe `scanner/pulses.rs` owns fixed five-second cadence and analytical funded linear-path detection. Runtime `session/scanner.rs` composes ordinary Core funding after construction, samples current body-local state and exposes read-only automation. Unknown orbital phase/velocity is never invented; body-local and galactic observations cannot be mixed.
See the [scanner contract](universe-scanner.md).


## Scanner delivery (#431)

Scanner contact checkpoints validate catalog revision, exact IDs/frames, timestamp ordering and bounds before atomic reconstruction. Runtime schema 25 saves canonical scanner parameters, clock/remainder and encounter history; legacy canonical saves migrate empty, current missing/malformed sources reject. Closed-app launch advances freshness once without inventing offline detections.
See the [scanner contract](universe-scanner.md).


## Scanner delivery (#430)

Renderer `scanner_hologram.rs` owns bounded depth-tested ship-local 3D projection geometry; runtime `session/scanner/presentation.rs` maps only fresh same-frame discoveries to ship pose. The physical console projector shows logarithmic distance, true 3D direction/elevation and selected contacts. M cycles contacts only in an admitted canonical cockpit; power loss suppresses projection, not history.
See the [scanner contract](universe-scanner.md).


## Scanner delivery (#469)

Universe `scanner/chart.rs` owns player-known observations, inferred/transmitted leads with explicit uncertainty, and immutable voluntary route annotations. Runtime `session/chart.rs` owns cockpit P/M/Enter controls and schema-26 pair validation with scanner history. Catalog truth never seeds the chart; observed knowledge outlives contact TTL and source frames are preserved without guessed transforms.
See the [scanner contract](universe-scanner.md).


## Canonical geology (#612)

World `terrain::geology` owns versioned terrain-bound province/history/substrate
queries. The returned physical sample is the exact existing terrain; geology
never displaces ground or reshuffles resource identities. See the
[geology contract](planetary-geology.md) for codec, profiles, compatibility and tests.
Vesper remains a homogeneous silicate interpretation with unchanged published
terrain/resources; rendering and later environmental systems retain their owners.


## Geological surface materials (#613)

Follow the [geological material contract](geological-materials.md) for terrain-bound
profile mapping, immutable material cache keys, the 52-byte vertex layout,
filtered stratification and original software-Vulkan evidence. Runtime owns
geology-to-DTO mapping; renderer owns shading only. Vesper and resource palettes,
physical queries, geometry and save identities are preserved.


## Regional climate and weather (#615)

Follow the [climate contract](planetary-climate.md) for immutable geology-bound climate inputs,
strict versioned publication and stateless TT epoch queries. World owns pressure,
temperature, coherent storm/cloud/precipitation metadata and body-fixed tangent wind;
presentation cannot rewrite them. Vacuum has no airborne effects. Existing terrain,
resource identities, saves and drag physics remain unchanged. Weather visuals are #616.


## Active geological sources (#617)

The [active geology contract](active-geology.md) owns explicit heat/mechanism inputs,
stable terrain-bound sites, strict publication restore and shared-TT source levels.
Spatial queries cannot create/rekey sources. Vacuum permits ballistic plumes and
contact coupling, never atmospheric wind/sound. Source radii do not add colliders.


## Rare biospheres and ecological regions (#618)

Follow the [ecology contract](planetary-ecology.md) for independent fixed life decisions,
geology/climate-bound potential, continuous realms and stable patch/grammar IDs.
Suitability never grants life; observed bodies retain no procedural life claim
and Vesper stays sterile. Strict catalog/save bindings reject incompatible
publications. Organism meshes, physical instances and harvesting remain #619–#621.


## Hybrid astronomical sky (#423)

The [implemented sky contract](universe-sky.md) maps the universe source/streaming,
runtime projection and renderer pass/test owners. Full ephemeris-frame adoption,
full source coverage and human/hardware release qualification remain separate.


## Optical atmosphere implementation (#614)

See [planetary optical atmosphere](planetary-atmosphere.md) for source/test ownership,
physical inputs, bounded depth-aware scattering and exact fixture/native limits.
Clouds/weather presentation remains #616; human/hardware qualification is separate.


## Coherent weather presentation (#616)

[Implemented weather visuals](planetary-weather-visuals.md) route authoritative
shared-time samples through runtime lighting and fixed production renderer
fields. Acoustic integration remains #627 and hardware/human qualification separate.


## Organism family presentation (#619)

[Procedural organism families](planetary-organisms.md) map the world grammar,
stable cosmetic surface tiles, runtime adapter and bounded renderer/test owners.
Physical colliders/landing safety are #620; biological material response is #622.


## Physical organisms and clearance (#620)

[Physical organism placement](physical-organisms.md) owns stable source/part
bindings, root/exclusion rules, common physics sweeps and live character/ship
clearance. Vesper remains sterile; mutable harvesting/removal remains #621.


## Scanner cloud integration (#438)

See the [scanner acceptance contract](universe-scanner.md#cloud-scanner-acceptance-438). The existing action bar places fresh scanner status top-right at 1280+ physical pixels to leave the projector visible at simulated 2x scale; compact windows and chart mode retain their existing bottom placement. Renderer owns generic `TopRight` image placement; runtime chooses placement from viewport width without changing scanner, input, collision or saved authority. The required scanner suite and exact frozen-frame/pixel review preserve failure evidence and physical/hardware qualification limits.


Ordinary-play isolation (#427): [canonical runtime and prototype-save gate](universe-gameplay.md#ordinary-canonical-play-427).


## Suit observations (#625)

See [suit measurements and field marks](suit-observations.md) for source/validation
ownership, F10/F11/F12, configuration, schema 27 and preserved scanner boundaries.


## Personal planetary chart (#626)

[Planetary chart](planetary-chart.md) maps personal history, physical inspection,
native text/keys, uncertainty presentation and schema-28 validation to owners/tests.


## Planetary audio (#627)

[Source and verification contract](planetary-audio.md): runtime `environment_audio.rs`
feeds the existing native continuous mixer from local world definitions. Run its
portable regressions, production callback export and native environment-audio scenario.


## Biological harvesting (#621)

[Source, frame, shape, transaction and schema contract](biological-harvesting.md).
World `environment/harvest.rs` and `biological_session.rs` own source/mass rules;
runtime `session/biology.rs` composes input, current anatomy and checkpoint restore.
Run world/runtime harvest tests, full gates and both native biological scenarios.


## Canonical render acceptance (#436)

See the [cloud render acceptance contract](universe-terrain.md#cloud-integration-acceptance-436) for the required
`universe-render` suite, native frozen/revisit pixel checks, canonical stitched
seam tests and bounded raw frame/draw observations. Measurements remain
observational; missing GPU counters stay null and software timings do not
establish hardware FPS.
