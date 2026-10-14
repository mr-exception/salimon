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
| `flight.rs` | Force-based acceleration, persistent velocity and free-flight integration |
| `throttle.rs` | Signed selection, authority and simulation-time held-key cadence |
| `collision.rs` | Catalog/terrain segment sweeps, damped rebound and crash grounding |
| `steering.rs` | Input clamps, cockpit/flight gate, ramps and local-axis rotations |
| `assist.rs` | Landing range, captured paths, timing/derivatives, takeoff interlocks and all-body assists |
| `door.rs` | Landed/open-space permissions, locked interactions and reversible hinge animation |
| `cockpit.rs` | Control authority and typed/contextual message routing; authority coverage also lives with flight/steering/assist rules |
| `telemetry.rs` | Core display DTO/default/bounds and nearby-body sensor selection/radial speed |

Update order remains door animation, clamped steering, then flight or full-delta
assist dispatch. Module methods use only parent-visible access where needed;
external consumers still observe `ShipSnapshot`. Dependencies are unchanged.

`ShipController` is authoritative portable state. It owns a double-precision
world pose, landed/flying/assisted state, door state, cockpit authority, persistent
thruster percentage, an authoritative configurable joule Core, and a typed
cockpit message. `ShipSnapshot` is the only
runtime-facing observation needed by character composition, diagnostics, and
renderer mapping.

Orientation is a normalized local-to-world quaternion. `ShipPose::axes` exposes
forward/up/port axes without leaking a math or engine dependency. The current
advance step applies local-axis quaternion steering, force-based acceleration along local +X,
and continuous movement-segment sweeps against every solid catalog body while
flying. Steering target and smoothed values are portable numeric state; platform
key state remains outside this crate.
Landing captures the arbitrary approach normal and starting pose. It aligns
local `+Y` over 2 seconds using eased shortest-arc quaternion interpolation,
approaches over 4 seconds, then spends 2 seconds descending the last 15 m or
less to the corresponding tangent surface pose. Takeoff follows that same
normal, lifting 15 m over 2 seconds before clearing the landing volume over
4 seconds. Smoothstep radial segments provide continuous position and velocity,
with zero velocity at internal phase boundaries. Takeoff clearance uses an
accelerating quadratic segment and hands its nonzero analytic terminal velocity
to free flight with throttle zero. Captured radial waypoints and elapsed
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

## Inertial propulsion (#201)

Flying ships own persistent world-space f64 velocity independent of orientation.
The starter dry hull is 100,000 kg; forward/reverse capacities are 4 MN/2 MN.
`Propulsion` validates cached total mass and force capacities and computes signed
local +X acceleration as force/mass. `set_propulsion` is an event-driven handoff
for E01 cargo composition, with no per-tick cargo scans. Constant
acceleration uses v*dt + 0.5*a*dt² and v += a*dt. Zero throttle coasts, steering
changes only future thrust direction, and no drag, speed cap or body gravity is
applied. Snapshot speed is the magnitude of actual velocity, including assists.
`flying` starts at rest; `flying_with_velocity` explicitly seeds finite momentum.
Assisted landing/takeoff retain captured paths and finish at rest. Collision
uses the bounded swept contact and crash-state policy described below.

## Signed held throttle (#186)

Selection is an i8 percentage clamped to −100..+100. Positive/negative values
use forward/reverse force capacity; zero coasts. `set_throttle_key` consumes typed
Up/Down edges, increments one point immediately, then repeats each 100 ms of
simulation time. Repeated presses are idempotent and simultaneous keys cancel
repeat. Release/direction changes reset cadence. Cockpit authority, Flying state
and nonzero authoritative Core energy gate changes; power consumption is owned by E02.
Leaving the cockpit preserves selection and momentum while clearing held keys.
Assists clear holds and reject changes; landing completion/collision zero selection.

## Swept collision and crash settling (#202)

Each free-flight movement segment sweeps every solid catalog body; exact ties
use catalog order, and the visual-only Sun is ignored. Analytic body-local
line/sphere intervals avoid a high-speed endpoint-only test. Terrain extrema
are broad bounds, then version-1 heightfield intervals use analytic gradient
bounds to reject clear spans. Chronological subdivision detects hills even
with both endpoints outside; resolution is 1 mm. Work is capped at 4,096
uncertain intervals per body. Exhaustion conservatively contacts the earliest
unresolved span (a possible early contact, never a pass through solid terrain).
No planet mesh, cargo scan or full physics engine is required.

