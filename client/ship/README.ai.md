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
Future upgrade/module-load/inertial-cabin/orbit/manual-power features remain out
of scope; do not interpret tuning setters as progression mechanics.

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
