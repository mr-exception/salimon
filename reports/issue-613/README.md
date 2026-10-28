# Issue #613 — geological surface materials

Parent: non-draft [PR #686](https://github.com/mr-exception/salimon/pull/686),
`46af70800a7254bcd242a888583d0c84f088f318` (#612). This child targets
`issue-612-canonical-geology`; merge the parent first. Nothing is merged here.

Runtime now maps canonical geological exposure, weathering and layers into neutral
terrain luminance and filtered stratification. The existing terrain sample owns
all positions/normals; material observations never move geometry or resources.
Cache identity binds the full geology/terrain recipe. GPU encoding, vertex layout,
finite/range validation and byte budgets change together from 36 to 52 bytes per
vertex. No new draw/pipeline or normal/depth/displacement behavior is added.

Ice has an explicit generic material using the approved pale U11 hue; original
six body swatches, shared detail slot and all resource assets remain unchanged.
Vesper retains unit luminance/zero strata strength and its original Slate ground,
immutable geography, resource identities and save schema. Current native bodies
exercise rock profiles; the explicit ice capability uses declared synthetic GPU
fixtures, not a claim of a newly published icy world. No optional micro-cover or
new physical seed coupling is enabled.

The [canonical guide](../../docs/geological-materials.md) documents source/test
mapping, parameters, identity, layout, limits and exact validation routes. Runtime
and renderer owning guides, root routing, sphere texture allocation, terrain and
approved-direction status are updated in this PR.

## Validation environment and commands

Cloud Debian 13 x86_64; Rust/Cargo 1.99.1, GCC 14; Python tooling 3.12.14.
Native tests run in isolated Debian trixie with Python 3.13.5, Xvfb, Mesa 25.0.7,
LLVM 19.1.7 llvmpipe Vulkan, `WGPU_BACKEND=vulkan`,
`VK_DRIVER_FILES=/usr/share/vulkan/icd.d/lvp_icd.json`, `XDG_RUNTIME_DIR=/tmp`.
`CARGO_BUILD_JOBS=4`, `CARGO_PROFILE_DEV_DEBUG=0`, `CARGO_PROFILE_TEST_DEBUG=0`.
Assertions remain enabled. Actual binary/source/image hashes are retained below.

- `cargo test --workspace --locked`: **1,143 passed, 0 failed, 39 opt-in ignored**.
  The 39 include the two explicit renderer evidence checks executed separately below.
- After the final engineering-camera clearance test and test-only Clippy cleanup,
  `cargo test --workspace --locked geolog`: **9 passed**, and
  `cargo test --workspace --locked terrain_materials`: **2 passed**. These overlap
  the workspace cases and additionally verify the final sampled-ground cameras.
- `cargo build --workspace --locked`: passed.
- `cargo clippy --workspace --all-targets --locked -- -D warnings`: passed after
  replacing a test's chunk iterator with the idiomatic fixed-array equivalent.
- `cargo fmt --all -- --check`: passed.
- `python3 scripts/check_docs.py --base 46af70800a7254bcd242a888583d0c84f088f318`:
  zero failures. Python tooling: **134 passed**; all four new scenario JSON files
  also pass the production parser.
- `SALIMON_TERRAIN_STYLE_DIR=<absolute-output-directory> cargo test --workspace --locked terrain_style_reference -- --ignored --nocapture`:
  passed; **21** neutral production-pipeline image cases.
- Same command with `terrain_geology_reference`: passed; **28** rock/ice day/night
  cases at near, mid-transition, far and orbital distances. Each case asserts
  frozen repeated pixels and transition continuity; costs CSV records actual
  draw/triangle/buffer/timing context. These are synthetic shader patches.
- Native command: `xvfb-run -a -s '-screen 0 1280x800x24' python3 scripts/salimon-test run scenarios/evidence/geology-<body>.json --binary target/debug/salimon-client --screenshot-command '["python3","scripts/capture_settled.py","{path}"]' --summary`.
  Final route results and original images are recorded below.

## Observations and retained failures

The neutral 21-case comparison has identical coverage. Nine pixels differ by at
most one 8-bit color level, consistent with GPU interpolation/rounding after the
additional varyings; no silhouette difference or palette replacement is claimed.
The six rock/ice near/far/orbit day/night pairs have identical coverage and lower
night brightness. Layer filtering retains the existing LOD fade continuity.

An optional direct-native-readback probe on the **parent** binary timed out at
step 10 after nine successful steps during loaded verification. Its failed result
is retained rather than called a pass. The supported settled-desktop path passed
on the parent and current production pipeline. Initial native captures used the
1920x1080 default drawable with a 1280x800 desktop; those are cropped evidence,
not a full-frame proof. The first Earth run also exceeded its preparation wait on
software rendering at that size. Final scenarios explicitly resize and wait for
1280x800 before capture; original failed/intermediate results remain retained.
No production capture deadline or functional assertion was weakened.

The first resized Moon run hit the existing five-second inspection deadline
at step 9 (8 steps complete). An isolated rerun with the identical scenario,
assertions and deadlines passed all 17 steps. Both outcomes are retained in
`probes/initial-moon-inspection-timeout/` and `native-moon/` respectively; no
transport deadline was extended and no assertion was removed.

Final live production routes passed **73/73 steps**: Vesper 22, Earth 17,
Moon 17 and Mars 17. The matching parent Vesper control passed 22/22 steps.
Original final results are retained in [Vesper](native-vesper/result.json.xz),
[Earth](native-earth/result.json.xz), [Moon](native-moon/result.json.xz),
[Mars](native-mars/result.json.xz) and [parent control](native-parent-vesper/result.json.xz).

Representative originals: [Vesper surface](native-vesper/step-012-universe-surface.png),
[Earth near](native-earth/step-007-geology-earth-near.png),
[Moon mid](native-moon/step-012-geology-moon-mid.png),
[Mars far](native-mars/step-017-geology-mars-far.png),
[rock day](gpu-geology/rock-day-near.png), [rock night](gpu-geology/rock-night-near.png)
and [ice day](gpu-geology/ice-day-near.png). Every distance/lighting original
remains in the corresponding folder, not just these examples.

The compact [native summary](native-results.json) lists final pass/fail and durations.
Reproduce image/coverage, native outcome and real buffer-bound assertions with
`python3 reports/issue-613/compare_evidence.py` (Pillow 12.3.0). The generated
[comparison](comparison.json) includes per-image deltas and actual native maxima.
[Capture context](capture-context.json), [source hashes](source-files.sha256.json)
and `evidence-sha256.txt` bind source/binary and stored evidence. Full original
stdout/protocol streams and detailed result JSON are losslessly compressed as `.xz`; decompressed-byte
hashes are in [uncompressed trace hashes](uncompressed-trace-hashes.json).
Per-checkpoint states are retained in each original `result.json.xz`; redundant
per-step JSON copies are omitted. No failed probe is relabeled a pass.

## Limits and ownership

Software-Vulkan timings and observed preparation overruns describe this workload
and executor, not M1/Metal or hardware FPS. Existing cooperative admission may
finish an admitted patch beyond its 2 ms budget; original observations are retained.
The 32 MiB GPU cache and 2 MiB/frame upload bounds remain enforced on real bytes.
Subjective appearance/readability and baseline approval remain
[#561](https://github.com/mr-exception/salimon/issues/561); hardware/Metal budgets
remain [#563](https://github.com/mr-exception/salimon/issues/563), and physical OS
input/display behavior remains #557. None is reported passed or required for this
routine cloud delivery. No baseline is promoted. Climate/weather, active geology,
biology, liquid surfaces and terrain/resource migration remain separate scopes.

## Observed native resource limits

| Route | Steps | GPU cache MiB | Peak upload bytes | Peak preparation ms |
| --- | ---: | ---: | ---: | ---: |
| native-parent-vesper | 22 | 5.17 | 2096916 | 23.966 |
| native-vesper | 22 | 6.58 | 2094368 | 22.377 |
| native-earth | 17 | 6.72 | 2096916 | 115.092 |
| native-moon | 17 | 6.62 | 2096496 | 253.388 |
| native-mars | 17 | 6.50 | 2096496 | 538.373 |

These are observed software-renderer peaks, including warmup/selection, not
steady-state hardware budgets or an M1/Metal performance claim.
