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
external consumers still observe `ShipSnapshot`. `clear_steering_input` discards
both target and ramp on runtime lifecycle/view discontinuities without changing
pose, momentum or selected throttle. Dependencies are unchanged.

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
less to the corresponding tangent surface pose. Takeoff lifts 10 m in 2 seconds, then rises another 40 m in 4 seconds using
smoothstep segments. It ends 50 m above the captured landed origin at rest,
with throttle zero and immediate powered manual control. The stages peak at
7.5 and 15 m/s respectively, each with acceleration magnitude at most 15 m/s²; position and velocity are
continuous at the join and handoff. This local clearance is independent of the
planetary landing-eligibility shell. There is no free-flight planetary gravity,
so zero-throttle flight remains stationary until thrust is selected. Captured radial waypoints and elapsed
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
the 50 m local takeoff rise clears this sphere at the validated landed site.
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
The complete starter dry ship is 100,000 kg, including its registered portable shelf.
The permanent hull minimum subtracts that 2 kg accessory when offboard. `Propulsion` validates cached total mass and
one fictional impulse coefficient, initially 1 N·s/J (#493). Forward/reverse
power limits are 100/50 MW; only allocated propulsion joules generate signed
local +X impulse, giving 1000/500 m/s² dry acceleration. No independent force
capacity or kinetic-energy charge exists. `set_propulsion` is an event-driven handoff
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

Ship's `ShipMass` caches the 100,000 kg complete starter dry ship, subtracts
registered dry equipment only while offboard, and adds positive finite physical masses
by stable `MassObjectId` (separate fragment/equipment namespaces). Cargo uses
an incremental error-free partial-sum expansion separate from dry hull mass, so
replacement/removal retains small residuals without scanning objects. An empty
ledger resets to the exact current dry mass; invalid/overflowing updates are transactional.
This numerical rule is owned here; runtime only drains physical events. Upsert/removal
is idempotent; handling and storage do not add another identity. Installed
components use the same ledger. The registered portable shelf is included in
starter dry mass and has a separate 2 kg source parameter for physical boundary
transfers; it is never also inserted as cargo. Other dry equipment has no
separately movable mass definition in this revision. `reconciles` is an explicit diagnostic,
not flight work. `set_total_mass_kg` preserves the impulse coefficient and velocity.

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
Landing completes Landed with velocity and throttle zero. Takeoff ends at rest after a 50 m local rise, hands the same zero analytic
velocity to Flying, and sets throttle zero. It remains inside the landing
eligibility shell; a powered pilot can immediately command thrust or land again. Updates split at completion and apply remaining time
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
Full reverse estimates use configured reverse watts, impulse coefficient and mass, with energy including idle
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


## Accepted review and future boundaries (#197)

The [2026-10-14 gameplay review](../../docs/spaceship-building-and-survival.md#accepted-propulsion-and-core-review-2026-10-14)
records all gameplay choices and final supersessions. Review decisions 42/78
belong to the [timing contract](../../docs/technical-architecture.md#flight-update-and-validation-decisions-197).
Mass has no steering/recoil effect; starter tuning is configurable rather than
an endurance guarantee. Future force/efficiency upgrades, landing speed/range/
cost upgrades, extra module consumers, gravity/orbits, manual power management
and detailed inertial cabin forces are OUT OF SCOPE. Existing setters are
integration/tuning seams, not shipped upgrade mechanics.


## Consumable raw resources (#337)

See the canonical [resource catalog and compatibility contract](../world/resource-contracts.md#consumable-raw-resources-337) for new feedstock generation, physical lifecycle, distinct presentation and Core yields.

## Continuous Fabricator power (#340)

Runtime registers the runnable head recipe's watts with ShipController before
every shared gameplay tick, regardless of view, player location or flight state.
The Core settles conversion, propulsion, idle and Fabricator demand together.
Every ordinary bus consumer shares the exact funded interval, while reserved
assist energy remains isolated. Runtime floors that interval to nanoseconds
(no free progress; less than one nanosecond of paid work can be lost at an
exhaustion boundary). No blackout time is banked. Recipe completion splits the
tick and removes its load immediately; idle and blocked tray demand zero watts.
Core telemetry exposes Fabricator watts separately and includes them in total
draw, depletion and braking-energy estimates. Recipe efficiency is watts times
actual worked duration; there is no second battery or assumed full powered frame.

## Logical save boundary (#364)

The [canonical save inventory](../../docs/save-contract.md) defines schema-1
source state and excluded caches. Domain-owned save state is validated before
restore; runtime composes temporary owners and rebuilds mass, immutable terrain
queries and presentation/query/solver caches without advancing simulation.
Stable IDs, depletion, carrying, motion, Core/assist progress and logical
contact/sleep/dwell state survive. No input latch, OS handle or GPU state is
restored. An attached mining sphere becomes a detached existing physical entity,
without allocating a new ID or losing mass. Whole-session/Fabricator publication
is composed in #365; disk checkpoints remain #366.

## Explicit pilot revocation (#369)

`revoke_pilot_control` is a separate authority-loss command: select zero manual
throttle and clear cockpit/steering/held inputs, preserving pose, world velocity
and committed assist progression. Runtime uses it for exhausted/terminal pilots.
Ordinary `set_cockpit_control(false)` still preserves selected throttle. Recovery
requires a new cockpit entry and fresh controls; no hidden thrust resumes.

## Funded Core impulse and engine switch (#493)

The engine change is implemented in this revision. For each power-funded segment,
`E_propulsion = requested_propulsion_watts * funded_seconds`; only that share of
ordinary-bus allocation creates `signed_delta_v = kappa * E_propulsion / mass`.
Idle, cockpit, Fabricator and assist reservations never create free-flight impulse.
Powered displacement uses `v*dt + delta_v*dt/2`; unpowered remainder coasts. The
same forward/reverse configured watt limits determine instantaneous acceleration
and braking. Numerical coefficient validation admits finite 0..=1000 N·s/J so
extreme finite energy divided by the dry hull mass cannot overflow; this is not
a speed cap or an implemented upgrade progression. The default remains exactly 1.

Held throttle repeats split at their true 100ms boundaries, billing the preceding
selection before changing it. Conversion completion, depleted-bus intervals and
assist completion preserve existing funding splits. Orientation at the admitted
substep directs only new impulse. Inertia, mass accounting, steering and swept
collision remain production authorities.

The dry-hull change is 25× legacy 4/2 MN: 100/50 MN at 100/50 MW, or 1000/500 m/s².
Landing eligibility stays <=20 m/s. Takeoff still eases 50 m over six seconds,
peaks at 15 m/s and 15 m/s², and hands off at rest with throttle zero. Isolated
600/800/1200 MJ assist reservations remain conservative kinematic power-time
budgets, not a second free-flight impulse. Portable all-body/mass/energy/handoff
and collision tests protect these choices; native handling acceptance is #498.

B toggles the explicit `engines_enabled` switch with powered cockpit authority
outside active assists/bounces. Off preserves selected throttle and velocity,
zeros propulsion draw/acceleration and makes powered braking unavailable. Idle
loads/steering remain electrical systems. Power restoration never turns an
explicitly off engine on; switched-on selected throttle still resumes after
blackout. An on engine at zero throttle remains distinct. Assistance activation
requires the engine switch on; committed assists cannot be interrupted by B.

Schema 5 persists the switch and coefficient; runtime migrates v1-v4 legacy
force capacities to coefficient 1 and engine-on without changing energy, velocity
or cargo source. Invalid/unknown/missing current-schema fields reject atomically.

Future Jump Drive is a separate ship module powered by the same authoritative Core. It computes a virtual delta-v = 0.20 * (c - pre_jump_speed) **per light-second** (c=299,792,458 m/s) and quotes total cost = distance_light_seconds * mass_kg * virtual_delta_v_mps / (1 N·s/J). It translates the entire ship along its current forward direction while retaining **actual** world-frame position-independent velocity and orientation; no direct boost, braking or clock manipulation. **Reject while propulsion engines are switched on**, even at zero throttle. The implemented `engines_enabled` state supplies this distinction; the Jump Drive interlock itself remains future work. Above/equal light speed is currently an explicit refused/undecided case to avoid zero/negative costs; destination safety, quantum loading staging and crash-safe Core debit belong to future Jump Drive work, not issue #493. Future offline ships coast on one shared 1:1 clock; they do not jump automatically.

## Low-priority data recovery energy (#468)

`fund_auxiliary_work` spends only actual stored, unreserved Core energy above
a caller-supplied emergency floor after ordinary ship/Fabricator settlement.
Invalid requests are atomic; no auxiliary joule creates propulsion impulse.
Last-tick auxiliary watts are transient observation, included in total draw
without another debit and cleared on subsequent advance/restore. Runtime owns
record progress/config and the `RecoveryModule` physical mass ledger entry. See
[canonical funding/priority contract](../../docs/data-recovery-module.md#core-funding-and-reserve).

## Portable structural foundation (#541)

`structure/` owns `ShipStructure`, independent of the live flight controller,
renderer, character traversal and future power/pressure graphs. `grid.rs` defines
3 m ship-local cells (+X forward, +Y up, +Z port), checked six-face neighbors and
explicit position queries. The caller supplies a precise `f64` origin; existing
Scout anchors, generated layouts, collision radius and dry hull are unchanged.
No implicit grid migration or 0.5 m furnishing rule is introduced.

`structure/mod.rs` admits connected multi-cell `StructuralPart`s into sparse,
ordered indexes. Each part has a stable nonzero ID, frame/full/tapered/corner
shell hint, one space envelope and conservative collision envelopes. These are
portable authored bounds, not generated mesh colliders or final placement
admission. Face metadata models walls, floors, ceilings, openings and doorways;
a shared face has one owner. Removing a face preserves its underlying frame and
has no salvage/mass transaction. `Room`s group connected existing cells without
another mass contribution. Multi-cell equipment overlays supporting structure;
equipment footprints reserve cells against other equipment, while room/frame
membership remains distinct. Fine placement/overlap and physical door behavior
belong to later placement/runtime owners.

Normal removal refuses the permanent Core, a cut disconnecting any remaining
cell, or a part referenced by room/equipment metadata. Damage can remove a
non-Core part and partition the rest. The returned `DamageOutcome` transfers
separate disconnected components with their original IDs, unsupported equipment
and invalidated room metadata. It never creates another Core or silently deletes
loose equipment. Physics, salvage, hull damage parameters and recovery gameplay
remain #549. IDs removed/detached from the parent are retired, preventing reuse.

`StructuralMass::IncludedInDryHull` explicitly marks baseline authored structure
already covered by the starter's 100 t dry hull; `AddedKilograms` holds validated
positive finite installation mass. `mass_entries` emits only additional part and
equipment contributions in the disjoint `MassObjectId::Structure` namespace.
Rooms/faces do not add duplicate entries. Future runtime installation must
atomically transfer source material ownership before applying these entries;
removal/detachment must remove them. Changing baseline dry-hull damage accounting
is future integration, not an automatic change in this foundation.

`structure/persistence.rs` owns independent domain schema 1. Ordered vector
payloads retain stable IDs, precise origin, parts/faces/envelopes, room/equipment
membership and retired IDs. `from_save` validates a temporary graph, handles IDs
in arbitrary topological order and rebuilds indexes before publication. Unknown
schemas, duplicates, unsupported topology and malformed geometry/mass reject.
`StructureSaveState` implements the existing portable `Validate` boundary.
The current whole-game schema and `ShipController` payload are unchanged;
construction project/session save integration remains #550.

Tests in `structure/tests.rs` map the admission/removal/damage/save and mass
contracts to deterministic headless evidence. No player construction UI,
blueprints, timers, Core upgrades, native hull migration or flight-readiness
policy is enabled by this issue; #542–#550 consume the new boundary.

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

`attachments.rs` owns registered rails, stable mount/child references, compatibility and explicit release on parent removal. `mass.rs` subtracts an existing dry equipment identity only while physically offboard. See [the canonical contract](../../docs/physical-mounts.md).


Scout cabin #577: follow [the canonical source, collision, posture and presentation contract](../../docs/cabin-mechanisms.md). Runtime owns admitted mechanism/posture source; renderer never grants physical authority. Manual mechanisms preserve original belongings and halt on swept obstruction.


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


## Construction damage (#549)

Follow the [canonical damage, physical salvage and schema-21 contract](../../docs/construction-damage.md). Surviving work preserves paid time and receipts; lost cells and destroyed projects cannot respawn or duplicate salvage. Core identity is protected. Fresh debris IDs, exact material shares, collision/containment and installed mass publish together; restore reconstructs source without reallocating debris.
