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
Cloud production-protocol engine captures, the seven realistic-flight routes and
engine-off disk relaunch regression are recorded in
[the #498 report](../reports/issue-498/README.md). Run its `verify.py` against a
built native executable on Xvfb/software Vulkan; fresh output paths are required.
Human handling/physical input and hardware qualification remain #561/#557/#563.
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

## Structural foundation (#541)

`cargo test -p salimon-ship --locked structure::tests` verifies six-face support,
edge/corner rejection, precise origin/coordinate bounds, multi-deck/multi-cell
membership, safe removal, damage transfers, stable IDs, atomic invalid-input
rejection, disjoint mass contributions and versioned save reconstruction.
Run workspace tests/build/Clippy/fmt for its public mass/save boundaries. The
foundation is not composed into native gameplay; no visual, Metal/M1 or
construction-interaction acceptance is claimed by these headless tests.

## Scout continuous audio verification (#582)

`cargo test -p salimon-client --locked scout_audio` covers production ship cues,
EVA/surface silence, save/focus/terminal admission and spatial mapping.
`cargo test -p salimon-client --locked native_audio` covers continuous callback
chunk invariance, mute/reset/watchdog event retirement and finite headroom.
`condition_feedback` tests retain the survival threshold/settings owner.

Run `SALIMON_AUDIO_EVIDENCE=/absolute/output cargo test --release -p salimon-client
--locked continuous_audio_evidence_582 -- --ignored --nocapture` for original
10-second stereo PCM and 60 Hz synthesis timings. Run the separate
`native_continuous_output_582` opt-in test with the same variable only when an
actual default output device is available; it retains callback telemetry and
fails explicitly if unavailable. Offline exports are not speaker recordings.
Human listening #561, physical backend/device switching #557 and hardware
budgets #563 require their separate evidence. Historical #485 reports remain
valid for their archived subprocess revision. See the maintained
[soundscape contract](scout-presentation.md#soundscape-boundary-465).

## Scout machinery cloud walkthrough (#489)

`cargo test -p salimon-client --locked scout_walkthrough` verifies physical
Fabricator intake/menu/work/blocked/exact collection, unfunded-job Core recovery
and byte-identical pause/save/restore feedback. Existing `scout_feedback`, flight,
cargo, ramp/collision and renderer screen-orientation contracts remain required.
With Xvfb and lavapipe active, run `python reports/issue-489/walkthrough.py
--binary target/debug/salimon-client --output artifacts/new-489-walkthrough`.
It uses production protocol/window adapters and direct GPU capture, isolated
native input, explicit seeded initial states and fixed simulation steps.
Its motion frames/encoded clips are not wall-clock/display-FPS evidence.
See the [report](../reports/issue-489/README.md) for native routes, source/context
hashes, exact results and remaining #561/#557/#563 criteria.

Scout UI cloud matrix #502: the [report and driver](../reports/issue-502/README.md)
record actual wgpu screens at 1280x800/1920x1080 and simulated X11 scales 1/1.5/2,
power restoration through Core intake, physical Fabricator prompts/modal exclusion
and the existing anchored-world-prompt route. Device DPI remains #557 and human
readability/appearance remains #561.

Archive #505: `cargo test -p salimon-client --locked archive` includes the real
Save Now worker, normal relaunch and verified restart reset. The
[cloud report/driver](../reports/issue-505/README.md) captures all seven intact and
damaged records via physical E/arrow/Tab dispatch at both supported resolutions.

Recovery #515: `cargo test -p salimon-client --locked session::recovery` covers
concurrent craft/thrust/recovery funding, old-schema resource/job retention and
real Save Now/normal-launch/restart with zero offline work. The
[cloud report/driver](../reports/issue-515/README.md) exercises production
intake/Fabricator/tray/installation/Archive controls and GPU captures.

Scientific CPU cloud evidence #521: build the three documented release examples,
then use the [report-local measurement driver](../reports/issue-521/README.md)
for seven serial processes each of `reference_bench` and `partition_bench`.
The C supervisor records Linux `wait4` peak RSS without Python's inherited memory
floor. Raw inner JSON, process time/memory, fixture/binary/compiler/CPU context,
control failures and comparisons are retained; no FPS/model-error budget follows.

Blueprints #542: `cargo test -p salimon-ship --locked structure::placement` and
`cargo test -p salimon-client --locked construction` check reservations, safety and
schema-8 atomic restore. Run the production native
`scenarios/evidence/blueprint-placement.json` route for visible previews/projects,
rotation, inspect/cancel and one module spanning four cells. These placeholders
have no intake/progress/completed geometry; those later gates remain #543–#551.

### Exact historical current-sphere stress (#559)

`reports/issue-559/collect.py --binary /absolute/release-test-binary --output
artifacts/559/lane` runs the ignored `current_sphere_stress_cpu` test serially.
It retains every flushed sample and 120s moving / 45s natural-preparation
subprocess timeout. The fixture uses production extraction/adapter snapshots, archived mass/radius
bits and the stateful solver at 16 ms; natural preparation is at most 300 steps, without installed sleep flags.
For the historical #455 comparison use its unchanged pre-starter-layout geometry
and the identical test harness/instrumentation on both builds. See the report
for exact checkout/build commands. Current layout results are separate controls,
not silently the historical v1 population. Query/contact/activation timings
include instrumentation; software/CPU results do not establish hardware FPS.


## Exact contact stress (#592)

Run physics and math tests, workspace tests/build, Clippy and documentation gates
for changes to the shared arithmetic/contact boundary. The retained
`iteration_oracle` compares exact old ground/pair response and warm continuation;
spatial all-pairs, correction-created contact and sleep/retained-projection tests
remain required. The [serial reproducer](../reports/issue-592/reproduce.sh) builds
an isolated archived-layout fixture and records every sample with immutable
binary/source inventories. Never run builds, render workloads or other timing
lanes concurrently with its release measurements. Diagnostic prefix snapshots
cannot replace the required 100 completed moving intervals. Historical failed
300-step preparations and native/frame-budget failures remain failures.

## Physical mounts (#576)

Run `cargo test -p salimon-client --locked session::mounts`, ship/world/character tests and workspace gates. The physical-mount evidence route retains original production-wgpu images. See [the canonical contract](physical-mounts.md).


## Automatic construction (#544)

See [automatic construction contract](construction-progress.md) for active Core-funded timers, safety/pause admission, simple interior completion, schema 17, exact save/refund behavior, production evidence and remaining #545/#547 installation adapters.


## Completed modular structure (#545)

See the [canonical completed construction contract](modular-construction.md) for the ship assembly/shell and runtime installed/work owners, character traversal, per-cell/face identity, material mass transfer, schema-18 reconstruction/migration, partition editing and the production room evidence scenario. Gameplay damage/salvage remains #549; offline construction remains #550.


## Construction flight readiness (#546)

See [construction flight safety](construction-flight-safety.md) for schema-19 Stop/Resume, actual hull and attachment admission, cockpit warnings and airborne stop latches. Source: runtime `session/construction/flight.rs`, ship `structure/construction.rs`; regressions: runtime `session/construction/flight/tests.rs`. Run workspace and construction checks plus the production `construction-interlocks.json` evidence scenario.


## Optional module commissioning (#547)

See [non-Core module contract](non-core-modules.md) for the Archive processor, G/E/T/RMB maintenance, immediate activation downtime, real commissioning faults, paid restoration, schema 20, source/test mapping and deferred Jump Drive follow-up #633.


## Construction damage and salvage (#549)

See the [canonical damage and schema-21 contract](construction-damage.md) for runtime `session/construction/damage.rs`, ship lost-cell geometry, world sealed salvage allocation, controls, exact source/mass/identity boundaries and focused tests/evidence routes. Run workspace tests/build/Clippy/fmt plus documentation validation.


## Offline construction (#550)

See [the maintained chronology and schema-22 contract](construction-offline.md): runtime `session/offline.rs`, ship `controller/offline.rs`, launch/checkpoint staging and existing work owners. Run offline/checkpoint/Recovery and ship tests, workspace build/tests/Clippy/fmt, the two documented wgpu routes and docs/base-impact validation. Future #433 supplies verified universe events; no scanner or orbit implementation is duplicated here.


Cosmetic aging #581: [source/identity, successful use, schema 23, rendering budgets and exact verification](surface-age.md). Damage remains condition-driven; clean repairs restore normal geometry and preserve age.


## Architecture review (#645)

Run `python3 -m unittest discover -s scripts -p 'test_architecture_*.py' -v` and
`python3 scripts/architecture_review.py --base origin/main --output artifacts/architecture/review.json`.
Use the actual parent SHA for stacked PRs. See the [policy and limitations](architecture-baseline.md#warning-only-boundary-and-active-pr-review-645).
CI retains candidates and active-PR inventories without failing valid builds on warnings.

## Integrated Scout and construction acceptance (#551)

The [maintained suite](../scripts/README.md#integrated-scout-and-construction-acceptance-551)
combines production native routes for completed #538–#550 implementations and
exact 0/50/99% cancellation/refund boundaries. Run runtime construction/offline,
ship, persistence and workspace checks plus the suite and supplemental machinery
and Archive drivers. Validate the Scout export with
`python3 models/assets/ships/salimon-scout/validate.py`; retain current measured
asset metrics without claiming pending hardware ceilings are approved. Coverage,
original production images, state and platform limitations belong in the
[completion report](../reports/issue-551/README.md). Human visual/audio review
remains #561, physical devices/sleep/wake #557 and hardware budgets #563.


## Incremental private offline packs (#413)

See the [implemented contract](universe-pack.md) for ownership, inputs, bounds, validation and remaining production/runtime limitations.


## Bounded read-only pack queries (#414)

See the [implemented contract](universe-reader.md) for ownership, inputs, bounds, validation and remaining production/runtime limitations.


## Time-addressable ephemeris models (#415)

See the [implemented contract](universe-ephemerides.md) for ownership, inputs, bounds, validation and remaining production/runtime limitations.


## Dominant-source celestial gravity (#416)

See the [implemented contract](universe-gravity.md) for ownership, inputs, bounds, validation and remaining production/runtime limitations.


## Gravity-aware ship trajectory (#417)

See the [implemented contract](universe-trajectory.md) for ownership, inputs, bounds, validation and remaining production/runtime limitations.


## Real-scale surface flight (#418)

See the [surface-flight contract](universe-surface-flight.md) for rotating/orbiting support,
Core-funded physically bounded landing/takeoff, production source/tests and
remaining playable-world adoption. Current compressed gameplay is unchanged.


## Real-scale atmospheric drag (#419)

See the [atmosphere contract](universe-atmosphere.md) for planet-specific density,
co-rotating gas/wind, atomic online/offline drag integration and physically paid
assist compensation. Model validity, tests and playable-adoption limits are explicit.
