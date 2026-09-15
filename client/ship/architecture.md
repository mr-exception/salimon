# Ship Architecture

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
Landing captures the arbitrary approach normal, aligns local `+Y` to it, and
moves to the corresponding tangent surface pose. Takeoff follows that same
normal until beyond the landing volume. Runtime maps the active body's frame to
character traversal and owns only the contextual `L` key translation.
The default landed pose derives its vertical clearance from the enlarged asset's
lowest local-Y point, keeping visible geometry tangent to Earth's nominal surface.
The wider 20.30 × 4.00 × 20.00 m hull uses a conservative 15 m collision sphere;
this same clearance is included when completing automatic takeoff.
Snapshot velocity reflects direct flight or the active assist sequence. The ship
combines that velocity with world-owned surface-distance/radial helpers to report
the nearest of all six catalog bodies within an inclusive 3,000,000 m range.
