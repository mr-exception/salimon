# Validation matrix

Run commands from the repository root. Use `python3` on macOS/Linux where
`python` is not available, and `python` on Windows. Native builds require Python
3.10+, stable Rust 1.89+ with rustfmt/Clippy, locked Cargo dependencies, and the
[platform build prerequisites](../scripts/BUILDING.md#platform-prerequisites).
CI uses Python 3.12. Initial installation/builds require network access.

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

## Native graphical gates

A running display, compatible GPU/backend and an unobscured capture surface are
required. After release staging, the Linux CI-equivalent commands are:

```sh
export WGPU_BACKEND=vulkan
export VK_DRIVER_FILES=/path/to/lvp_icd.json
xvfb-run -a -s '-screen 0 1280x800x24' python scripts/salimon-test suite --binary artifacts/build/release/salimon-client
xvfb-run -a -s '-screen 0 1280x800x24' python scripts/salimon-test run scenarios/evidence/lower-cockpit-windows.json --binary artifacts/build/release/salimon-client --screenshot-command '["python", "scripts/capture_settled.py", "{path}"]'
xvfb-run -a -s '-screen 0 1280x800x24' python scripts/salimon-test suite --group resource-collection --evidence --binary artifacts/build/release/salimon-client --screenshot-command '["python", "scripts/capture_settled.py", "{path}"]'
xvfb-run -a -s '-screen 0 1280x800x24' python scripts/salimon-test suite --group ship-eva --evidence --binary artifacts/build/release/salimon-client --screenshot-command '["python", "scripts/capture_settled.py", "{path}"]'
xvfb-run -a -s '-screen 0 1920x1080x24' python scripts/packaged_smoke.py --binary artifacts/build/release/salimon-client --timeout 90
```

Use the actual installed lavapipe ICD filename (CI discovers
`/usr/share/vulkan/icd.d/lvp_icd*.json`); do not copy the placeholder. Install
Xvfb, xauth, xdotool, ImageMagick and Mesa Vulkan plus desktop libraries as in
[CI](../.github/workflows/native-build.yml). Check `import -window root` capture
works on the test display. Screenshot/capture failures are failures, not skips.

On macOS/Windows use a local graphical desktop, omit Xvfb/Linux backend variables,
and select the staged executable (`salimon-client.exe` on Windows). Use the
[runner guide](../scripts/README.md) for platform capture configuration and
[packaged smoke prerequisites](../scripts/PACKAGED_SMOKE.md) for OS input/capture.
CI runs Rust/Python/build gates on all three OSes, required baseline/evidence and
packaged smoke on Linux only, and optional `scenarios/evidence/*.json` checkpoints
on `workflow_dispatch` with `visual_evidence=true`.

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
