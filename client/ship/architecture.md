# Ship Architecture

`ShipController` is authoritative portable state. It owns a double-precision
world pose, landed/flying/assisted state, door state, cockpit authority, persistent
thruster percentage, and a typed cockpit message. `ShipSnapshot` is the only
runtime-facing observation needed by character composition, diagnostics, and
renderer mapping.

Orientation is a normalized local-to-world quaternion. `ShipPose::axes` exposes
forward/up/port axes without leaking a math or engine dependency. The current
advance step applies local-axis quaternion steering, direct speed along local +X,
and nearest-valid-position correction against every solid catalog body while
flying. Steering target and smoothed values are portable numeric state; platform
key state remains outside this crate.
Landing captures the arbitrary approach normal, aligns local `+Y` to it, and
moves to the corresponding tangent surface pose. Takeoff follows that same
normal until beyond the landing volume. Runtime maps the active body's frame to
character traversal and owns only the contextual `L` key translation.
The default landed pose derives its vertical clearance from the enlarged asset's
lowest local-Y point, keeping visible geometry tangent to Earth's nominal surface.
