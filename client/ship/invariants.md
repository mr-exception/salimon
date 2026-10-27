# Ship Invariants

1. The default ship is landed on Earth with its door closed and zero thruster.
2. The exit door can change state while landed or flying outside nearby-body range;
   nearby flight and active assists lock it closed on interaction.
3. A locked door interaction emits exactly `Door locked while in flight` through
   typed cockpit-message state.
4. Leaving cockpit control never changes orientation, thruster, or current velocity.
5. Flying inertial motion continues while cockpit control is inactive.
6. Authoritative position and orientation math remains CPU-side `f64`.
7. The crate owns no platform input, GPU resources, or character state.
8. The flat default fixture places the authored feet on the nominal surface with
   local `+Y` aligned to the outward normal and 2 m non-gear hull clearance.
9. Sustained pitch, yaw, and roll each use exactly 5 degrees/second after a
   0.12-second start/stop presentation ramp; leaving the cockpit clears input.
10. Thruster adjustments require powered cockpit authority, change by explicit integer
    percentage points, clamp to −100..+100%, and select forward/reverse thrust capacity.
11. Flying movement segments sweep all solid bodies and terrain using the
    conservative collision radius. Terrain extrema are broad bounds only;
    local-normal clearance remains precise. The visual-only Sun is ignored.
12. Landing is offered only with powered cockpit authority inside inclusive starter `1.15R`
    and total body-relative speed ≤20 m/s (including tangential motion), works
    for every solid catalog body, and preserves the captured approach location.
13. Landing and takeoff automation cannot be cancelled and continues after the
    cockpit is left; steering and thruster input cannot override it.
14. Takeoff requires cockpit authority and a closed door, follows the landed
    surface normal, and returns control at rest after 50 m of local surface clearance.
15. Core storage is bounded by configurable capacity (starter 60 GJ, 75% charge).
    Fractional joules and watt consumers are authoritative; telemetry is display-only.
16. Nearby-body telemetry selects the nearest catalog surface at an inclusive
    3,000,000 m threshold and reports actual signed radial velocity: negative
    approaching, positive receding, and zero stationary/tangential.
17. Landing lasts 8 seconds and takeoff 6 seconds of supplied update time,
    independent of frame partition or the render frame partition; recovery takes 12 seconds.
18. Assist activation preserves the pose. Landing levels smoothly over 2 seconds
    before descending; takeoff preserves landed orientation. Every radial phase
    joins at rest, including takeoff completion; the same zero analytic velocity
    is handed to inertial flight with throttle zero. Completion consumes leftover time.
19. Landing reserves 2 seconds for its last 15 m or less; takeoff reserves 2 seconds
    for its first 10 m, so local surface motion stays readable on every solid body.

## Source ownership

