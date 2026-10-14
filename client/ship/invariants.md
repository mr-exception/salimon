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
12. Landing is offered only with cockpit authority inside exactly `1.15R`, works
    for every solid catalog body, and preserves the captured approach location.
13. Landing and takeoff automation cannot be cancelled and continues after the
    cockpit is left; steering and thruster input cannot override it.
14. Takeoff requires cockpit authority and a closed door, follows the landed
    surface normal, and returns control only after clearing the landing volume.
15. Core telemetry is bounded by its 1 TJ fixture capacity and does not
    imply consumption, generation, fuel, persistence, or production energy rules.
16. Nearby-body telemetry selects the nearest catalog surface at an inclusive
    3,000,000 m threshold and reports actual signed radial velocity: negative
    approaching, positive receding, and zero stationary/tangential.
17. Landing lasts 8 seconds and takeoff 6 seconds of supplied update time,
    independent of frame partition or the render frame partition.
18. Assist activation preserves the pose. Landing levels smoothly over 2 seconds
    before descending; takeoff preserves landed orientation. Every radial phase
    begins and ends at rest, and the final pose is sampled exactly once at completion.
19. Landing reserves 2 seconds for its last 15 m or less; takeoff reserves 2 seconds
    for its first 15 m, so local surface motion stays readable on every solid body.

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
    ship and cannot trigger landing/takeoff through the existing L action.
24. Collision work is bounded and conservative on unresolved terrain intervals.
    Rebound speed decreases monotonically; at most four captured rebounds occur.
    Contact settling introduces neither damage nor free-flight planetary gravity.
