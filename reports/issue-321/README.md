# Issue #321 — software-Vulkan Option A candidate corpus

2026-10-19. **Owner approval recorded; approved software-Vulkan comparison passed.**
The follow-up PR completes #321 on merge. Metal appearance/native M1 FPS remain
independent in #314.

## Approval and final validation

Repository owner `mr-exception` merged [PR #335](https://github.com/mr-exception/salimon/pull/335)
at 2026-10-19T09:40:25Z and explicitly confirmed approval in ChatGPT at
2026-10-19T12:41:40+03:00, following the explanation that approval covers images,
masks and tolerances. This owner confirmation is the approval evidence; no
GitHub review submission is claimed. The corpus now records the actual reviewer,
date, evidence and `masks_and_tolerances_reviewed: true`. Image bytes, source and
binary provenance, fixture actions/seeds, empty masks and thresholds are unchanged.

- **Passed:** `python scripts/model_visual_tests.py compare --reviewed --baselines scenarios/visual/baselines/software-vulkan-option-a-v1 --output artifacts/model-visual/approved-diffs`
  — exit 0, all 128 baseline comparisons and all temporal checks passed.
  [Coverage](appearance.json) / [per-fixture metrics and summaries](approved-comparison/).
  This compares the retained production captures from the original clean source
  revision against the exact owner-approved corpus; it is not a new game launch.
- **Passed detection test:** `python reports/issue-321/reproduce-injection.py`
  — the approved full CLI rejects the injected Core patch with exit 1 and
  [failed appearance coverage](reviewed-injection-appearance.json).
  [Core metrics](reviewed-injection-core.json) show all three pixel comparisons
  fail while both temporal comparisons pass. Injected image hashes were refreshed
  so this tests pixel failure, not rejection of stale hashes. Test images are
  disposable; the approved corpus is never modified.
- **Passed:** `python -m unittest discover -s scripts -p 'test_*.py'`
  — 93 tests; [approval validation log](approval-python-tests.log).
- **Passed:** `git diff --check`, corpus image/fixture hash integrity, and local
  report-link validation.
- **Not run:** a new graphical capture/build (runtime and fixture inputs are
  unchanged), a live modified shader/asset, Metal captures or native M1 FPS.

The earlier pending result is retained in [appearance-before-approval.json](appearance-before-approval.json).
The following sections preserve the initial candidate delivery and its checks;
references to pending review there describe that initial state, before approval.

## Delivered

- All **128 production PNGs / 46 checkpoints / nine fixtures**, captured through
  the maintained `model_visual_tests.py capture` command from clean tracked
  revision `6eacbbac2cd341a990b3d50e0482cd7997dac702`.
- Candidate images and provenance in
  [software-vulkan-option-a-v1](../../scenarios/visual/baselines/software-vulkan-option-a-v1/corpus.json).
  At initial delivery the metadata was `status: candidate`; approval fields
  were added only after the owner confirmation recorded above.
- [Checkpoint review index](checkpoints.md) links **every frame**, including all
  repeats. Review silhouette, materials, glazing, console/Core, resources/tool/
  spheres, contact floor, ramp and door. All rows initially awaited human review; the index now records approval.
- [Capture evidence](capture-evidence.json) retains passed fixture setup, step
  counts, screenshot statuses and observed context for every image.
- [Temporal summaries](temporal/) and [appearance coverage](appearance.json)
  preserve passed repeat checks and pending human approval separately.
- [Injected-change evidence](injection.json) and
  [reproduction script](reproduce-injection.py) demonstrate a persistent change
  cannot pass pixel comparison merely by remaining temporally stable.
  [Difference](injected-core-difference.png) / [highlight](injected-core-highlight.png).

## Capture blocker repaired

The [2026-10-19 nightly run](https://github.com/mr-exception/salimon/actions/runs/39203287820)
failed with `unexpected image outside fixture contract`. Capture previously
stored the runner's raw diagnostic PNGs under each candidate fixture; the strict
recursive corpus inventory correctly rejected those extra images. Move runner
diagnostics into the sibling `candidates-runs/<fixture>/` directory. Keep the
strict inventory, approval protections, fixture actions/seeds and image thresholds
unchanged. A regression test exercises capture through a mocked native runner,
then builds a corpus and verifies genuine stray PNGs still fail.

## Actual environment and provenance

| Property | Observed value |
| --- | --- |
| OS | Ubuntu 24.04.3 LTS, x86_64, glibc 2.39 |
| Kernel/platform | Linux 6.18.55; `Linux-6.18.55-x86_64-with-glibc2.39` |
| Backend | Vulkan |
| Adapter | llvmpipe (LLVM 20.1.2, 256 bits), Mesa software Vulkan/lavapipe |
| Driver | llvmpipe / Mesa 25.2.8-0ubuntu0.24.04.2 (LLVM 20.1.2) |
| Display | Xvfb, 1280×800×24; physical client viewport 1280×800; DPI scale 1.0 |
| Software threads | `LP_NUM_THREADS=2` |
| Rust | rustc 1.99.1 (58cf8f9c3 2026-10-13), release, locked dependencies |
| Python / Pillow | 3.12.14 / 12.3.0 |
| Binary SHA-256 | `cb72f0d0d017785d2adf59bf39aa0bbce433ec661162de2ff409744cbc556147` |
| Fixture/seed/action/mask/tolerance SHA-256 | `c9ba706b23b0a150b96ec8d33837b0ba98bc12eb6ca392a778259d6472b7fa8d` |

Mesa/X11 packages were downloaded from Ubuntu's 20260828 snapshot and extracted
under `/tmp/salimon-deps` without a system package installation. The package
versions are therefore pinned by that snapshot. `LD_LIBRARY_PATH`, `LIBRARY_PATH`
and `PKG_CONFIG_PATH` selected its `usr/lib/x86_64-linux-gnu` libraries/pkgconfig;
`PATH` included its `usr/bin`. Xvfb required an `/usr/bin/xkbcomp` symlink to that
extracted binary. This session restricts Unix sockets, so Xvfb listened on local
TCP (`DISPLAY=127.0.0.1:93`) with Unix/local transports disabled. These captures
use the game's direct GPU capture, not desktop image substitution.

The first build attempt in repository-local `target/` failed creating a compiler
archive (zero-length mmap); a fresh isolated `/tmp/salimon-target` build passed.
Initial display startup attempts failed before game readiness (Unix sockets and
a stale display lock); a fresh TCP display captured all fixtures successfully.
Those failed setup attempts were excluded from the candidate corpus. No failed
checkpoint was substituted, masked or approved.

## Commands and results

After installing/extracting the documented [Linux prerequisites](../../scripts/BUILDING.md#platform-prerequisites):

```sh
export PATH=/root/.cargo/bin:/tmp/salimon-deps/usr/bin:$PATH
export LD_LIBRARY_PATH=/tmp/salimon-deps/usr/lib/x86_64-linux-gnu
export LIBRARY_PATH=/tmp/salimon-deps/usr/lib/x86_64-linux-gnu
export PKG_CONFIG_PATH=/tmp/salimon-deps/usr/lib/x86_64-linux-gnu/pkgconfig
export CARGO_TARGET_DIR=/tmp/salimon-target CARGO_BUILD_JOBS=2
cargo build --release --locked -p salimon-client
export WGPU_BACKEND=vulkan LP_NUM_THREADS=2
export VK_DRIVER_FILES=/tmp/salimon-deps/usr/share/vulkan/icd.d/lvp_icd.json
export XDG_RUNTIME_DIR=/tmp/salimon-xdg DISPLAY=127.0.0.1:93
mkdir -p "$XDG_RUNTIME_DIR"
chmod 700 "$XDG_RUNTIME_DIR"
Xvfb :93 -screen 0 1280x800x24 -listen tcp -nolisten unix -nolisten local -ac &
# Wait for xdotool getdisplaygeometry to succeed before capturing.
python scripts/model_visual_tests.py capture --binary /tmp/salimon-target/release/salimon-client
python scripts/model_visual_tests.py compare --temporal-only
python scripts/model_visual_tests.py compare --reviewed --baselines scenarios/visual/baselines/software-vulkan-option-a-v1
python reports/issue-321/reproduce-injection.py
```

Capture requires the clean source revision above; the last reproduction command
requires fresh captures at the default path. Choose a free display number.

| Gate | Result |
| --- | --- |
| Release client build | **Passed**, [build log](build.log) |
| Production capture | **Passed**, exact 128-image inventory, observed context, source/binary/contract/image hashes |
| Temporal comparison | **Passed**, all nine fixtures; no masks and no tolerance changes |
| Reviewed comparison before copying candidate | **Pending**, exit 3 against the original pending metadata |
| Reviewed comparison against submitted candidate | **Pending**, exit 3; [appearance.json](appearance.json) |
| Persistent post-capture 120×120 Core patch | **Passed detection test**: appearance failed for all three repeats (14,400/1,024,000 pixels = 1.40625%); temporal repeats passed with zero changed pixels |
| `python -m unittest discover -s scripts -p 'test_*.py'` | **Passed**, 93 tests; [log](python-tests.log). Printed simulated timeout/failure belongs to a passing failure-path test |
| `cargo fmt --all -- --check` | **Passed** |
| `git diff --check` | **Passed** |
| Human image/mask/tolerance review | **Pending** |
| Approved `compare --reviewed` / live changed-binary regression | **Not run**; no approved production corpus yet. Injection above modifies production PNGs, not a game shader/asset |
| Metal / native M1 performance | **Not run**, independent #314 scope |

## Reviewer action and completion

Review every image/repeat in [checkpoints.md](checkpoints.md) and explicitly review
**channel tolerance 8/255, changed fraction 0.001 (0.1%), masks `[]`** for all nine
fixtures. No setting was broadened. Visual inspection by the authoring assistant
is not human approval.

Only after explicit human approval should the metadata become `approved`, with
actual reviewer, review date, evidence link and
`masks_and_tolerances_reviewed: true`. Then run and retain a successful approved
`compare --reviewed` against this exact corpus and finish #321. A merge or a
passing temporal check alone does not satisfy that approval contract. Until
then, periodic appearance coverage must remain pending. The original draft kept #321 open without transferring or waiving requirements.
The approval and passed reviewed comparison recorded above now satisfy those gates.
