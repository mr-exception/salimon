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
21. Validated cached mass/force inputs determine acceleration; speed telemetry is
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
    changes preserve velocity and thrust capacities; empty cargo restores exact dry
    mass and finite residuals survive replacement/removal. Validation reconciliation
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


The [accepted review](../../docs/spaceship-building-and-survival.md#accepted-propulsion-and-core-review-2026-10-14)
supersedes historical direct-speed/no-energy baseline prose. Force/energy tuning
is configurable; no ten-minute endurance invariant, steering inertia, reaction
mass, upgrade system or orbital gravity is implied. GroundedAfterCrash and Landed
remain distinct until powered recovery completes (final decisions 79–83).
