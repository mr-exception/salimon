# Ship AI Maintenance

Keep this crate independent of `winit`, `wgpu`, character, renderer, and
diagnostics. It may consume stable world identities/constants. Presentation and
input adapters belong to runtime; mesh loading belongs to renderer.

Task 13 adds the portable assisted-landing/takeoff state machine on top of Task
12 motion. Protect cockpit-only activation, exact landing range, arbitrary
surface normals, uncancellable automatic completion, door-interlocked takeoff,
cockpit exit, monitor-message routing, direct-speed continuation, steering, and
collision correction with unit tests.

Keep Core values as a bounded, noncanonical Phase 0 telemetry fixture until an
energy-management task defines production behavior. Ship snapshots own actual
flight/assist velocity and nearby-body telemetry; runtime must only map it.
