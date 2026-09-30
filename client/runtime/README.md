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

## Deterministic E2E launch

For declarative end-to-end tests, see [the scenario runner](../../scripts/README.md).

Pass `--e2e` after `--` to opt into reproducible initial conditions. For example:

```sh
cargo run --locked -p salimon-client -- --e2e --scenario orbit-moon --seed 42 --step-ms 16
```

Scenarios: `landed-earth` (default, standing in the ship), `cockpit-earth`
(seated in the landed ship), `orbit-earth`, and `orbit-moon` (seated in a flying
ship, 1 km above the nominal surface). The world catalog is immutable; the seed
selects a reproducible tangent offset of at most 100 m for orbit scenarios.
`--seed` defaults to 0. `--step-ms` defaults to 16 and accepts 1–100 milliseconds.
The command channel advances simulation only when asked to step; redraws do not
advance it. Ship flight, doors, character motion, and collision remain on their
normal controller paths. The process still needs a native window and renderer.

Wait for the stdout line `SALIMON_E2E_READY scenario=<name> seed=<n> step_ms=<n>`
before sending input. It is emitted after window and renderer initialization;
the next stdout line is a versioned JSON ready event. Send newline-delimited JSON
requests on stdin, one at a time, and read one JSON response per request. The
legacy ready line is not a JSON response. Example:

```json
{"protocol":1,"id":1,"op":"key","key":"forward","pressed":true}
{"protocol":1,"id":2,"op":"step","frames":10}
{"protocol":1,"id":3,"op":"key","key":"forward","pressed":false}
{"protocol":1,"id":4,"op":"inspect"}
```

`key` accepts `forward`, `backward`, `left`, `right`, `jump`, `roll_left`, and
`roll_right`. Held forward/backward/left/right steer the ship when seated, and
move the character otherwise. `look` accepts finite pixel deltas `dx` and `dy`.
`interact` uses the same aimed/range interaction as E, so walking and looking at
the cockpit seat is required to enter it; interacting again leaves it. `landing`
uses the same L action and its cockpit and proximity gates. `thruster` accepts a
`direction` of `1` or `-1` and applies one Up/Down Arrow step when seated. `step`
accepts 1–600 `frames` of the launch-configured duration. `inspect` returns
player pose/location, ship pose/flight/door/control/telemetry, the aimed
interaction target, and all catalog bodies with ship surface distances.
`player.ship_local_eye_position_meters` expresses the eye in the current ship
frame for every location, including surface/doorway positions where
`player.local_ship_position_meters` is null. This allows door-collision
assertions without subtracting large world coordinates in scenario files.

Responses include `protocol`, the caller's string or numeric `id`, `ok`, and
either `result` or `error` with `code` and `message`. Requests need protocol 1,
an ID, and an operation. Lines are limited to 16 KiB. A command waiting longer
than five seconds gets a structured timeout; expired queued commands are dropped.
Only E2E launches start a stdin reader; ordinary launches do not consume stdin.
The JSON channel does not expose mutable scenario state after setup.

Setup or GPU/window failure exits nonzero with a diagnostic on stderr/log output.
Close the window to finish the run. Scenario flags without `--e2e`, malformed
values, and unknown scenarios fail before opening a window. Normal launches use
the original monotonic update clock and default starting state.

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
character input. Looking at the cockpit while close to its seat from inside the
ship presents `Press E to use`; E performs contextual cockpit/door interaction.
The doorway and surface positions can still operate the landed door but cannot
enter cockpit control. F2 toggles the camera fixture, which approaches
automatically from a six-body compressed Solar System overview to Earth's
meter-scale surface markers and retreats in a loop.
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
on the physical cockpit screens through `ShipMeshInstance::instruments` on every
gameplay frame, including while the player walks away from the seat. The port
screen shows live stored/capacity Core energy; the starboard screen shows the
nearest body within 3 Mm, its surface distance, and approaching/receding/zero
radial speed, or a clear out-of-range state. Mouse look
remains independent and never steers or recenters the ship. Leaving with E clears
held steering but preserves heading, thruster, and autonomous forward motion.
The speed screen preserves the ship snapshot's direct-speed reading. Separate
snapshot velocity keeps nearby radial telemetry accurate during assisted landing
and takeoff without redefining the direct-speed field.

While controlling the cockpit, L starts assisted landing whenever the ship is
inside a solid body's `1.15R` volume. When landed, the same key starts takeoff;
the door must be closed. Contextual prompts and interlock/progress messages remain
in the native window title. Automatic sequences continue after leaving
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
generic mesh instance with live renderer-owned `CockpitInstruments` data, and
supplies diagnostics with nonnegative camera-to-body surface observations for
every catalog body; diagnostics displays the closest.
`winit` integration remains here until platform-specific behavior justifies an
adapter under `client/platform/`. The runtime must not acquire GPU, backend,
networking, or persistence responsibilities in Phase 0.

See [README.ai.md](README.ai.md), [architecture.md](architecture.md), and
[invariants.md](invariants.md) before changing lifecycle or timing behavior.
Use the root [native smoke check](../../README.md#native-smoke-check) to validate
launch, resize, minimize/restore, close, and relaunch behavior.
