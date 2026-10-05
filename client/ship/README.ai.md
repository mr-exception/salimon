# Ship AI Maintenance

Keep this crate independent of `winit`, `wgpu`, character, renderer, and
diagnostics. It may consume stable world identities/constants. Presentation and
input adapters belong to runtime; mesh loading belongs to renderer.

Task 13 adds the portable assisted-landing/takeoff state machine on top of Task
12 motion. Protect cockpit-only activation, exact landing range, arbitrary
surface normals, uncancellable automatic completion, door-interlocked takeoff,
cockpit exit, monitor-message routing, direct-speed continuation, steering, and
collision correction with unit tests.

Task 17 captures each assist's starting pose and radial waypoints, then evaluates
the path from elapsed `Duration`. Landing is 8 seconds (2 alignment, 4 approach,
2 touchdown); takeoff is 6 seconds (2 local lift, 4 clearance). Keep the local
15 m phases readable across every catalog radius and preserve smooth position,
orientation, and velocity at phase boundaries. Assist timing must consume the
full supplied delta independently of the existing 100 ms direct-flight clamp.
Tests cover every solid body, multiple normals and starting heights, frame
partitioning, exact completion, controls/interlocks, and velocity derivatives.
Allow for submillimeter `f64` position quantization at the catalog's 1e12 m anchor.

Keep Core values as a bounded, noncanonical Phase 0 telemetry fixture until an
energy-management task defines production behavior. Ship snapshots own actual
flight/assist velocity and nearby-body telemetry; runtime must only map it.

## Shared maintenance rules

Follow the root [coding conventions](../../docs/coding-conventions.md),
[validation matrix](../../docs/validation.md) and
[feature map](../../docs/maintenance-map.md). Update affected contracts/guides
with behavior changes and record completion evidence under root `reports/`.
