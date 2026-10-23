# Validation matrix

## Verification ownership

Follow [cloud and human verification](verification-workflow.md): reproducible
Linux cloud checks are the default completion path (`auto-verification`).
Subjective reviews, physical OS/device experiments and explicit owner decisions
belong to instructed `human-verification` issues; return evidence to Codex.
References below to older native/M1 issue gates follow the current issue labels
and transferred ownership, not a blanket desktop prerequisite. Preserve actual
failures and context: software Vulkan cannot certify Metal or hardware FPS.
The existing native graphical commands below run the actual application on Linux;
no game-engine or browser migration is implied. Existing numerical budgets remain
in force for their stated workload/hardware.

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
The same runtime flight regression module replays the landing-cutoff and cockpit
monitor evidence routes without GPU capture. Run `cargo test -p salimon-client
--locked flight_regression` for both baseline and screenshot-route gameplay;
nightly still owns screenshot, temporal and reviewed appearance checks. Software
corpus v2 approval is pending #518; see [periodic CI](periodic-ci.md#current-corpus-transition-2026-10-22).

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


## Scout UI (#466)

Workspace gates cover shared glyphs/SI case, opaque palette contrast, bounded
instrument lines, physical viewer-right/top-left screen orientation, existing
prompt/DPI/modal/reticle/input contracts and Fabricator transactions. Opt-in CPU
pixel evidence: set `SALIMON_SCOUT_UI_DIR` to an absolute directory and run
`cargo test -p salimon-renderer --locked scout_ui_raster_evidence -- --ignored`
and `cargo test -p salimon-client --locked scout_prompt_raster_evidence -- --ignored`.
These pixels are not native screenshots. M1 view/DPI/OS-input/frame-tail review
remains #502; scanner integration #503 waits for #430. See the
[UI report](../reports/issue-466/README.md) and
[canonical matrix](scout-presentation.md#scout-ui-readability-contract-466).


## Scout Archive (#467)

`cargo test -p salimon-client --locked archive` covers physical ray/access, read
selection, whole-session restore/legacy migration, invalid source transaction,
modal gates and bounded spoiler-free text. Workspace gates also cover all prior
save/lifecycle and instrument cache contracts. Renderer test `terminal_faces_cabin`
checks panel normals/UVs. Native walk/use/save/relaunch, power transitions, DPI and
reference frame tails remain separate from portable results.

## Universe U01 (#402)

`cargo test -p salimon-universe --locked` checks canonical sorted interchange,
frozen schema-1 compatibility, exact IDs, missing/uncertain/provenance preservation,
references/cycles, invalid quantities, domain dependencies and schema rejection.
Run workspace build/tests/fmt/all-target Clippy for workspace membership changes.
These are synthetic portable fixtures; no real-data accuracy, production pack,
playable real-scale integration or native performance claim is established.

## Documentation readiness (#440)

Apply [U35's author/reviewer gate](documentation-maintenance.md#readiness-and-reviewer-gate)
to each Universe implementation PR. Check changed local links/anchors, actual
source/module/command paths and semantic correspondence at the final revision;
record results in the PR/report. Documentation-only policy changes need those
checks, not invented native/GPU evidence. Automated repository-wide link/path and
stale-contract CI remains #442; this review policy does not add a CI status check.

## Offline documentation CI (#442)

Run `python scripts/check_docs.py`; for PR source-impact coverage also run
`python scripts/check_docs.py --base origin/main` after committing changes.
Validator contracts: `python -m unittest discover -s scripts -p test_check_docs.py -v`.
The Ubuntu source workflow runs both checks before compilation, without network
link lookups. See [scope, exclusions and exceptions](documentation-maintenance.md#offline-ci-validation-442).

U02: universe tests cover million-cell submeter/GPU differences, boundary carries,
rotating moving-frame roundtrips, proper/radial space motion, invalid hierarchy,
Gregorian leap days, wire bounds and scale/leap-second refusal. They are synthetic
numerical contracts, not real ephemeris or native rendering evidence.

U05: `cargo test -p salimon-persistence --locked` includes universe wire roundtrip,
version/catalog/delta refusal, bad poses/history and torn-slot recovery. Workspace
gates cover the added portable dependency; no playable-world migration is claimed.

U06: `python -m unittest discover -s scripts -p test_import_universe_sources.py -v`
checks no-network reproducibility, hash tampering, paths, unknown versions and
fail-closed production reuse. The source workflow runs it and the fixture importer
plus Rust typed-manifest tests. Production `--production` refusal is expected.

## Data Recovery (#468)

Run workspace tests/build/fmt/all-target Clippy after schema, recipe and DTO changes.
Focused contracts: `cargo test -p salimon-client --locked recovery` and
`cargo test -p salimon-ship --locked auxiliary`. They cover real-domain mine/carry/
craft/Core work, reserve floor, duplicate/mass atomicity, legacy/current restore,
R input, pause and continuation. Shader tests validate the component branch.
Opt-in `recovery_ui_evidence` emits CPU RGBA/size pairs using
`SALIMON_RECOVERY_EVIDENCE` and is not a native screenshot. Reference appearance,
physical route, DPI and budget remain #515.

## Universe binary storage (#405)

`cargo test -p salimon-universe --locked` covers independent version-1 wire decode,
lossless floats/missing provenance, random access independent of corrupt neighbors,
header/index CRC, hostile lengths/ranges/order and payload/index/source mismatch.
`cargo run -p salimon-universe --example partition_bench --release --locked` emits
codec density/timings for frozen observed fields and labeled synthetic replication.
See [storage limits and budget interpretation](universe-storage.md); these numbers
are not a full-catalog size forecast, production publication or native FPS signoff.

## Universe scientific reference fixtures (#404)

`cargo test -p salimon-universe --locked` covers frozen JPL vector/epoch/SI goldens,
analytic production-U02 frames/motion/GPU narrowing, injected unit/axis/epoch/scale
faults, missing/extra/duplicate coverage, malformed wire, finite budgets and norm
boundaries. Release comparator and repeated CPU workloads are documented in the
[reference guide](universe-reference-fixtures.md); retain raw JSON/context/hashes.
Cloud #521 measurements do not approve M1 performance (hardware decisions #563), future ephemeris/integrator
model drift or whole-game frame rates. Reference data is not production reuse.

U07: universe/workspace tests run Gaia/CSV goldens, quality/missing-data counts,
matching, duplicates and malformed input. Run `cargo run -p salimon-universe
--example gaia_bench --release --locked` as one command for synthetic resource
sensitivity; see the [Gaia contract](universe-gaia-ingestion.md) for projection limits.

U08: universe/workspace tests compare the production source importer against U03
Earth goldens, physical headers and Proxima quantities/IDs/errors, plus synthetic
limits/matching/error cases. `cargo run -p salimon-universe --example observed_import --locked`
emits the deterministic development fixture; Source build CI compares two outputs.
See the [observed ingestion contract](universe-observed-ingestion.md) for source
verification, unsupported formats and exact-epoch/refused fallback policy.

## Universe inference (#410)

Run `cargo test -p salimon-universe --locked` and the workspace build/test/Clippy/fmt
gates. `cargo run -p salimon-universe --example inference_sanity --locked` emits
8192 fixed-seed synthetic systems, proposal histograms and numerical/physical
checks; repeat output must compare identically on the tested toolchain. CI runs
this gate and repeat comparison. See [model limits](universe-inference.md) and
[measured evidence](../reports/issue-410/README.md). No empirical frequency fit,
production source approval, runtime migration or native M1 signoff is implied.

## Universe fictional builder (#411)

`cargo test -p salimon-universe --locked` includes U10 source preservation,
identity/order/namespace, input bounds, hashed recipe references, fixed publication
goldens and Roche/Hill/Kepler plus age/metallicity trend checks. Run
`cargo run -p salimon-universe --example fiction_sanity --release --locked` for
four synthetic 64-system distributions, repeat/reorder byte checks and raw/zlib
storage. Source CI repeats/compares the summary and macOS/Windows portable jobs
run the same synthesis goldens in debug and release profiles, retaining failed
publication bytes as CI artifacts; those are CPU contracts, not graphical signoff.
See [file adapter usage and limits](universe-fiction-builder.md#commands-and-validation).
Workspace build/tests/all-target Clippy/fmt and docs/base-impact gates also apply.
## Universe permanent starting world (#412)

Run `cargo test -p salimon-universe --locked starting_system`, workspace
build/tests/all-target Clippy/fmt and documentation/base-impact validation.
`cargo run -p salimon-universe --example starting_system --locked` must reproduce
`client/universe/fixtures/starting-world-v1.json` exactly (including newline).
The `-- --audit` option emits all candidate assessments and definition hashes.
macOS/Windows portable CI runs debug/release U11 goldens; Ubuntu repeats/compares
the manifest. Tests cover independent >=30 ly proof, physical screens, immutable
IDs/digests, altered/unknown/oversized data, single-surface queries, all five
resources, ship-clearing walk paths and visual independence. See
[contract](universe-starting-system.md) and [evidence](../reports/issue-412/README.md).
These CPU checks do not replace U24 integration or cloud #436–#439 rendered and
session acceptance. Human presentation/platform/budget criteria are #561/#557/#563.

## Reference save/HUD fixtures (#376)

Opt-in `SALIMON_NATIVE_REVIEW_DIR=/absolute/new/output cargo test -p salimon-client
--locked native_reference_checkpoints_376 -- --ignored` emits five validated
normal-launch checkpoint fixtures (landed/cockpit/moving/EVA/terminal). The output
must not already exist. Fixtures explicitly set reduced meters, apply production
medkit/damage/crafting transactions and use existing #375 placement fixtures;
they do not prove physical travel or naturally elapsed death. Each checkpoint
round-trips through production validation and the real two-slot disk manager.
Run [the native review harness](../reports/issue-376/native_restore.py) only against
those isolated directories with the locally authorized OS-input helper. Retain
source/binary/adapter/DPI, input/checkpoint records and actual images. Fixture
rendering never grants full gameplay, balance or representative FPS acceptance.

## Normal-clock cargo review fixture (#477)

```sh
SALIMON_NATIVE_CARGO_DIR="$PWD/artifacts/new-native-cargo" cargo test --release -p salimon-client --locked native_cargo_checkpoint_477 -- --ignored --nocapture
```

The output directory must not exist. This test reuses the existing seeded
FragmentPile fixture, advances production ticks, aims at an existing identity,
validates exact save roundtrip with automation absent after restore, and proves
the typed F route can pick that identity. Only then does it write two verified
slots. Use the [#477 native harnesses](../reports/issue-477/README.md) against a
copy in a fresh isolated save directory. Normal launch uses the real wall clock
and native OS events; no stdin stepping or rendering override is active.
Fixture placement/aiming is not physical mining/travel, visual approval or FPS
evidence. Keep blocked placement failures and human temporal review separate.
