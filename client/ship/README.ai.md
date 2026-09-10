# Ship AI Maintenance

Keep this crate independent of `winit`, `wgpu`, character, renderer, and
diagnostics. It may consume stable world identities/constants. Presentation and
input adapters belong to runtime; mesh loading belongs to renderer.

Task 8 deliberately exposes a minimal motion foundation. Do not add Task 9
steering or Task 10 landing sequences opportunistically. Protect cockpit exit,
door locking, monitor-message routing, pose axes, and direct-speed continuation
with unit tests.
