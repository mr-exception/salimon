# Ship AI Maintenance

Keep this crate independent of `winit`, `wgpu`, character, renderer, and
diagnostics. It may consume stable world identities/constants. Presentation and
input adapters belong to runtime; mesh loading belongs to renderer.

Task 11 deliberately exposes a minimal motion foundation. Do not add Task 12
steering or later assisted-landing sequences opportunistically. Protect cockpit exit,
door locking, monitor-message routing, pose axes, and direct-speed continuation
with unit tests.
