# Issue #464 — Scout machinery feedback

## Outcome and ownership

Portable presentation delivered on top of #368 completion PR #488. Runtime maps
existing ship/Core/Fabricator observations to immutable renderer-neutral inputs;
no gameplay controller, elapsed admission, event bus or animation save is added.
[Canonical behavior/ownership](../../docs/scout-presentation.md#machinery-feedback-boundary-464).

Engine drive glow now follows powered free-flight thrust magnitude (including
reverse), extinguishing during coasting/blackout. Separate running lamps retain
bus power. Core/charge surfaces reflect stored charge, with a restrained
four-second conversion modulation. Fabricator panels show a progress-positioned
teal work sweep, quiet idle or steady amber blocked output. Rudder pedals follow
actual ramped yaw in opposition with <=1.5 cm travel; cockpit ready lamps follow
power/control authority. Existing screen text/status and prompts remain the
nonanimated alternatives. No particles, camera shake or flashing alerts.

Existing door/ramp hinge/extension and presented ship/character/cargo transforms
are reused. Landing supports remain fixed authored structure; no unsupported
retraction or collision movement is invented. Output still appears deliberately
on the tray and survives blackout. Paused/dead active time freezes the Core
phase; Fabricator work uses actual job progress and cannot catch up a hidden
visual clock after interruption. Restore directly samples current state.

## Validation — Linux cloud, 2026-10-21

Rust 1.99.1, debug/unoptimized profile, checked-in assets, locked dependencies.
The parent is `eb37dfeb1f0e77aad46f6ebd990cc64ab4e23a0b` (PR #488), based on
main `b79601afc9cee60f6a434189d4242dc04f21617e`.

| Check | Result / evidence |
| --- | --- |
| `cargo test --workspace --locked` | PASS: 703 passed, 23 opt-in ignored; [raw output](workspace-tests.txt) |
| `cargo build --workspace --locked` | PASS: [raw output](workspace-build.txt) |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | PASS: [raw output](workspace-clippy.txt) |
| `cargo fmt --all -- --check` / `git diff --check` | PASS |
| Offscreen production ship pipeline | PASS: 14 synthetic states, repeated PNGs byte-identical; [raw output](gpu-check.txt), [adapter](adapter.txt), [raw timing CSV](timings.csv), [summary](summary.json) |

The offscreen command (absolute output directory required) is documented in the
canonical contract. It exercises the real checked-in GLB importer, bind/vertex
layout, ship shader, opaque/glass pipelines, atlas and readback. Frozen pairs
match exactly; Core idle/conversion/blackout, Fabricator work progression/blocked,
coasting/thrust and open/closed ramp produce different images. These are explicit
synthetic DTO fixtures, with matching synthetic instrument text, not a recorded
production gameplay run. CPU tests independently use real Fabricator enqueue,
funded progress, output blocking, blackout/restoration, pause, save/restore,
ramped yaw and read-only automation inspection. Existing workspace tests retain
ramp/cargo/interaction/power/mass constraints.

Initial checks exposed an off-panel work-sweep position; the actual authored panel
bounds and real job progress now position it correctly. A new yaw test initially
put a flying ship at ground contact and correctly lost control on collision;
the fixture now starts 100 m above the surface. Final checks above pass.

## Structural and measured cost

Exactly 19,479 expanded vertices / 6,493 triangles and two draws remain. One f32
role per vertex adds 77,916 bytes (1,402,488 -> 1,480,404 bytes of ship geometry).
Two vec4 slots extend the uniform 160 -> 192 bytes/frame. No additional textures,
buffers, pipelines, geometry objects or draw calls; the existing objects grow.
No changes to authored GLB/source/colliders or shader-driven physics.

The ship-only offscreen llvmpipe/Mesa Vulkan run uses 1280x800 and 16 frames per
case. Raw samples include the two initial frames; summaries exclude them,
leaving 196 measured frames. CPU prepare median/p95/max: 153.4935 / 1381.669 /
9883.715 microseconds. GPU median/p95/max: 26.892966 / 195.355537 / 569.156138 ms.
These are noisy software-rendered debug diagnostics, with all tails retained;
they are not a frame-budget pass, FPS target, before/after comparison, normal-play
frame time, or M1 measurement. Native matched costs remain unrun.

## Reference images and known limitations

| Reference | Evidence |
| --- | --- |
| Core state pairs | [Idle](images/core-idle.png), [converting](images/core-converting.png), [blackout](images/core-blackout.png) |
| Fabricator states | [Idle](images/fabricator-idle.png), [work A](images/fabricator-working-a.png), [work B](images/fabricator-working-b.png), [blocked](images/fabricator-blocked.png), [blackout](images/fabricator-blackout.png) |
| Drive | [Coasting](images/engine-coasting.png), [forward](images/engine-forward.png), [reverse](images/engine-reverse.png) |
| Ramp/cockpit | [Closed](images/door-closed.png), [open](images/door-open.png), [cockpit](images/cockpit-active.png) |

The Fabricator reference exposes mirrored screen text. This is retained as a
known reference/UI orientation defect for #466/#489; pixel repeatability grants
no text-readability or baseline approval. No human mask/tolerance or image
promotion has been performed. Ordinary display/input, a filmed complete
interaction walkthrough and reference Apple M1 appearance/performance were not
run in this cloud session. They remain explicitly tracked in
[#489](https://github.com/mr-exception/salimon/issues/489), which owns remaining
acceptance before any native signoff. Future scanner feedback remains with
#430/#466 because no scanner hologram is implemented yet.

Closing #464 records portable delivery and scope transfer, not passing these
remaining native/physical checks. No M1 signoff is claimed.