The public facade in `src/lib.rs` reexports the same contracts. `state.rs` owns
pose/snapshot DTOs; `controller/mod.rs` keeps state private and preserves update
order. Flight, steering, assist, door, cockpit and telemetry rules live in the
corresponding `controller/` child module, with regression tests beside their
owners. Orientation policies and regressions live in `orientation.rs`. See
[architecture](architecture.md#source-ownership) for the detailed map.

## Terrain integration (#179–#181)

Follow the canonical [integration/budgets/validation guide](../../docs/terrain-validation.md)
and [world definition contract](../world/terrain-contract.md). Keep local rules
and their consumers aligned; flat deterministic fixtures are explicit.

20. Flying velocity is authoritative world-space f64 momentum; orientation and
    cockpit authority never reset it. Zero throttle coasts without drag or a cap.
21. Validated cached mass and funded propulsion joules determine impulse; speed telemetry is
    the actual velocity magnitude, including captured assist paths.

22. Up/Down presses change one signed percentage point immediately and repeat
    every 100 ms of simulation time while held. OS repeats/focus replay do not
    create edges. Opposing held keys suspend repeats. Exit, focus/lifecycle loss,
    power loss and assists clear held input; selection persists until landing
    completion or collision resets it to zero.

23. Contact zeros throttle and starts bounded normal rebounds, then stable
    `GroundedAfterCrash` at rest. Crash states never count as an ordinary landed
    ship; L can start powered recovery from GroundedAfterCrash, while ordinary
    takeoff requires a completed proper landing.
24. Collision work is bounded and conservative on unresolved terrain intervals.
    Rebound speed decreases monotonically; at most four captured rebounds occur.
    Contact settling introduces neither damage nor free-flight planetary gravity.

25. Physical cargo mass is cached by stable identity exactly once regardless of
    carrying/storage. Invalid/zero masses are rejected transactionally. Mass
    changes preserve velocity and impulse coefficient; empty cargo restores exact dry
    mass for the equipment currently aboard and finite residuals survive replacement/removal. Validation reconciliation
    never runs in flight updates.

26. Energy exhaustion stops free-flight force at the exact cutoff and preserves
    selected throttle/world velocity. Charging resumes that selection automatically.
27. Empty Core powers down cockpit controls/display and artificial cabin gravity;
    no planetary gravity takes over inside. Atomic exchanges debit consumers
    before clamping surplus to capacity. Invalid energy inputs are transactional.

## Assisted energy reservation (#203)

Activation atomically removes full forward watts × captured assist duration from
the ordinary Core bus (starter landing 800 MJ, takeoff 600 MJ). This is a
conservative configurable tuning estimate for the current kinematic paths, not
a physical work calculation. Only the active assist spends the isolated budget;
idle drains and conversion use ordinary storage. Bus blackout cannot cancel the
sequence. Completion refunds unused budget with capacity clipping. Repeated
activation does not reserve twice; insufficient storage rejects before changing
the path/flight state. The same portable reservation contract is available to
the powered recovery path; the full 12-second reservation is 1.2 GJ by default.

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
Landing completes Landed with velocity and throttle zero. Takeoff rises 50 m locally, with peak speed 15 m/s and peak acceleration
15 m/s² independent of planetary radius/landing eligibility. It hands zero
analytic terminal velocity to Flying, throttle zero. Manual thrust is available
immediately with powered cockpit authority. The ordinary 1.15R landing prompt
can therefore remain available at handoff. Updates split at completion and apply remaining time
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
Full reverse estimates use configured reverse power, impulse coefficient and mass, with energy including idle
loads. They require nose alignment within 1e-9 transverse unit-vector magnitude;
misalignment, no power, non-flight, engine-off and zero reverse power/coefficient never claim a stop.
Required energy is compared with unreserved fractional storage; future conversion
is not credited. Estimates assume unchanged heading/mass and collision-free travel,
and never command steering or braking.

The existing wide console shows velocity XYZ in ship-local axes (+X forward,
+Y up, -Z starboard), angle, signed throttle/acceleration, kg mass, nearby total
speed and reverse duration/distance/GJ. Unavailable readings use --; insufficient
energy and alignment have explicit status. Rounded strings independently cache
flight/braking/navigation regions. Core-local screen ownership is unchanged.


The [accepted review](../../docs/spaceship-building-and-survival.md#accepted-propulsion-and-core-review-2026-10-14)
supersedes historical direct-speed/no-energy baseline prose. Force/energy tuning
is configurable; no ten-minute endurance invariant, steering inertia, reaction
mass, upgrade system or orbital gravity is implied. GroundedAfterCrash and Landed
remain distinct until powered recovery completes (final decisions 79–83).

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

## Fabricator mass transitions (#341)

The runtime updates two dedicated cached ledger keys for integer raw reserves
and the one physical tray item. Every machine command and completion synchronizes
immediately; zero contributions remove their keys. World fragment consumption
drains the same physical journal before the input command returns, so carrying
never creates a second cargo identity. Enqueue removes committed ingredients
from accounted mass deliberately, collection removes tray mass, and cancellation
never refunds. Queue/in-progress/consumable inventory contribute zero. Existing
loose cargo and Core-consumed identity accounting retain their owners. Runtime
UI/inventory adapters must use execute_fabricator and validate receiving slots
before collection; no consumable collection UI is introduced by this issue.

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

- Explicit survival pilot revocation (#369) selects zero manual throttle and
  clears cockpit/held steering/throttle authority without braking world momentum
  or cancelling committed assists. Ordinary cockpit exit remains invariant 4.

## Future impulse and Jump Drive invariants (not enforced by current client)

See [approved travel design](../../docs/jump-drive-and-interstellar-travel.md) and [engine implementation issue #493](https://github.com/mr-exception/salimon/issues/493). Current force/watt values and historic default-Earth compressed fixture invariants above are current implementation facts, **not** the new real-scale future homeworld or a claim the Jump Drive exists.

Future engine: funded propulsion energy in joules yields impulse with baseline conversion **1 N·s/J**; projected delta-v equals vector impulse / ship mass; coasting consumes zero propulsion joules, and depleted Core yields zero new engine impulse. Upgrade/tuning and exact split-step tests must preserve energy/mass/velocity authority.

Future Jump Drive: retain one shared time clock, no velocity-dependent time dilation; owner-only command; explicitly switched-**off** engines (even zero throttle while switched on rejects), no active assists, initial **N=20%**, valid 0<=speed<c, charge = D_light_seconds * mass_kg * 0.20*(c-speed_mps)/(1 N·s/J); no zero-distance jump. Validate region safety/charge and apply exactly once. The quantum loading transition does not advance fictitious travel time; entire ship/passengers/cargo translate along forward axis while world-frame velocity/orientation are preserved. Do not auto-jump offline or expose undiscovered chart positions; at/above-c behavior, safe departure zones and progression economics require further approval.

## Funded-impulse propulsion (#493)

Only ordinary-bus propulsion joules generate impulse at default 1 N·s/J. Service,
Fabricator and reserved assist joules are excluded. No independent force capacity
or kinetic-energy debit; zero funding gives zero impulse. Engine-off is durable
and distinct from zero throttle; restoration/refueling cannot enable it. B and
assist activation share powered cockpit gates; committed paths stay uncancellable.
Throttle repeat boundaries bill the previous selection for its actual interval.
See the [current math/safety contract](architecture.md#funded-core-impulse-and-engine-switch-493).

## Low-priority data recovery energy (#468)

`fund_auxiliary_work` spends only actual stored, unreserved Core energy above
a caller-supplied emergency floor after ordinary ship/Fabricator settlement.
Invalid requests are atomic; no auxiliary joule creates propulsion impulse.
Last-tick auxiliary watts are transient observation, included in total draw
without another debit and cleared on subsequent advance/restore. Runtime owns
record progress/config and the `RecoveryModule` physical mass ledger entry. See
[canonical funding/priority contract](../../docs/data-recovery-module.md#core-funding-and-reserve).

## Portable structure (#541)

- Structural cells use exactly 3 m on each ship-local axis and six-face support.
  Edge/corner contact alone never attaches a part. Room, traversal, pressure and
  power relationships never substitute for structural support.
- All admitted cells are connected to the one permanent Core-bearing part.
  New parts and equipment have internally connected, nonempty, duplicate-free
  footprints. Conservative envelope coordinates and additional masses are finite.
- IDs are nonzero and unique across parts, rooms, equipment and retired IDs;
  mutation/restore rejects invalid input without publishing partial state.
- Ordinary removal cannot disconnect retained structure, remove the Core or
  orphan existing room/equipment membership. Damage transfers each disconnected
  component once with original identity; unsupported metadata/equipment is
  returned explicitly. Destroying the Core is outside this API.
- Baseline structure marked `IncludedInDryHull` adds no mass. Additional structure
  and equipment emit one `MassObjectId::Structure` contribution each; rooms,
  faces and carried input do not create another installed contribution.
- Domain schema 1 restores source state and retired IDs, rebuilding indexes;
  it neither mutates the live flight controller nor changes the whole-game save.
- Current Scout precision anchors/colliders remain authored data. No existing
  asset position is silently rounded or moved to this grid.

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

One item can belong to at most one mechanical mount. Existing equipment is included in dry mass, never counted again as cargo; boundary removal/return is idempotent. See [the canonical contract](../../docs/physical-mounts.md).


Scout cabin #577: follow [the canonical source, collision, posture and presentation contract](../../docs/cabin-mechanisms.md). Runtime owns admitted mechanism/posture source; renderer never grants physical authority. Manual mechanisms preserve original belongings and halt on swept obstruction.


Persistent finishes #579: [canonical ownership, schema 13, bounded render factors and controls](../../docs/room-finishes.md). Ship owns room/item cosmetic identity; runtime owns nearby physical control admission; renderer applies the explicit nonfunctional surface allowlist. Cosmetic changes never edit physical material/age/damage.


Instant Core upgrades #548: [canonical input, source, mass and schema-14 contract](../../docs/core-upgrades.md). Ship owns validated capacity-only transactions; world returns original caller-owned reserved entities through the one carry slot; runtime maps the data catalog/physical controls and reconstructs pending+installed material mass. No timer, joule grant, Core replacement or flight command.


Construction receipts/refunds #543: [canonical transaction, mass, controls and schema-15 contract](../../docs/construction-materials.md). Funding respects the explicit economy approval gate; progress/geometry retain #544/#545 owners.


Room and fixture lights #580: [canonical controls, source, effective output and schema-16 contract](../../docs/room-lighting.md). Master/blackout/inoperable lamps suppress output without changing preferences; critical machine displays remain independent.


## Automatic construction (#544)

Follow the [canonical progress/completion contract](../../docs/construction-progress.md). Ship owns validated paid nanoseconds and ppm; runtime owns actual Core funding, current safety and atomic interior/structural installation. No unfinished collider, free progress, duplicate mass or unsupported module completion. Schema 18 restores exact work and validated modular edits without elapsed-time credit; see the completed modular structure contract below.

Atomic room installation: a `RoomAssembly` has exactly one new structural part per
room cell and a separate room identity. All parts plus membership publish together,
including their existing face/collision/mass source. Rejected installation neither
consumes identities nor mutates the graph. Existing dry hull remains untouched.
The primitive grants no work completion, material receipt or flight authority.


## Completed modular structure (#545)

Follow the [canonical assembly, traversal, identity, mass and schema-18 contract](../../docs/modular-construction.md). Completed cell parts and face edits regenerate shared render/collision geometry; unpublished projects supply no support or colliders. Preserve the existing gate, per-cell identities, atomic restore and single-owner material ledger.


## Construction flight readiness (#546)

Follow the [canonical Stop/Resume, physical securing, takeoff and schema-19 contract](../../docs/construction-flight-safety.md). The durable stop latch never clears on landing; actual hull/attachment state governs readiness. Preserve Core/engine/cockpit/ramp authority and inertial momentum.


## Construction damage (#549)

Follow the [canonical damage, physical salvage and schema-21 contract](../../docs/construction-damage.md). Surviving work preserves paid time and receipts; lost cells and destroyed projects cannot respawn or duplicate salvage. Core identity is protected. Fresh debris IDs, exact material shares, collision/containment and installed mass publish together; restore reconstructs source without reallocating debris.


## Offline construction (#550)

Follow the [canonical schema-22 clock, energy, safety and atomic launch contract](../../docs/construction-offline.md). Runtime owns chronological work and verified-event admission; ship owns closed power without impulse or new engine/assist authority. Saved scene/cargo and survival active sources remain frozen; timed service, Fabricator, Recovery and eligible construction settle before verified publication. Universe/scanner/trajectory event production remains #433.


## Cosmetic surface age (#581)

Follow the [persistent exposure, schema-23 and clean-repair contract](../../docs/surface-age.md). Ship owns bounded integer exposure; runtime admits physical identity/use and verified offline time; renderer consumes observations only. Recolor/repair retain age, replacements use fresh IDs, and no cosmetic state grants damage or collision authority.


## Real-scale trajectory foundation (#417)

`trajectory.rs` owns split-position velocity-Verlet integration with the existing paid Propulsion contract and universe gravity. Runtime has not adopted it; current controller, saves, assists and contact remain unchanged. Read the [trajectory contract](../../docs/universe-trajectory.md) for epoch, mass, error and atomic-failure rules. The direct universe dependency is a portable domain dependency, with no GPU/window coupling.
