# Issue #436 — Canonical rendering and streaming cloud acceptance

The `universe-render` suite requires the production canonical orbit/surface/
face-edge/revisit, new-journey/boarding and mining/carry/reload routes. Native
GPU image review checks settled canonical geometry, the existing patch/cache/
upload bounds, four exact frozen RGB repeats, exact orbit revisit and projected
planet diameter against the declared camera's physical radius/distance ratio.
The seam regression now pairs actual canonical stitched triangle boundaries at
face edges and corners, in addition to the existing full-sphere topology,
cache-eviction identity, precision and physical-query regressions.

Runtime diagnostics retain at most 3,600 raw frame rows with explicit units and
scene/total draws. Unavailable GPU samples remain null. A physical-clearance audit also found the old face-edge eye 675.070 m below
terrain: its 707.107 m nominal radial altitude was below 1,382.177 m relief.
The eye now samples that authoritative height and stays 500 m above ground.
Native reference-camera observations expose actual clearance, and image review
rejects nonpositive clearance or a different view. This fixes the evidence
camera, with no ordinary gameplay, world, save, shader, source-selection or
resource-bound policy change. Source and
verification mapping are in the [maintained contract](../../docs/universe-terrain.md#cloud-integration-acceptance-436).
Runtime architecture/invariants and usage/validation/maintenance guides are
updated together. Renderer/Universe ownership and their existing contracts are
unchanged; their production tests are reused without changing their algorithms.

Parent: `main`, exact commit `bfa5a17e0b5cf680bcb4c3ae690a4a8c46c49642`.
No active PR existed at selection. Prerequisites #423, #424, #425 and #427 were
closed and their implementations were present on this parent. No merge is
performed by this delivery.

## Environment and reproduction

Cloud Debian x86_64 / Linux 6.18.57, Rust 1.99.1, Python 3.12.14 / Pillow,
Xvfb 1280×800 (scale 1), Mesa 25.0.7
llvmpipe Vulkan (LLVM 19.1.7). Two software-renderer threads are used. ALSA
headers/runtime were missing from the preconfigured local sysroot; downloaded
Debian `libasound2-dev` and `libasound2t64` before rebuilding. The first build
and test attempts failed at ALSA discovery/linking; these are setup failures,
not successful source checks.

```sh
cargo build --workspace --locked
cargo test --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo fmt --all -- --check
python3 -m unittest discover -s scripts -p 'test_*.py'
python3 scripts/check_docs.py --base origin/main
python3 scripts/salimon-test suite --group universe-render --binary target/debug/salimon-client --artifacts artifacts/issue-436 --screenshot-command '["native"]' --summary
python3 scripts/review_universe_captures.py artifacts/issue-436/RUN/result.json --output artifacts/issue-436/image-review.json
SALIMON_SKY_DIR="$PWD/artifacts/issue-436-sky" cargo test -p salimon-renderer --locked sky_production_reference -- --ignored --nocapture
```

Use the repository's Linux Xvfb/Mesa environment from the validation guide;
replace `RUN` with the terrain result directory. Exact source, compiler,
commands, logs, original captures and results accompany this report.

## Results

- Corrected-source workspace tests: **1,244 passed, zero failed, 51 existing opt-in tests ignored**. This includes the canonical camera clearance, actual stitched face-edge/corner, cache eviction/revisit, stale target, source projection, precision, physical integration and malformed-input contracts.
- Workspace build, all-target Clippy with warnings denied, fmt and diff checks: passed.
- Python tooling: **138 passed**. Negative capture-review cases include changed pixels, wrong provenance/dimensions/context, missing/duplicate views, underground/wrong camera, failed runs, missing raw data, excessive resource use, blank/enlarged/shifted planet disks and absent originals.
- Parent-relative documentation: **zero failures**. Architecture audit executed; its 56 warning-only existing candidates remain advisory, not a claim of resolved architecture or active-PR overlap verification.
- Standalone production sky: **one opt-in test passed**, real llvmpipe Vulkan, exact origin/trillion-metre translation RGB equality, visible resolved star, complete reverse-Z occlusion and malformed/count rejection. Captures include synchronous readback; timings are diagnostic.

The final native suite and image-review results are recorded with the evidence
below. The source manifest binds changed runtime/tooling/scenario files and the
actual binary; reports are written afterward and are not claimed to have been
compiled into that binary.

## Final native evidence

**101/101 steps passed** in the corrected-source serial production suite on
Xvfb :98: terrain 52/52 (50.621 s), journey/boarding 25/25 (36.550 s),
mining/carry/reload 24/24 (34.163 s). Input uses the production protocol/session
adapter, not OS-injected or physical keys. Original GPU images are 1280×800 / 1x.

Independent image review passed: four exact frozen RGB repeats and one exact
orbit revisit, compatible renderer contexts, expected camera names, positive
physical terrain clearance, nonempty settled canonical geometry and unchanged
patch/cache/upload bounds. The orbit disk is **498 px**, versus **498.636 px**
from the declared 2.5-radius distance and 70° vertical FOV. Four pixels of
allowance cover raster boundary rounding and <=1,800 m relief on this
2,985,359 m nominal-radius body, not a promoted aesthetic baseline.

| View | Ground clearance | Active / cached patches | CPU / GPU terrain bytes | Latest scene / total draws |
| --- | --- | --- | --- | --- |
| Orbit | 4,478,038.5 m | 6 / 6 | 176,678 / 258,104 | 11 / 14 |
| Surface | 2.8034 m | 300 / 306 | 9,041,870 / 6,904,548 | 310 / 313 |
| Face edge | 500 m | 138 / 438 | 12,829,774 / 3,244,992 | 143 / 146 |
| Orbit revisit | 4,478,038.5 m | 6 / 438 | 12,783,158 / 258,104 | 11 / 14 |

The cooperative 2 ms terrain preparation admission may overrun on its final
unit; the actual peak is **7,046 µs**, with **502 cumulative overruns** by the
last view. They are retained, not relabeled as meeting a hard 2 ms ceiling.
Generation stayed within eight/frame and peak uploads within 2 MiB. The raw
per-frame CPU and software-GPU samples, draw rows, allocator readings, cumulative
aggregates and per-view observations remain in the original results and review.
No timing threshold was relaxed and no native hardware FPS claim follows.

- [Suite summary](native-suite-summary.json), [image review](image-review.json.gz), [original PNG hashes](capture-manifest.json), [source/binary hashes](source-manifest.json), [environment](environment.json).
- [Orbit](terrain/step-012-universe-orbit.png), [surface](terrain/step-025-universe-surface.png), [corrected face edge](terrain/step-038-universe-edge.png), [orbit revisit](terrain/step-051-universe-orbit-revisit.png).
- [Canonical start](journey/step-004-published-start.png), [entered ship](journey/step-024-entered-ship.png), [carrying mined material](mining/step-017-canonical-carry.png), [reload](mining/step-024-canonical-reloaded.png).
- [Sky](sky/sky.png), [translated sky](sky/translated.png), [occluded sky](sky/occluded.png).
- Original scenario/result/protocol/stdout/stderr are compressed beside each route's images. Their absolute artifact paths record the original run; use the manifest's relative paths for retained originals or rerun the documented commands.
- [Workspace tests](logs/workspace-tests.log.gz), [build](logs/build.log.gz), [Clippy](logs/clippy.log.gz), [Python](logs/python.log.gz), [docs](logs/docs.log.gz), [seams](logs/seams.log.gz), [sky](logs/sky.log.gz), [camera regression](logs/camera-regression.log.gz).
- [Contended failure](failed-contended-summary.json), [failed original result](failed-contended/result.json.gz), [display setup failures](failed-display-summary.json), [superseded suite](superseded-camera-summary.json), [superseded underground edge](superseded-camera/underground-edge.png).

## Preserved failures and qualification limits

The first graphics attempt timed out at the surface screenshot while builds
were running on the same two-CPU quota. It reached 300 active patches and
settled geometry within the existing cache/upload limits; the subsequent native
capture-status request exceeded the protocol's five-second deadline. This is
retained as failed evidence, not accepted as a capture or classified as flaky.
The isolated pre-camera-correction suite passed 97/97 steps and captured the
same surface successfully. Its pixel reviewer also passed, but those checks
alone missed the underground face-edge camera. The physical query above found
that problem; final acceptance additionally requires positive ground clearance
and uses the corrected camera. The pre-correction original and result remain
explicitly superseded, not accepted as an above-ground face-edge view. A separate attempt
failed all three launches before any step because the old Xvfb process had
exited and left a stale X99 socket. A dedicated foreground Xvfb :98 session
fixed this setup failure; its failed summary is retained as well.

The camera route uses explicit fixed reference cameras and overhead fixture
illumination. It is a rendered orbit-to-surface probe, not a physically flown
orbital transfer or certification of evolving ephemeris illumination. The
Milky Way is authored; the current host direction uses the documented fixed
starting-system axes, and optional pack sky sources are bounded. Complete
real-star coverage/source accuracy remains #524/#526. Closed-game propagation
is #432–#434. Existing projected sphere/depth/split-origin tests and real shader
captures establish their stated contexts, not implementation of future models.

Presented intervals inherit the existing FrameClock 250 ms cap and reset-to-zero
semantics. Raw CPU update/render and returned GPU durations retain their values;
this is not an uncapped wall-clock frame-pacing trace. Aggregates are cumulative
within each process, not isolated per-view benchmark lanes. Scene/total draw
counts are instantaneous rows; preparation peaks/overruns and terrain upload/cache
observations are retained as reported. Software-renderer timings and preparation
overruns are diagnostics, not M1 FPS
or hardware-budget approval. Subjective visual/readability review remains
[#561](https://github.com/mr-exception/salimon/issues/561); actual physical
platform input/display/OS behavior remains [#557](https://github.com/mr-exception/salimon/issues/557);
real GPU/M1/Metal budgets and frame pacing remain [#563](https://github.com/mr-exception/salimon/issues/563).
These are separately **unverified**. No baseline, aesthetic, source-rights or
public-release approval is inferred. This source issue remains open until PR
merge.
