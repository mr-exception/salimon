# Ship Architecture

`ShipController` is authoritative portable state. It owns a double-precision
world pose, landed/flying state, door state, cockpit authority, persistent
thruster percentage, and a typed cockpit message. `ShipSnapshot` is the only
runtime-facing observation needed by character composition, diagnostics, and
renderer mapping.

Orientation is a normalized local-to-world quaternion. `ShipPose::axes` exposes
forward/up/port axes without leaking a math or engine dependency. The current
advance step applies direct speed along local +X only while flying.
The default landed pose derives its vertical clearance from the enlarged asset's
lowest local-Y point, keeping visible geometry tangent to Earth's nominal surface.
