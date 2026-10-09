# Issue #132 — Toolbar scenarios and controls

Implemented against main `d82db45` on 2026-10-09. Prerequisites #126 and
#128–#131 are merged; the earlier blocked assessment no longer applies.

## Outcome

The carrying baseline/evidence routes now assert the five-slot initial loadout,
absent initial selection, and equip decisions for every slot. At a reachable
surface deposit, each empty slot 2–5 rejects held F mining with no extracted
mass or fragments and no held tool/target. Slot 1 restores the tool and mines.
The existing route continues through equipped F pickup, all-five-slot carrying
lockout, deselected F drop and explicit re-equipping without resuming mining.

The evidence variant adds initial-loadout, empty-selected and equipped-tool
checkpoints; existing carrying and post-drop checkpoints complete the four
required states. Baseline and evidence share every action/assertion. Existing
runtime tests execute both complete JSON routes, and existing native CI already
runs them through baseline discovery/resource-collection evidence. No new
inspection field, test-only action or gameplay implementation was needed.

Updated root controls, runtime maintenance/architecture/invariants, the feature
map and runner coverage. Corrected stale E pickup/drop and procedural resource
presentation descriptions in the runtime guide to match production F input
and authored meshes.

## Validation

Ubuntu 24.04 x86_64, Rust/Cargo 1.99.0, Python 3.12. Native graphics use Xvfb
1280×800 and Mesa lavapipe software Vulkan. Build commands used:

```sh
export CARGO_BUILD_JOBS=1 CARGO_INCREMENTAL=0
export CARGO_PROFILE_DEV_CODEGEN_UNITS=1 CARGO_PROFILE_TEST_CODEGEN_UNITS=1
export CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1
```

| Command | Result |
| --- | --- |
| `cargo fmt --all -- --check` | Passed |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | Passed |
| `cargo test --workspace --locked` | Passed: 282 tests; one existing ignored GPU test |
| `cargo build --workspace --locked` | Passed |
| `python -m unittest discover -s scripts -p 'test_*.py'` | Passed: 34 tests, including evidence/baseline synchronization |
| `python scripts/build_game.py --profile debug --output artifacts/build/debug` | Passed |
| `python scripts/salimon-test suite --binary target/debug/salimon-client --artifacts artifacts/issue-132-baseline` | Passed: all 13 baseline scenarios, including 307-step carrying |
| `python scripts/salimon-test run scenarios/evidence/carrying.json --binary target/debug/salimon-client --artifacts artifacts/issue-132-evidence --screenshot-command '["python", "scripts/capture_settled.py", "{path}"]'` | Passed: 320 steps and 13 captures |
| `python scripts/build_game.py --profile release --output artifacts/build/release` | Passed |
| `python scripts/salimon-test run scenarios/carrying.json --binary artifacts/build/release/salimon-client --artifacts artifacts/issue-132-release` | Passed: 307 steps against staged release |
| `python scripts/packaged_smoke.py --binary artifacts/build/release/salimon-client --timeout 90 --artifacts artifacts/issue-132-packaged` | Passed: launch/presentation, real OS F2 round trip, captures |
| `git diff --check` | Passed |

Native commands share a TCP Xvfb display with their runner process, using
`DISPLAY=127.0.0.1:75`, `WGPU_BACKEND=vulkan` and `VK_DRIVER_FILES` pointing
to the local Mesa `lvp_icd.json`. The local extracted native binaries/libraries
were added to `PATH`/`LD_LIBRARY_PATH`. The staged-release carrying/package checks used display `127.0.0.1:74` at
1920×1080 with the same native libraries/backend. Separate execution sessions have isolated
network namespaces, so launching Xvfb in a separate session does not work.

Initial default and one-build-job Rust attempts failed with zero-length objects
in third-party naga/wgpu-core archives. Disabling incremental compilation and
using one codegen unit resolved this; all reported final Rust gates passed.


## Evidence

[Selected authoritative state snapshots and native results](native-checks.json).
These fresh images were visually inspected at 1280×800:

| State | Evidence |
| --- | --- |
| Initial loadout, no selection | [Initial toolbar](toolbar-initial-loadout.png) |
| Slot 1 selected, held tool visible | [Equipped mining tool](toolbar-mining-tool-selected.png) |
| Empty slot 2 selected, tool hidden | [Selected empty slot](toolbar-empty-slot-selected.png) |
| Carrying, no highlight or held tool | [Carrying](carrying-first-fragment.png) |
| After drop, still deselected | [Post-drop](dropped-toolbar-deselected.png) |

## Limitations

Software Vulkan establishes gameplay/render/capture coverage, not macOS/Windows
hardware fidelity or reference-machine performance. No authored asset, native
key mapping or lifecycle code changed; Blender/model authoring and manual
resize/minimize/reference-machine performance checks were not applicable.
Cross-platform release checks are also enforced by GitHub CI.

