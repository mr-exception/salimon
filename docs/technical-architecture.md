# Technical architecture and AI maintenance

## Implemented architecture

Salimon uses a custom Rust runtime with low-level `winit` and `wgpu`, without a
full game engine. The current prototype extends the Phase 0 baseline and remains native and
client-only. Build scripts and
CI stage macOS, Windows and Debian-based Linux executables; graphical CI runs
on Linux software Vulkan. Native macOS is the reference playable/performance
target. Web/server reuse remains a future capability, not a shipped target.

The [feature maintenance map](maintenance-map.md) routes work to source, tests,
invariants and scenarios. The [client guide](../client/README.md) defines package
dependencies; each crate's architecture/invariants defines its local contracts.

| Boundary | Implemented ownership |
| --- | --- |
| `client/runtime/` | Native lifecycle, input, clocks, domain composition, automation, resource presentation and physical-object frame/session adapters |
| `client/world/` | Static six-body compressed catalog, `f64` coordinates/camera tour, resource contracts, generation, extraction/session deltas, fragments and one-object carrying |
| `client/ship/` | Pose, cockpit authority, direct flight, uncancellable landing/takeoff, airlock interlocks and live telemetry |
| `client/physics/` | Portable small-object gravity, ejection/release velocity, spherical contacts, restitution and deterministic substeps; caller-supplied surfaces/geometry |
| `client/math/` | Dependency-free `f64` vector component arithmetic; no domain/frame ownership |
| `client/character/` | Portable movement, collision, gravity, cockpit transitions and open-space/nearby-body EVA |
| `client/renderer/` | GPU resources, GLB loading, sphere/ship/image pipelines, reverse-Z, instruments and measurements through renderer-neutral DTOs |
| `client/diagnostics/` | Observational metrics, formatting and overlay rasterization |
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
persistence. No orbital simulation, multiplayer, production survival or energy
management is implemented. Core energy is a bounded telemetry fixture.

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
| runtime `app/interaction.rs` (formerly in `app.rs`) | Subtract/dot/length also compatible, but leave these composition helpers outside the limited first migration. |
| world `src/lib.rs`, `mining.rs`, `resource_fragments.rs` | Length/dot/cross are compatible candidates, left local for now to avoid extending the first migration into world ownership. `WorldPosition`, subtraction/rebasing and resource pose validation remain world-owned. |
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
and uses three monitors on one center console with walking routes on both sides.
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
existing domain controllers. The current solver uses equal contact weighting,
spherical proxies and quadratic pairs; it is not a mass-aware rigid-body engine.
Multiple ship frames must be stepped separately; the current single-ship adapter
preserves the old grouping, constants, iteration order and 48-substep cap.
See [physics architecture](../client/physics/architecture.md) and
[invariants](../client/physics/invariants.md) before extending it.

## Future candidates and deferred capabilities

These directions require their own issue, explicit contracts and validation;
they are not dependencies or scaffolding requirements for current work:

| Direction | Status and decision boundary |
| --- | --- |
| Web/WASM and WIT Component Model | Portable Rust reuse and versioned capability interfaces where useful; no component host or WIT boundary currently shipped |
| ECS | Standalone `bevy_ecs` may be evaluated; no canonical ECS or Bevy engine adoption |
| Physics | Rapier remains the preferred candidate for evaluation; current fragments use custom deterministic motion/contact |
| Workers/scheduler/event bus | Evaluate for measured simulation needs; no worker infrastructure required by current native composition |
| Streaming/storage | Larger spatial hierarchies and storage interfaces remain future work; IndexedDB is the browser cache direction, native storage needs a platform implementation |
| Networking/server | Authoritative multiplayer server and shared deterministic domain reuse are future architecture |
| Survival/economy/crafting/NPCs | Product directions, not implemented crates or interfaces |

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
