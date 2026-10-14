# Ship Architecture

## Source ownership

`src/lib.rs` is the public facade: existing types, constants and methods keep
their crate-root paths. `state.rs` owns pose, axes, steering input, door/flight
states and snapshot DTOs. `orientation.rs` owns normalization, quaternion
composition, surface alignment and interpolation with its regressions.

`controller/mod.rs` keeps authoritative fields private and owns construction,
update dispatch and snapshot composition. Its child modules implement rules on
that same controller, without adding controllers, crates or framework layers:

| Module | Ownership and adjacent regression coverage |
| --- | --- |
| `flight.rs` | Thruster authority, direct speed/velocity, motion and solid-body correction |
| `steering.rs` | Input clamps, cockpit/flight gate, ramps and local-axis rotations |
| `assist.rs` | Landing range, captured paths, timing/derivatives, takeoff interlocks and all-body assists |
| `door.rs` | Landed/open-space permissions, locked interactions and reversible hinge animation |
| `cockpit.rs` | Control authority and typed/contextual message routing; authority coverage also lives with flight/steering/assist rules |
| `telemetry.rs` | Core fixture DTO/default/bounds and nearby-body sensor selection/radial speed |

Update order remains door animation, clamped steering, then flight or full-delta
assist dispatch. Module methods use only parent-visible access where needed;
external consumers still observe `ShipSnapshot`. Dependencies are unchanged.

`ShipController` is authoritative portable state. It owns a double-precision
world pose, landed/flying/assisted state, door state, cockpit authority, persistent
thruster percentage, a bounded noncanonical Core telemetry fixture, and a typed
cockpit message. `ShipSnapshot` is the only
runtime-facing observation needed by character composition, diagnostics, and
renderer mapping.

Orientation is a normalized local-to-world quaternion. `ShipPose::axes` exposes
forward/up/port axes without leaking a math or engine dependency. The current
advance step applies local-axis quaternion steering, direct speed along local +X,
and nearest-valid-position correction against every solid catalog body while
flying. Steering target and smoothed values are portable numeric state; platform
key state remains outside this crate.
Landing captures the arbitrary approach normal and starting pose. It aligns
local `+Y` over 2 seconds using eased shortest-arc quaternion interpolation,
approaches over 4 seconds, then spends 2 seconds descending the last 15 m or
less to the corresponding tangent surface pose. Takeoff follows that same
normal, lifting 15 m over 2 seconds before clearing the landing volume over
4 seconds. Smoothstep radial segments provide continuous position and velocity,
with zero velocity at phase boundaries. Captured radial waypoints and elapsed
`Duration` make sampling independent of frame partition, without accumulated
position error. Automatic sequences consume the full supplied delta; the
runtime supplies shared 16 ms gameplay ticks. Runtime maps the active body's frame to
character traversal and owns only the contextual `L` key translation.
The default and assisted landed poses use the generated landing-foot offset.
Flat fixtures place the non-gear hull 2 m above the nominal tangent surface.
Enabled terrain uses terrain_landing.rs and runtime-supplied authored footprint
samples to preserve that clearance and reject unsupported fixed feet/ramp sites. `landing_geometry.rs` is generated from the same authored GLB
as the character collision layout.
The current 18.33 × 4.00 × 20.00 m hull uses a conservative 16 m collision sphere;
this same clearance is included when completing automatic takeoff.
Snapshot velocity reflects direct flight or the analytic derivative of the active
assist segment, including zero during alignment and at rest. The ship
combines that velocity with world-owned surface-distance/radial helpers to report
the nearest of all six catalog bodies within an inclusive 3,000,000 m range.

## Shared vector primitives

Compatible `f64` component arithmetic comes from the dependency-free
`salimon-math` leaf crate. Normalization, frame and quaternion policies stay
with their owning domain. See the canonical
[decision and inventory](../../docs/technical-architecture.md#shared-math-decision-115).

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
