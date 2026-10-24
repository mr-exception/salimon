# Ship AI Maintenance

Keep this crate independent of `winit`, `wgpu`, character, renderer, and
diagnostics. It may consume stable world identities/constants. Presentation and
input adapters belong to runtime; mesh loading belongs to renderer.

Start at `src/lib.rs` for the stable public reexports and constants. Public
pose/snapshot DTOs live in `state.rs`; `controller/mod.rs` owns private state,
constructors, update ordering and snapshots. Change rules in the focused
`controller/{flight,steering,assist,door,cockpit,telemetry}.rs` owner and keep its
regressions alongside it. `orientation.rs` owns local quaternion/vector policy.

The portable assisted-landing/takeoff state machine builds on inertial
flight motion. Protect cockpit-only activation, exact landing range, arbitrary
surface normals, uncancellable automatic completion, door-interlocked takeoff,
cockpit exit, monitor-message routing, inertial continuation, steering, and
continuous collision and crash settling with unit tests.

Each timed assist captures its starting pose and radial waypoints, then evaluates
the path from elapsed `Duration`. Landing is 8 seconds (2 alignment, 4 approach,
2 touchdown); takeoff is 6 seconds (10 m local lift over 2 seconds, then 40 m
over 4 seconds), with both segments ending at rest. Landing eligibility range
is not a takeoff waypoint; see the [takeoff correction](../../docs/spaceship-building-and-survival.md#local-takeoff-and-powered-handoff-295).
Recovery adds 2 seconds of safe lift and 2 seconds of reposition before landing. Keep the local
local phases readable across every catalog radius and preserve smooth position,
orientation, and velocity at phase boundaries. Assist timing must consume the
full supplied delta independently of the shared runtime fixed-tick scheduler.
Tests cover every solid body, multiple normals and starting heights, frame
partitioning, exact completion, controls/interlocks, and velocity derivatives.
Allow for submillimeter `f64` position quantization at the catalog's 1e12 m anchor.

Core joule storage and watt consumers are authoritative in `energy.rs`; keep
fractional joules internally and map whole-joule telemetry only at snapshots. Ship snapshots own actual
flight/assist velocity and nearby-body telemetry; runtime must only map it.

## Shared maintenance rules

Follow the root [coding conventions](../../docs/coding-conventions.md),
[validation matrix](../../docs/validation.md) and
[feature map](../../docs/maintenance-map.md). Update affected contracts/guides
with behavior changes and record completion evidence under root `reports/`.

## Terrain integration (#179–#181)

Follow the canonical [integration/budgets/validation guide](../../docs/terrain-validation.md)
and [world definition contract](../world/terrain-contract.md). Keep local rules
and their consumers aligned; flat deterministic fixtures are explicit.


For approved flight/Core choices and historical supersession, follow the
[owned review](../../docs/spaceship-building-and-survival.md#accepted-propulsion-and-core-review-2026-10-14).
Thruster/landing upgrades, future module-load/inertial-cabin/orbit/manual-power features remain deferred. The explicit instant Core capacity progression is implemented by #548; tuning setters alone do not grant a paid upgrade.

#493 replaces independent force capacities with funded Core impulse. Read the
[math/safety/switch contract](architecture.md#funded-core-impulse-and-engine-switch-493).
Protect exact consumer allocation, throttle boundaries and engine-off persistence.

## Low-priority data recovery energy (#468)

`fund_auxiliary_work` spends only actual stored, unreserved Core energy above
a caller-supplied emergency floor after ordinary ship/Fabricator settlement.
Invalid requests are atomic; no auxiliary joule creates propulsion impulse.
Last-tick auxiliary watts are transient observation, included in total draw
without another debit and cleared on subsequent advance/restore. Runtime owns
record progress/config and the `RecoveryModule` physical mass ledger entry. See
[canonical funding/priority contract](../../docs/data-recovery-module.md#core-funding-and-reserve).

## Structural foundation (#541)

`src/structure/` owns portable topology, structural metadata and a separate
versioned save boundary. Read [structure architecture](architecture.md#portable-structural-foundation-541)
and its invariants before changing admission/removal/damage or mass mapping.
It is not yet composed into gameplay. Preserve authored coordinates; placement,
physical construction, flight readiness and full-save integration retain their
later issue owners. Run `cargo test -p salimon-ship --locked` and workspace gates
for mass/persistence boundary changes.

## Blueprint placement authority (#542)

`structure/placement.rs` owns catalog validation, exact 3 m/fine-pitch snapping,
quarter-turn transforms, stable non-reused placeholder IDs and geometry
reservations. Runtime supplies trusted owner, flight, physical support,
reachability and current player/cargo/installed/protected-route envelopes.
Admission is repeated at placement and immediately before physical completion;
rendered previews are never authority. One multi-cell/multi-deck volume is one
project. Unbuilt neighbors cannot supply structural support. Interior versus
hull-affecting consequences determine airborne permissions.

Projects capture requirements/duration and exact #543 material receipts/progress.
There is no separate worksite or Start action.
Cancellation atomically returns physical unconsumed material; #543 owns intake/refunds,
#544 progress, #545 geometry and #546 flight interlocks. Domain schema-1 validation
and the whole-session boundary are documented in the
[save contract](../../docs/save-contract.md#blueprint-placeholder-schema-8-542).
Tests live in `structure/placement/tests.rs`.

## Physical mounts (#576)

Read the physical mount contract before changing rail identity/compatibility or existing-equipment dry-mass transfers. See [the canonical contract](../../docs/physical-mounts.md).


Persistent finishes #579: [canonical ownership, schema 13, bounded render factors and controls](../../docs/room-finishes.md). Ship owns room/item cosmetic identity; runtime owns nearby physical control admission; renderer applies the explicit nonfunctional surface allowlist. Cosmetic changes never edit physical material/age/damage.


Instant Core upgrades #548: [canonical input, source, mass and schema-14 contract](../../docs/core-upgrades.md). Ship owns validated capacity-only transactions; world returns original caller-owned reserved entities through the one carry slot; runtime maps the data catalog/physical controls and reconstructs pending+installed material mass. No timer, joule grant, Core replacement or flight command.


Construction receipts/refunds #543: [canonical transaction, mass, controls and schema-15 contract](../../docs/construction-materials.md). Funding respects the explicit economy approval gate; progress/geometry retain #544/#545 owners.


Room and fixture lights #580: [canonical controls, source, effective output and schema-16 contract](../../docs/room-lighting.md). Master/blackout/inoperable lamps suppress output without changing preferences; critical machine displays remain independent.


## Automatic construction (#544)

Follow the [canonical progress/completion contract](../../docs/construction-progress.md). Ship owns validated paid nanoseconds and ppm; runtime owns actual Core funding, current safety and atomic interior/structural installation. No unfinished collider, free progress, duplicate mass or unsupported module completion. Schema 18 restores exact work and validated modular edits without elapsed-time credit; see the completed modular structure contract below.



## Completed modular structure (#545)

Follow the [canonical assembly, traversal, identity, mass and schema-18 contract](../../docs/modular-construction.md). Completed cell parts and face edits regenerate shared render/collision geometry; unpublished projects supply no support or colliders. Preserve the existing gate, per-cell identities, atomic restore and single-owner material ledger.


## Construction flight readiness (#546)

Follow the [canonical Stop/Resume, physical securing, takeoff and schema-19 contract](../../docs/construction-flight-safety.md). The durable stop latch never clears on landing; actual hull/attachment state governs readiness. Preserve Core/engine/cockpit/ramp authority and inertial momentum.


## Cosmetic surface age (#581)

Follow the [persistent exposure, schema-23 and clean-repair contract](../../docs/surface-age.md). Ship owns bounded integer exposure; runtime admits physical identity/use and verified offline time; renderer consumes observations only. Recolor/repair retain age, replacements use fresh IDs, and no cosmetic state grants damage or collision authority.


## Real-scale trajectory foundation (#417)

`trajectory.rs` owns split-position velocity-Verlet integration with the existing paid Propulsion contract and universe gravity. The compressed playable controller and saves remain unchanged; real-scale support, drag and contact compose through the #418–#420 companion APIs below. Read the [trajectory contract](../../docs/universe-trajectory.md) for epoch, mass, error and atomic-failure rules. The direct universe dependency is a portable domain dependency, with no GPU/window coupling.


## Real-scale surface flight (#418)

See the [surface-flight contract](../../docs/universe-surface-flight.md) for rotating/orbiting support,
Core-funded physically bounded landing/takeoff, production source/tests and
remaining playable-world adoption. Current compressed gameplay is unchanged.


## Real-scale atmospheric drag (#419)

See the [atmosphere contract](../../docs/universe-atmosphere.md) for planet-specific density,
co-rotating gas/wind, atomic online/offline drag integration and physically paid
assist compensation. Model validity, tests and playable-adoption limits are explicit.


## Real-scale moving contact (#420)

See the [continuous-contact contract](../../docs/universe-contact.md) for moving solids/terrain,
certified earliest-event queries, atomic gravity/drag interception, body-relative
consequences, funded-work receipts, cloud tests and adoption/precision limits.


## Canonical surface gameplay and schema 24 (#425)

Follow the [canonical surface contract](../../docs/universe-gameplay.md):
selected body/terrain context, immutable U11 deposits, shared physical ground,
conserved fragment/cargo/Fabricator ownership and exact U05 definition binding.
Historical fixtures remain explicit; new-run startup is #426.


## Bounded navigation prediction (#421)

Follow the [prediction/telemetry contract](../../docs/universe-navigation.md): ship owns cloned real-scale coasting and current-controller previews, runtime owns disposable observation caching, renderer owns instrument DTO/rasterization only. Preserve explicit model assumptions, bounded horizon, sampled contact/arrival semantics, empirical convergence (not a certified error bound), visible validity/age and nonmutation of controls, power, source saves and simulation clocks.

## Regional weather composition regression (#615)

`tests/regional_weather.rs` exercises the real world climate evaluator with the
existing U18 gas and drag operators. See the [climate contract](../../docs/planetary-climate.md).
Climate wind is body-fixed; the composition owner must rotate it into inertial
axes exactly once before supplying `Atmosphere::sample`. Density, body motion,
rotation, passive drag and propulsion/Core energy retain their existing owners.
The explicit test atmosphere is a fixture, not a newly published lunar atmosphere.


## Physical organisms and static clearance (#620)

The [physical organism contract](../../docs/physical-organisms.md) maps canonical
placement/part/baseline identity, physics-owned swept boxes, character/ship typed
adapters, runtime residency and matching renderer evidence. Quality never changes
physical populations. Explicit obstruction safety overrides ordinary assist continuation.
