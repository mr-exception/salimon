# Runtime

The `salimon-client` binary is the native client composition entry point. It owns
the `winit` application lifecycle, native window, redraw scheduling, resize
routing, recoverable surface-loss handling, monotonic frame clock, and Task 3
diagnostics composition. It drives `salimon-renderer` and
`salimon-diagnostics` but does not own their GPU or aggregation implementation.

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

Press **F3** to show or hide the engineering diagnostics overlay. It is hidden by
default and remains a developer view rather than a normal gameplay HUD. Runtime
frame observations feed the diagnostics crate after each successful present;
the prior published image is supplied to the renderer on the next frame. Timing
history resets across suspension, occlusion, zero-sized drawables, and renderer
reconstruction so pauses do not contaminate FPS data.

## Boundaries

Keep game and world behavior in their future owning modules. The bootstrap
triangle is presentation-only renderer content, not world state. `winit`
integration remains here until platform-specific behavior justifies an adapter
under `client/platform/`. The runtime must not acquire backend, networking, or
persistence responsibilities in Phase 0.

See [README.ai.md](README.ai.md), [architecture.md](architecture.md), and
[invariants.md](invariants.md) before changing lifecycle or timing behavior.
Use the root [native smoke check](../../README.md#native-smoke-check) to validate
launch, resize, minimize/restore, close, and relaunch behavior.
