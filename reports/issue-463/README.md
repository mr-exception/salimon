# #463 — Scout light zoning and reference foundation

Delivered on 2026-10-21 against main `db5e37eb18c363f4dcbd7acf23e55927327d277d`.

## Changes

- Audited ship GLB/shader vs Blender: base/emission colors, diffuse vertex fill,
  screen atlas and blended glass work; ordinary PBR/roughness/textures/light
  transport do not. Retained geometry and materials from the existing Scout.
- Warm cockpit/workshop zones and cooler exterior; real bus blackout suppresses
  warm fill/emission while preserving a cool non-electrical visibility floor.
  Existing screen blank/restoration consumes the same explicit power boolean.
- Four shared camera poses, native E2E-only presentation override, eight real
  blackout/powered checkpoints, three repeats each; read-only Blender preview.
- Structural delta: zero vertices/draws/textures/GPU buffers/pipelines; unchanged
  160-byte uniform uses one spare scalar. Extra vertex arithmetic is bounded.
  This establishes allocation/draw cost only, not a measured frame budget.

## Validation

Environment: Ubuntu 24.04 x86_64, Rust/cargo 1.99.1, Python 3.12.14.

- `cargo test --workspace --locked`: passed (684 tests, 22 opt-in ignored).
- `cargo build --workspace --locked`: passed.
- `cargo clippy --workspace --all-targets --locked -- -D warnings`: passed.
- `cargo fmt --all -- --check`: passed.
- `python -m unittest discover -s scripts -p 'test_*.py'`: 95 tests passed.
- `python -m py_compile models/assets/ships/salimon-scout/preview_lighting.py`: passed.
- Shared camera pose/translation, paired capture coverage and naga shader
  validation are included in those portable tests. No claim of image quality.
- Attempted native capture with `WGPU_BACKEND=vulkan`, Mesa lavapipe and
  `xvfb-run -a python scripts/model_visual_tests.py capture --binary
  target/debug/salimon-client --fixtures scenarios/visual/scout-fixtures.json
  --captures artifacts/scout-lighting/candidates`: failed before ready signal,
  `Failed to open connection to X server`. Xvfb cannot establish listening
  sockets here. Earlier runner allowlist failure was corrected; no image was
  captured. No comparison or GPU/frame timing pass is claimed.
- Blender render: not run, supported Blender not installed.

## Remaining validation

[#483](https://github.com/mr-exception/salimon/issues/483) owns native/offline image
capture/review, projection inspection and matched presentation timing. Existing
#451/#477 evidence does not approve this new shader. Source closure records
implementation plus explicit transfer; appearance/performance remain unverified.
See [canonical contract and exact commands](../../docs/scout-presentation.md).
