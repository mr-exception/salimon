# Runtime

The `salimon-client` binary is the native client composition entry point. It owns
the `winit` application lifecycle, native window, redraw scheduling, resize
routing, recoverable surface-loss handling, monotonic frame/update clocks, and
client composition. It drives `salimon-world`, `salimon-renderer`, and
`salimon-diagnostics` but does not own their domain, GPU, or aggregation
implementation.

Run from the repository root with `cargo run --locked -p salimon-client`.
The process opens a resizable native window and runs until the window is closed.

## Lifecycle contract

- Create window-bound rendering state only after the native application is
  ready to resume.
- Route nonzero drawable-size changes to the renderer and avoid configuring or
  drawing a zero-sized surface while minimized.
- Schedule continuous redraws while the window is active.
- Rebuild or reconfigure rendering state after recoverable surface loss and exit
  cleanly for a close request or unrecoverable GPU failure.
- Measure frames from a monotonic clock in the runtime. The renderer must not
  become the owner of simulation time.
- Advance the portable camera fixture with a bounded monotonic delta before
  rendering the static Task 5 world. Reset only the update clock across lifecycle
  gaps; preserve the portable camera state across renderer reconstruction.

Press **F3** to show or hide the engineering diagnostics overlay. It is hidden by
default and remains a developer view rather than a normal gameplay HUD. Runtime
frame observations feed the diagnostics crate after each successful present;
the prior published image is supplied to the renderer on the next frame. Timing
history resets across suspension, occlusion, zero-sized drawables, and renderer
reconstruction so pauses do not contaminate FPS data.

The camera fixture approaches automatically from a six-body compressed Solar
System overview to Earth's meter-scale surface markers and retreats in a loop.
Press **P** on its initial key press to
pause/resume the transition, **R** to restart it at the far endpoint, and **N**
to select the exact 12 m near-surface dwell and pause it for inspection. The
runtime translates those `winit` events into typed world commands; raw platform
events never cross the world boundary.

## Boundaries

Keep world/camera behavior in `salimon-world`. The runtime retains the portable
camera object only as the composition root. It maps the snapshot's six bodies to
colored radius-scaled cuboid `SceneInstance` values, appends the three separate
precision markers, and supplies diagnostics with nonnegative camera-to-body
surface observations for every catalog body; diagnostics displays the closest.
`winit` integration remains here until platform-specific behavior justifies an
adapter under `client/platform/`. The runtime must not acquire GPU, backend,
networking, or persistence responsibilities in Phase 0.

See [README.ai.md](README.ai.md), [architecture.md](architecture.md), and
[invariants.md](invariants.md) before changing lifecycle or timing behavior.
Use the root [native smoke check](../../README.md#native-smoke-check) to validate
launch, resize, minimize/restore, close, and relaunch behavior.
