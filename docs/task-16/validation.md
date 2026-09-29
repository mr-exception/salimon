# Task 16 — central cockpit monitor

[GitHub issue](https://github.com/mr-exception/salimon/issues/25)
— implementation recorded on 2026-09-19. Status: **In Progress**, awaiting the
remaining native visual acceptance checks.

## Change

The complete center monitor assembly is uniformly scaled by 0.7 about its
existing mounting-stem attachment `[4.426, 0.80, 0.0]` meters. This includes the
screen, housing, stem, bezel, fasteners, keys, and indicator. The side monitors
and dashboard retain their previous geometry.

| Measurement | Before (asset version 7) | After (asset version 8) |
| --- | --- | --- |
| Screen width × height | 1.80 × 0.680 m | 1.26 × 0.476 m |
| Assembly width × height × depth | 2.040 × 0.895 × 0.247 m | 1.428 × 0.6265 × 0.1729 m |
| Assembly top above ship origin | 1.695 m | 1.4265 m |
| Screen plane X | 4.280 m | 4.3238 m |

The combined console/monitor collision proxy covers `[4.3126, 0.25, -1.06]`
through `[6.12, 1.4265, 1.06]` meters. The character controller uses that footprint
expanded by the player's radius, preserving the existing conservative planar
collision model. The GLB contains final meter coordinates; no runtime scale or
camera change is required. Screen UVs, atlas resolution, telemetry formatting,
contextual action handling, and two ship draw calls are preserved.

## Automated validation

Rust/Cargo 1.89.0, native Apple Silicon macOS:

- `python3 client/assets/ship/source/generate_salimon_phase0_ship.py` — passed.
- `python3 client/assets/ship/source/validate_salimon_phase0_ship.py` — passed:
  5,494 triangles, 106 primitives, 13 materials, 468,388-byte GLB.
- `cargo build --workspace --locked` — passed.
- `cargo fmt --all -- --check` — passed.
- `cargo clippy --workspace --all-targets --locked -- -D warnings` — passed.
- `cargo test --workspace --locked` — all 150 working-tree tests passed.
- An isolated export of the staged tree, excluding pre-existing uncommitted
  doorway changes, also passed all 141 tests.

Asset checks validate every center-assembly vertex against the 0.7 transform,
unchanged side/dashboard geometry, normals/UVs/topology, fitted collision
metadata, and unobstructed seated-eye display rays. Renderer regression coverage
checks the imported final dimensions, positions, normals, and UV fit of all three
displays. Character regression coverage checks walking and jumping into the
center assembly's pilot-facing edge.

## Visual evidence and remaining checks

Native baseline images were captured at the original 1920×1080 drawable size
(macOS scale factor 2; the capture service returns 960×572 window images):

- [Standing baseline](before-standing.jpg): default spawn/exit viewpoint, looking
  forward from the starboard aisle.
- [Seated baseline](before-seated.jpg): normal cockpit camera at
  `[2.76, 1.799032258064516, 0]`, with the default slight downward pitch.

A temporary startup fixture called the normal character/ship cockpit-entry
methods for repeatable seated captures. That fixture was removed and is not
part of the commit. Normal startup remains standing inside the ship.

The updated [seated source preview](../../client/assets/ship/previews/cockpit-seated.png),
[pilot station](../../client/assets/ship/previews/cockpit-station.png), and
[six-view sheet](../../client/assets/ship/preview.jpg) show the new geometry.
These are software-rendered source previews, not native runtime evidence.

The native after-capture session opened a blank surface despite successful
Apple M1/Metal renderer initialization. Relaunching the saved **before** binary
reproduced the same blank surface; minimize/restore did not recover it. The
capture service also returned ScreenCaptureKit error `-3811` (audio/video
capture failure). The cause is unresolved; this is not evidence of a successful
visual check or a monitor-specific rendering regression.

Before marking Done, capture matching native seated and standing after views,
confirm forward visibility and legibility while using free-look, exercise live
speed/thruster readings and contextual messages, and complete the repository's
native Solar System/precision/resize/minimize/close/relaunch smoke checks. The
existing regression tests cover telemetry and contextual state behavior, but
they do not establish the smaller screen's native visual readability.
