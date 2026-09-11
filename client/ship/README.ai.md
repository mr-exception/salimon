# Ship AI Maintenance

Keep this crate independent of `winit`, `wgpu`, character, renderer, and
diagnostics. It may consume stable world identities/constants. Presentation and
input adapters belong to runtime; mesh loading belongs to renderer.

Task 12 extends the motion foundation with portable steering, discrete thruster
adjustment, and solid-body boundary correction. Do not add later assisted-landing
sequences opportunistically. Protect cockpit exit, door locking, monitor-message
routing, pose axes, direct-speed continuation, fixed steering rate/ramp, and
collision correction with unit tests.
