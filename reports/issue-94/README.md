# Issue #94 — Stateful first-person reticle

Implemented a screen-space `+` for the equipped mining tool and a small dot when
stowed. Selection reads `MiningTool.equipped` every gameplay redraw; no duplicated
UI equip state. Precision tour hides the reticle. Removed the old world-space aim
cuboid while preserving tool geometry and the eye/look-target ray used by mining
and interactions.

Runtime supplies two static 17×17 RGBA images with white strokes and dark outlines.
Renderer reuses its generic cached image pipeline in an independent overlay slot,
composites after the scene without depth occlusion, and includes the draw in total
draw-call measurements. Center placement derives from current drawable dimensions,
so resize/reconstruction keep the midpoint at NDC (0, 0). Pixels upload only when
equip state changes. Updated runtime/renderer architecture and invariants.

## Validation

Environment: Ubuntu 24.04 x86_64, Rust/Cargo 1.99.0, software Vulkan (Mesa
lavapipe), Xvfb 1280×800. No asset regeneration or new dependencies required.

| Check | Result |
| --- | --- |
| `cargo fmt --all -- --check` | Passed |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | Passed |
| `cargo test --workspace --locked` | Passed: 240 tests |
| `cargo build --workspace --locked` | Passed |
| `git diff --check` | Passed |
| Native `scenarios/evidence/mining.json` with debug binary | Passed: 141 steps, five captured checkpoints |

Focused regressions cover equip/stow selection and revision changes, hidden tour
presentation, dot/cross symmetry and shared center, removal of the 3D marker at
multiple pitches, and projection-center alignment across landscape, portrait,
odd-sized and small drawable dimensions.

Native command (run under an isolated Xvfb display with software Vulkan):

```sh
WGPU_BACKEND=vulkan VK_DRIVER_FILES=/path/to/lvp_icd.json \
python3 scripts/salimon-test run scenarios/evidence/mining.json \
  --binary target/debug/salimon-client --artifacts artifacts/issue-94/mining \
  --screenshot-command '["python3", "scripts/capture_settled.py", "{path}"]'
```

The environment lacked native test packages. Dependencies were downloaded and
extracted into `/tmp/salimon-94-native` rather than installed into the system.
Separate shell invocations could not connect to the test X server; running Xvfb
and the scenario in the same shell with TCP display `127.0.0.1:96` succeeded.
Earlier failed launches reached no gameplay steps; the final run passed.

[Native result summary](mining-result.json). Screenshots reviewed for both shapes
and their placement on the targeted deposit:

- [Empty-handed dot](dot.png)
- [Equipped cross](cross.png)
- [Mining still updates deposit state and mass](mining.png)

## Limits

Native resize/fullscreen interaction was not exercised in this run; the placement
regression checks the same size/position calculation uploaded to the overlay
shader. macOS/Windows graphical fidelity and reference-hardware performance were
not tested. Only the mining tool currently has an equip state; future equipment
must map its authoritative state into the same presentation selection.
