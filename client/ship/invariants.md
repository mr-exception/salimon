# Ship Invariants

1. The default ship is landed on Earth with its door closed and zero thruster.
2. The exit door can change state only while landed; it is always closed in flight.
3. A flying door interaction emits exactly `Door locked while in flight` through
   typed cockpit-message state.
4. Leaving cockpit control never changes orientation, thruster, or current speed.
5. Flying direct-speed motion continues while cockpit control is inactive.
6. Authoritative position and orientation math remains CPU-side `f64`.
7. The crate owns no platform input, GPU resources, or character state.
8. The default Earth pose places the authored scout mesh's lowest local-Y point on the
   nominal surface with local `+Y` aligned to the outward normal.
9. Sustained pitch, yaw, and roll each use exactly 5 degrees/second after a
   0.12-second start/stop presentation ramp; leaving the cockpit clears input.
10. Thruster adjustments require cockpit authority, change by explicit integer
    percentage points, clamp to 0–100%, and map directly to speed.
11. Flying ships are corrected outside every solid body's nominal surface plus
    the conservative ship collision radius; the visual-only Sun is ignored.
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
    independent of frame partition or the direct-flight 100 ms clamp.
18. Assist activation preserves the pose. Landing levels smoothly over 2 seconds
    before descending; takeoff preserves landed orientation. Every radial phase
    begins and ends at rest, and the final pose is sampled exactly once at completion.
19. Landing reserves 2 seconds for its last 15 m or less; takeoff reserves 2 seconds
    for its first 15 m, so local surface motion stays readable on every solid body.