Contact retains the existing conservative 16 m ship collider and projects its
center to the sampled local-normal clearance, within 1 mm at the distant f64
anchor. This is conservative proxy contact, not triangle-level hull simulation.
Throttle/held input and steering clear immediately. Tangential impact motion
is absorbed; the normal component rebounds at restitution 0.35, capped at
8 m/s, along a captured contact normal. A local 10 m/s² settling response
returns it to the contact anchor; free flight still has no planetary gravity.
Each rebound launch speed decays by 0.35. Below 0.1 m/s or after four rebounds
the ship becomes `GroundedAfterCrash` at zero velocity; captures/timing make
bounce sampling independent of elapsed-time partition. No damage is applied.

`Bouncing { body }` and `GroundedAfterCrash { body }` remain distinct from
`Landed`: ordinary takeoff and throttle commands cannot activate there.
Contextual messages identify settling/crash grounding; runtime maps their
selected body's surface frame without treating the ship as properly landed.
A powered cockpit L action starts reserved recovery after crash grounding;
settling remains locked. #198 owns native M1 collision smoke.

## Physical mass accounting (#187)

Ship's `ShipMass` caches 100,000 kg dry hull plus positive finite physical masses
by stable `MassObjectId` (separate fragment/equipment namespaces). Upsert/removal
is idempotent; handling and storage do not add another identity. Installed
components can use the same ledger; the current asset has no separately defined
installed-equipment masses beyond dry hull. No inventory/container implementation
or invented component weights is added. `reconciles` is an explicit diagnostic,
not flight work. `set_total_mass_kg` preserves force capacities and velocity.

World coalesces creation/growth, pickup, carried/loose pose changes and physical
removal into an ordered mutation journal. `remove_fragment` is the future Core
consumption/despawn boundary and never reuses IDs. Streaming/read-only queries
emit nothing. Runtime drains changes after carrying/physics synchronization,
classifies centers in the current authored cabin/nose frame, and updates cached
mass for the next ship step. Door animation/furniture/handling do not change
containment. Existing physics visits and anchored frame writebacks provide pose
events even for sleeping contents; no extra full scene scan computes mass each
tick. Explicit scene reconciliation is available for validation. Full inertial
cabin motion, arbitrary future physical items and Core intake are separate tasks.

## Authoritative Core power (#188)

`energy.rs` owns validated `EnergyConfig`, fractional f64 joule storage and atomic
energy transfers. Starter capacity is 60 GJ at 75% charge. Free-flight draw is
100 MW times positive throttle, 50 MW times negative magnitude; zero throttle
has no propulsion draw. Configurable gravity/cockpit idle loads default to
100 kW/10 kW independently of cockpit occupancy and flight mode. The ten-minute
full-capacity estimate is tuning, not a fixed endurance invariant.

All consumers share a bus. Energy determines an exact powered interval followed
by coasting when exhausted, preserving selection and world velocity. Empty Core
rejects throttle/steering/assist activation, blanks the display and disables cabin
artificial gravity without adopting planetary gravity. Held controls clear;
charging automatically resumes selected throttle on the next ship step. Cockpit
occupancy remains a physical state, separate from power authority.

`exchange_core_energy_joules(incoming, requested)` is the E03 atomic conversion
handoff. Incoming joules serve the requested load before excess storage is
capped; the result reports actual delivered energy. It is an explicit energy
transaction, not a persistent charging-rate assumption. Idle/propulsion draw is
performed by `advance`; callers must not debit those consumers a second time.
Invalid config/transactions leave previous state intact. Whole-joule snapshots
truncate fractional energy; `power_available` is authoritative even below 1 J.
Captured assists remain uncancellable, with isolated reserved budgets and
powered crash recovery as described below.

`advance_with_core_input(delta, incoming_joules)` settles converted energy at a
step boundary with that step's normal idle/propulsion consumers before storage
clipping. It preserves exact powered-thrust duration. Conversion timing/rate is
owned by E03; no continuous charging source is assumed by this task.

## Assisted energy reservation (#203)

Activation atomically removes full forward watts × captured assist duration from
the ordinary Core bus (starter landing 800 MJ, takeoff 600 MJ). This is a
conservative configurable tuning estimate for the current kinematic paths, not
a physical work calculation. Only the active assist spends the isolated budget;
idle drains and conversion use ordinary storage. Bus blackout cannot cancel the
sequence. Completion refunds unused budget with capacity clipping. Repeated
activation does not reserve twice; insufficient storage rejects before changing
the path/flight state. The same portable reservation contract is available to
the powered recovery path, which reserves 12 seconds (starter 1.2 GJ).

## Mechanical Core intake and conversion (#189)

