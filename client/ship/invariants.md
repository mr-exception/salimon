# Ship Invariants

1. The default ship is landed on Earth with its door closed and zero thruster.
2. The exit door can change state only while landed; it is always closed in flight.
3. A flying door interaction emits exactly `Door locked while in flight` through
   typed cockpit-message state.
4. Leaving cockpit control never changes orientation, thruster, or current speed.
5. Flying direct-speed motion continues while cockpit control is inactive.
6. Authoritative position and orientation math remains CPU-side `f64`.
7. The crate owns no platform input, GPU resources, or character state.
8. The default Earth pose places the Task 10 mesh's lowest local-Y point on the
   nominal surface with local `+Y` aligned to the outward normal.
9. Sustained pitch, yaw, and roll each use exactly 5 degrees/second after a
   0.12-second start/stop presentation ramp; leaving the cockpit clears input.
10. Thruster adjustments require cockpit authority, change by explicit integer
    percentage points, clamp to 0–100%, and map directly to speed.
11. Flying ships are corrected outside every solid body's nominal surface plus
    the conservative ship collision radius; the visual-only Sun is ignored.
