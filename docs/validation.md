# Validation matrix

Run commands from the repository root. Use `python3` on macOS/Linux where
`python` is not available, and `python` on Windows. Native builds require Python
3.10+, stable Rust 1.89+ with rustfmt/Clippy, locked Cargo dependencies, and the
[platform build prerequisites](../scripts/BUILDING.md#platform-prerequisites).
Use Python 3.12 for tooling checks. Initial installation/builds require network access.

## Automatic CI

[Source build](../.github/workflows/native-build.yml) runs one Ubuntu 24.04 job
on pull requests, main pushes and manual dispatch: `cargo build --workspace --locked`
in the debug profile. It uses Cargo caching, a 15-minute timeout and cancels
superseded runs for the same PR/branch. The same cached Ubuntu job also runs `cargo test --workspace --locked`, including
portable session E2E contracts (#347). Graphical suites remain periodic. It does
not stage executables, build release profiles or upload artifacts.

The separate [Periodic verification workflow](periodic-ci.md) runs Linux tests,
model processing and graphical/smoke checks nightly, with CPU benchmarks and
macOS/Windows build verification weekly. It supports manual runs, summaries,
artifacts and a history dashboard, with optional GitHub Pages deployment. It has
no PR/push trigger. All local verification commands below remain available.

The new check name is **Source build (Ubuntu)**. If branch protection or rulesets
require the old `build (ubuntu-24.04)`, `build (macos-14)` or `build (windows-2022)`
checks, a maintainer must replace those requirements with the new check before
merging under that policy. This workflow change does not edit protection settings.

## CPU and tooling gates

| Area | Exact command | Prerequisite/coverage |
| --- | --- | --- |
| Formatting | `cargo fmt --all -- --check` | rustfmt; use `cargo fmt --all` to apply |
| Rust lint | `cargo clippy --workspace --all-targets --locked -- -D warnings` | Clippy and native compilation dependencies |
| Rust tests | `cargo test --workspace --locked` | Domain, runtime composition/protocol, renderer CPU contracts; no live GPU |
| Workspace build | `cargo build --workspace --locked` | Checked-in exports, no Blender |
| Model requirements | `python -m pip install -r models/tools/requirements.txt` | Prefer a virtual environment; separate from standard-library build/runner tooling |
| Model contracts | `python -m unittest discover -s models/tests -v` | Model requirements; real Blender export test explicitly skips without Blender |
| Script/runner contracts | `python -m unittest discover -s scripts -p 'test_*.py'` | Standard library; Unix shebang CLI fixture explicitly skips on Windows |
| Native debug staging | `python scripts/build_game.py --profile debug --output artifacts/build/debug` | Builds for host and writes build metadata |
| Native release staging | `python scripts/build_game.py --profile release --output artifacts/build/release` | Optimized host executable; not a signed app/installer |

For focused iteration use `cargo test -p salimon-math --locked`,
`cargo test -p salimon-world --locked`,
`cargo test -p salimon-fabrication --locked`,
`cargo test -p salimon-character --locked`, `cargo test -p salimon-physics --locked`, `cargo test -p salimon-ship --locked`,
`cargo test -p salimon-renderer --locked`, `cargo test -p salimon-diagnostics --locked`
or `cargo test -p salimon-client --locked` for the affected owner; broaden to
workspace gates for cross-crate changes. The
[maintenance map](maintenance-map.md) identifies modules and scenario contracts.
For documentation/report moves, check relative links, command/source paths,
removed-path references and evidence integrity. No native launch is needed to
prove unchanged gameplay in a documentation-only change; record unrun gates.

## Asset-specific gates

```sh
python models/tools/validate_asset.py resource.iron-fragment
python models/assets/ships/salimon-scout/validate.py
```

The generic validator needs model requirements but no Blender. For scout source
or contract edits, regenerate using the category adapter, then verify source
against exported interchange and run model plus affected Rust consumer tests:

```sh
python models/assets/ships/salimon-scout/export.py --blender /path/to/blender
blender --background --python-exit-code 1 --python models/assets/ships/salimon-scout/verify_source.py
blender --background --python-exit-code 1 --python models/assets/ships/salimon-scout/verify_modular.py
python models/assets/ships/salimon-scout/validate.py
BLENDER=/path/to/blender python -m unittest discover -s models/tests -v
```

The last command uses POSIX environment assignment; set `BLENDER` in the shell's
environment on Windows. Blender 4.5 LTS with its bundled glTF exporter is the
supported authoring prerequisite. See [model tooling](../models/tools/README.md)
for generic asset export and [scout source](../models/assets/ships/salimon-scout/README.md)
for generated layout ownership. Generic CLI export does not register the ship
extension. Inspect source/export geometry and applicable native screenshots;
passing metadata checks alone does not prove visual quality.

## Optional offline GLB pipeline

For local verification (also covered nightly), install the pinned local Khronos validator, run
`python models/tools/process_glb.py all --profile exact-v1 --output artifacts/glb`,
and check candidates with the production CPU importers using
`SALIMON_GLB_CANDIDATES="$PWD/artifacts/glb" cargo test -p salimon-renderer --locked offline_glb_candidate -- --ignored`.
See [model pipeline setup, reports and disable switch](../models/tools/README.md#offline-glb-compliance-and-cost-pipeline-227).
These gates prove compliance, budgets, spatial invariants and exact importer data
parity; native visual fidelity/performance remains a separate verification.

## Native graphical gates

A running display, compatible GPU/backend and an unobscured capture surface are
required. After local release staging, manual Linux verification commands are:

```sh
export WGPU_BACKEND=vulkan
export VK_DRIVER_FILES=/path/to/lvp_icd.json
xvfb-run -a -s '-screen 0 1280x800x24' python scripts/salimon-test suite --binary artifacts/build/release/salimon-client
xvfb-run -a -s '-screen 0 1280x800x24' python scripts/salimon-test run scenarios/evidence/lower-cockpit-windows.json --binary artifacts/build/release/salimon-client --screenshot-command '["python", "scripts/capture_settled.py", "{path}"]'
xvfb-run -a -s '-screen 0 1280x800x24' python scripts/salimon-test suite --group resource-collection --evidence --binary artifacts/build/release/salimon-client --screenshot-command '["python", "scripts/capture_settled.py", "{path}"]'
xvfb-run -a -s '-screen 0 1280x800x24' python scripts/salimon-test suite --group ship-eva --evidence --binary artifacts/build/release/salimon-client --screenshot-command '["python", "scripts/capture_settled.py", "{path}"]'
xvfb-run -a -s '-screen 0 1920x1080x24' python scripts/packaged_smoke.py --binary artifacts/build/release/salimon-client --timeout 90
```

Use the actual installed lavapipe ICD filename from
`/usr/share/vulkan/icd.d/lvp_icd*.json`; do not copy the placeholder. Install
Xvfb, xauth, xdotool, ImageMagick and Mesa Vulkan plus the desktop libraries
from the [build prerequisites](../scripts/BUILDING.md#platform-prerequisites). Check `import -window root` capture
works on the test display. Screenshot/capture failures are failures, not skips.

On macOS/Windows use a local graphical desktop, omit Xvfb/Linux backend variables,
and select the staged executable (`salimon-client.exe` on Windows). Use the
[runner guide](../scripts/README.md) for platform capture configuration and
[packaged smoke prerequisites](../scripts/PACKAGED_SMOKE.md) for OS input/capture.
Nightly verification runs these Linux graphical checks. macOS/Windows graphics
remain local. Dispatch **Source build** for compilation and portable tests, or **Periodic
verification** for a nightly/weekly suite; neither adds a PR test gate.

Linux software Vulkan establishes launch, gameplay/state and capture coverage;
it does not establish macOS Metal/Windows hardware fidelity or performance.
Use the root [native smoke check](../README.md#native-smoke-check) for gameplay,
F2 precision, F3 telemetry, resize, minimize/restore, close and relaunch. Record
reference Apple M1 iMac 1920×1080 performance separately; no substitute machine
can establish reference-machine performance. Record rolling FPS, average/p95
presented-frame intervals and CPU/GPU timings with machine, scenario, drawable
size and revision. Rolling diagnostics cannot prove a per-frame 60 FPS floor;
the dated [Phase 0 evaluation](phase-0-evaluation.md) did not prove that strict
historical target. Native smoke checks do not certify performance.

## Reporting

Keep disposable staging/E2E output under ignored `artifacts/`. Preserve meaningful
results in the [per-task report](coding-conventions.md#task-completion-reports):
exact commands, passed/failed/skipped/not-run status, OS/tool versions and
limitations. Promote applicable screenshots plus structured state/logs and
link the report from the issue/PR. Never claim GPU, Blender or hardware coverage
from CPU tests alone.
When an unresolved validation point remains at delivery, transfer it to a linked
follow-up issue and close the source issue through the current PR under the
[completion workflow](coding-conventions.md#issue-completion-and-follow-up).
Retain its failed, skipped or unrun status; issue closure does not change results.

## Fragment profiling (#155)

Use [the benchmark guide](../scripts/FRAGMENT_BENCHMARKS.md) for opt-in 500/1,000
release matrices and reference-machine settings. CPU-only measurements and
Linux software Vulkan do not establish native FPS. The ordinary workspace tests
include large-load conservation/profiler parity/carrying/frame regressions; the
native default suite includes `scenarios/fragment-load.json`. Record actual
settling and sample sufficiency before applying performance gates.

## Integrated terrain (#179–#181)

Use the canonical [terrain validation guide](terrain-validation.md) for the
13-route native terrain group, reference window/precision smoke, direct GPU
capture, budgets/tolerances and reference measurements. Keep native Metal and
software Vulkan evidence separate. Direct GPU capture verifies actual rendered
pixels; it does not certify OS input or a visible display FPS floor.

Tutorial #380: `cargo test -p salimon-client session::tutorial --locked` checks
early actions, exact item use, skip/reload, legacy migration, restart and bounded
prompt raster. Run workspace tests/build/Clippy/fmt for schema changes. Native
DPI, prompt priority and full interaction walkthrough remain #376.

Balance #384: `cargo test -p salimon-client survival_balance --locked`; the six
scenarios also run in workspace PR CI. See [JSON reproduction and limits](survival-balance.md).

Engine #493: `cargo test -p salimon-ship --locked` plus workspace tests/build/
Clippy/fmt. `inertial-flight.json` and `landing-speed-cutoff.json` now exercise
1000/500 m/s² dry thrust, coast/braking and the unchanged 20m/s landing limit.
Physical B input, handling and instrument appearance need #498 reference evidence.

## Portable persistence and survival (#375)

`cargo test --workspace --locked` includes the bounded whole-session disk/launch,
checkpoint worker failure, death/restart and survival scenario contracts. For
focused iteration: `cargo test -p salimon-client --locked persistence_scenario`.
The [portable DSL and operation reference](../scenarios/portable/README.md) adds
checkpoint/reload/continuation actions to existing production protocol requests.
All paths use isolated temporary storage, never the player's active save.

Run `cargo test -p salimon-client --locked persistence_extended -- --ignored` for
the 500-fragment whole-session continuation and full eighteen-hour 16/33/100 ms
survival partition matrix. Nightly verification runs this exact filter in its
Rust step and retains the raw output with periodic artifacts. Do not replace it
with an unrestricted `--ignored`, which also selects unrelated opt-in GPU tests.
See the [coverage/results report](../reports/issue-375/README.md); reference M1,
OS input and physical route acceptance remain #348/#376/#496/#498 as applicable.