Normal E targeting within 3 m of the authored Core center, aimed within 30°,
accepts the one carried fragment indoors, including blackout/zero-G. No
cockpit authority or electricity is required. Ship validates finite mass ×
material yield before removing the stable identity from the world session;
runtime immediately drains removal events into the mass ledger and clears
physical/presentation caches. Busy intake leaves the next fragment untouched.

Ship owns a single automatic process, 200 MJ/s by default. Catalog gameplay
yields default to 1 GJ/kg iron, 0.5 GJ/kg silicate, 2 GJ/kg ice; these are tuning
values, not chemistry. `ConversionConfig` permits finite nonnegative yields
and a positive finite rate. Zero yield consumes immediately. Generated joules
settle with ordinary discharge before capacity clipping; excess is wasted and
full batteries keep processing to completion. Updates split at process completion
so later idle time is not incorrectly charged by an already finished fragment.

## Cockpit blackout authority (#192)

Cockpit occupancy is physical; commands require powered cockpit ownership and
free flight. Steering uses the exact powered interval, stops at energy depletion,
and clears target/ramp/held-key residue at outage or cockpit exit. Signed throttle,
heading and momentum persist. Charging/conversion automatically resumes selected
nonzero thrust, including while the cockpit is empty; manual control still needs
a powered pilot. Assists remain authoritative and reject manual overrides.
Native and automation steering share gates; blackout/assist/exit clears the runtime
latch, and OS repeat/focus replay cannot reinstall a stale steering press.

## Physical Core status (#194)

`ShipSnapshot::core_status` owns live ordinary-bus consumption plus isolated
assist consumption, current conversion watts/yield remaining, low-energy state
(10% or less), and the estimate of ordinary-bus time to depletion. The estimate
assumes unchanged loads and the currently accepted fragment only, accounts for
finite conversion and wasted surplus at capacity, and excludes reserved assist
joules from ordinary storage. Zero drain has no depletion estimate. Fractional
storage powers the display even when whole-joule telemetry rounds to zero.

## Safe assisted landing and recovery (#190)

`AssistConfig` validates configurable activation range (starter 0.15R above the
nominal surface) and total planet-relative speed limit (starter 20 m/s). Static
catalog bodies have zero velocity, so total world velocity is planet-relative.
Tangential motion counts; heading does not. Only powered cockpit authority may
activate; full energy is reserved before state/path changes. Active sequences
continue through cockpit exit and ordinary-bus outage.

Crash-grounded recovery reserves 12 seconds: 2 seconds lifting with unchanged
orientation, 2 seconds relocating on a spherical shell above the body's maximum
terrain plus the conservative 16 m ship radius and 15 m margin, then standard
8-second alignment/approach/touchdown. The immutable existing bounded site search
checks authored feet/ramp/hull support at a 25 m grid over ±500 m in each tangent
axis (at most 1681 candidates). No safe site rejects before energy debit; no
power leaves the ship grounded. Rotation begins only after clearance.

All assistance snapshots report analytic vector velocity, including relocation.
Landing completes Landed with velocity and throttle zero. Takeoff clearance
accelerates smoothly from rest and hands its analytic terminal outward velocity
to Flying, throttle zero. Updates split at completion and apply remaining time
to normal flight/idle consumption; partitioned inertial world-coordinate updates
retain normal distant-anchor f64 quantization. Door interlocks are unchanged.

Future landing upgrades may increase activation speed/range or reduce energy
cost; upgrade progression and modifiers are documentation only in this wave.

## Vector navigation and braking (#193)

Ship snapshots own ship-local unit velocity direction (undefined at rest),
heading/velocity angle, signed instantaneous engine acceleration and an explicit
reverse-stop estimate. Free-flight power and cached total mass determine actual
engine acceleration; assists/crash states report it unavailable. Static catalog
bodies have zero velocity, so nearby total relative speed equals vector magnitude.
Full reverse estimates use configured reverse force, mass and watts including idle
loads. They require nose alignment within 1e-9 transverse unit-vector magnitude;
misalignment, no power, non-flight and zero reverse force never claim a stop.
Required energy is compared with unreserved fractional storage; future conversion
is not credited. Estimates assume unchanged heading/mass and collision-free travel,
and never command steering or braking.

The existing wide console shows velocity XYZ in ship-local axes (+X forward,
+Y up, -Z starboard), angle, signed throttle/acceleration, kg mass, nearby total
speed and reverse duration/distance/GJ. Unavailable readings use --; insufficient
energy and alignment have explicit status. Rounded strings independently cache
flight/braking/navigation regions. Core-local screen ownership is unchanged.
