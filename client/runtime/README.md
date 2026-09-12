# Runtime

The `salimon-client` binary is the native client composition entry point. It owns
the `winit` application lifecycle, native window, redraw scheduling, resize
routing, recoverable surface-loss handling, monotonic frame/update clocks, and
client composition. It drives `salimon-character`, `salimon-ship`,
`salimon-world`, `salimon-renderer`, and `salimon-diagnostics` but does not own their domain, GPU, or aggregation
implementation.

Run from the repository root with `cargo run --locked -p salimon-client`.
The process requests a physical 1920×1080 initial drawable for the Phase 0
benchmark, opens a resizable native window, and runs until the window is closed.

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
- Advance portable character, ship, and camera-fixture state with a bounded
  monotonic delta. Reset only the update clock across lifecycle gaps; preserve
  portable state across renderer reconstruction.

Press **F3** to show or hide the engineering diagnostics overlay. It is hidden by
default and remains a developer view rather than a normal gameplay HUD. Runtime
frame observations feed the diagnostics crate after each successful present;
the prior published image is supplied to the renderer on the next frame. Timing
history resets across suspension, occlusion, zero-sized drawables, and renderer
reconstruction so pauses do not contaminate FPS data.

Gameplay view is the default. WASD/Space/mouse events are translated to typed
character input. Looking at the cockpit while close to its seat presents
`Press E to use`; E performs contextual cockpit/door interaction. F2 toggles
the camera fixture, which approaches automatically from a six-body compressed
Solar System overview to Earth's meter-scale surface markers and retreats in a loop.
Escape releases the mouse-look cursor lock and a click in the game view captures
it again.
Press **P** on its initial key press to
pause/resume the transition, **R** to restart it at the far endpoint, and **N**
to select the exact 12 m near-surface dwell and pause it for inspection. The
keys **1–6** select Sun, Mercury, Venus, Earth, Moon, and Mars, respectively,
and restart that body's tour. Earth is the initial target; R retains selection.
The runtime translates those `winit` events into typed world commands; raw platform
events never cross the world boundary.

While seated in the cockpit, W/S pitch, A yaws left, D yaws right, and
Left/Right Arrow roll.
Up/Down Arrow change direct-speed thruster power by one percentage point on each
initial press. The current practical metric speed and percentage are published
as cockpit-monitor information, while mouse look remains independent and never
steers or recenters the ship. Leaving with E clears held steering but preserves
heading, thruster, and autonomous forward motion.

While controlling the cockpit, L starts assisted landing whenever the ship is
inside a solid body's `1.15R` volume. When landed, the same key starts takeoff;
the door must be closed. Contextual prompts and interlock/progress messages use
the cockpit-monitor message path. Automatic sequences continue after leaving
the cockpit, and runtime composition switches character radial gravity to the
active Mercury, Venus, Earth, Moon, or Mars surface frame.

Task 10 maps contextual interaction zones to the enlarged cockpit and exit-door
markers. It retains the 0.05 m gameplay near plane and passes the baked-scale GLB
to the renderer without an extra transform.

## Boundaries

Keep behavior in its owning portable crate. The runtime retains character, ship,
and camera objects only as the composition root. It maps the snapshot's six bodies to
`SphereInstance` values with generic material styles, maps the Sun to `PointLight`,
keeps the three precision-marker cuboids separate, maps the ship snapshot to a
generic mesh instance, and supplies diagnostics with nonnegative camera-to-body
surface observations for every catalog body; diagnostics displays the closest.
`winit` integration remains here until platform-specific behavior justifies an
adapter under `client/platform/`. The runtime must not acquire GPU, backend,
networking, or persistence responsibilities in Phase 0.

See [README.ai.md](README.ai.md), [architecture.md](architecture.md), and
[invariants.md](invariants.md) before changing lifecycle or timing behavior.
Use the root [native smoke check](../../README.md#native-smoke-check) to validate
launch, resize, minimize/restore, close, and relaunch behavior.
