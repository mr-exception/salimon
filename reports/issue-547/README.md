# Issue #547 — physical optional-module commissioning

## Outcome and scope

The previously unfinished utility-module placeholder now installs a physical
Archive utility processor, can be upgraded, and supports paid restoration after
activated cancellation. Full upgrade funding immediately disables the whole
processor; partial materials/planning do not. Only successful commissioning
publishes its tier-dependent recovery throughput. Core still pays every joule.
Exact material IDs, mass, paid progress, downtime, stop and commissioning faults
survive schema-20 saves. Cancellation refunds `(1-p)` physically and cannot undo
activated maintenance for free. Refund clearance now includes completed objects,
preventing a return from spawning inside the processor and blocking restoration.

The [canonical contract](../../docs/non-core-modules.md) records the authorized
utility defaults, controls, boundaries, source/test mapping and remaining work.
The [owner's scope decision](https://github.com/mr-exception/salimon/issues/547#issuecomment-6400459515)
authorizes these utility defaults while keeping Jump Drive tuning deferred.
[Follow-up #633](https://github.com/mr-exception/salimon/issues/633) retains actual
Jump Drive construction/commissioned-N acceptance. Initial N=20%, no consumable
jump cells and the existing jump/Core physics remain unchanged; no Jump Drive
recipe or operation is claimed. Actual damage producers/salvage remain #549.

## Eligibility and stack

Source: [#547](https://github.com/mr-exception/salimon/issues/547), primary label
`auto-verification`. Its listed blockers #542, #543, #544 and #545 are closed and
implemented. The issue comments resolve the earlier product prerequisite; #549
is described as independent work, not an implementation dependency.

All open PRs were inspected: active [#632](https://github.com/mr-exception/salimon/pull/632),
created 2026-10-25, and older draft [#567](https://github.com/mr-exception/salimon/pull/567).
Neither addresses #547. #632 is the newest active stack entry. The new branch
`codex/issue-547-module-commissioning` starts exactly at its head
`cfd3da370fa3137d1fba60a3de3e2536f866d4c1` and targets
`codex/issue-546-construction-interlocks`.
That base still stopped utility work at 999999 ppm with no installed capability;
its old unsupported-installation regression was updated to require real completion.

Merge order: **#632 first, then this PR**. Retarget this PR to main after the
parent lands, preserving only its incremental module changes. No parent changes
are included as new work, no PR is merged, and no source issue is closed manually.

## Implementation and checks

- Runtime `session/construction/modules.rs`: module/maintenance source, real
  commissioning, immediate activation downtime, G/E/T/RMB commands and bounded
  original receipts. Existing ship `Funding`/`Work` retain exact transactions.
- Runtime construction/work, fabrication mass and persistence: shared physical
  case/collision, one mass owner, schema migration and discrete checkpoints.
- Runtime recovery/Archive: actual 1.25×/1.5× requested watts and matching UI,
  preserving joules per record and the real emergency reserve.
- Native input/protocol, explicit fixture setup and three declarative scenarios.
  Tests cover install, partial supply, upgrade, outage/damage/stop, commissioning
  failure, 50% cancellation/refund, paid old-tier restoration, malformed/duplicate
  saves, real recovery energy, coasting and scarce-surplus nanosecond accounting.

| Check | Final result |
| --- | --- |
| `cargo test --workspace --locked` | 997 passed, 38 existing opt-in ignored, 0 failed |
| `cargo test -p salimon-client --locked session::construction` | 40 passed (included in final workspace run) |
| `cargo build --workspace --locked` | Passed |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | Passed |
| `cargo fmt --all -- --check` | Passed |
| `python3 -m unittest discover -s scripts -p 'test_*.py'` | 120 passed |
| `python3 scripts/check_docs.py` | 0 failures |
| `python3 scripts/check_docs.py --base origin/codex/issue-546-construction-interlocks` | 0 failures |
| `git diff --check` | Passed |

Raw logs are in [checks](checks/); initial missing ALSA development/link inputs
were repaired in the local tool sysroot, without repository dependency changes.
Earlier render failures are retained in
[initial-render-failures.tar.gz](evidence/initial-render-failures.tar.gz): an
inspection deadline, a dead temporary X server, and a capture deadline under
concurrent compilation/tests. A persistent X server and bounded llvmpipe thread
count produced the passing routes below. These failures are not counted as passes.

## Production wgpu evidence

Linux x86_64, Debian 13, kernel 6.18.55, Intel Xeon Platinum 8370C (3 exposed CPUs),
Rust/Cargo 1.99.1, Python 3.12.14. Debug binary hash:
[binary-sha256.txt](binary-sha256.txt). Changed source/guide/scenario hashes are
recorded in [source-sha256.txt](source-sha256.txt).

Xvfb :97 uses a 1280×800 drawable, scale 1. Actual renderer observations identify
**llvmpipe (LLVM 19.1.7, 256 bits), Vulkan, CPU device**. Mesa 25.0.7 comes from the
local sysroot; `LP_NUM_THREADS=2` bounds software renderer concurrency. Input is
protocol-to-native key dispatch; captures are original production GPU readbacks.
The scenarios use seed 547 and 16 ms fixed steps with isolated process/save state.

```sh
. /workspace/.salimon-tools/env.sh
# Persistent display, started once in its own terminal:
Xvfb :97 -screen 0 1280x800x24 -nolisten tcp -ac -fp /workspace/.salimon-tools/sysroot/usr/share/fonts/X11/misc
# For each scenario (installation, upgrade and restoration respectively):
DISPLAY=:97 WGPU_BACKEND=vulkan VK_DRIVER_FILES=/workspace/.salimon-tools/sysroot/usr/share/vulkan/icd.d/lvp_icd.json LP_NUM_THREADS=2 RUST_LOG=info python3 scripts/salimon-test run scenarios/evidence/construction-module.json --binary target/debug/salimon-client --artifacts artifacts/issue-547/verified --screenshot-command '["native"]' --summary
DISPLAY=:97 WGPU_BACKEND=vulkan VK_DRIVER_FILES=/workspace/.salimon-tools/sysroot/usr/share/vulkan/icd.d/lvp_icd.json LP_NUM_THREADS=2 RUST_LOG=info python3 scripts/salimon-test run scenarios/evidence/construction-module-upgrade.json --binary target/debug/salimon-client --artifacts artifacts/issue-547/verified --screenshot-command '["native"]' --summary
DISPLAY=:97 WGPU_BACKEND=vulkan VK_DRIVER_FILES=/workspace/.salimon-tools/sysroot/usr/share/vulkan/icd.d/lvp_icd.json LP_NUM_THREADS=2 RUST_LOG=info python3 scripts/salimon-test run scenarios/evidence/construction-module-restore.json --binary target/debug/salimon-client --artifacts artifacts/issue-547/final --screenshot-command '["native"]' --summary
```

| Route | Result | Declared initial conditions |
| --- | --- | --- |
| Installation | 16/16 steps passed | One valid reserved module and one carried 3 kg iron fragment; E intake pays the full 60 s |
| Upgrade | 27/27 steps passed | Prior commissioned tier 1 with paid 3 kg/60 s history; one carried 2 kg input; G plans, E disables, T freezes/resumes, 90 s commissions tier 2 |
| Restoration | 12/12 steps passed | Prior tier 1, paid half upgrade and actual cancellation/1 kg refund; one new carried 1 kg input; G/E and 30 s restore tier 1 |

These are declared fixtures, not a mining journey or physical keyboard test.
All later material transfer, work, commissioning and native command admission
use production owners. The recovery unit regression separately verifies real
6.25 MW debit/throughput at tier 1 and return to 5 MW during activated downtime.

Original images:

- [Reserved](evidence/install-step-002-planned.png),
  [half built](evidence/install-step-010-half-built.png),
  [commissioned tier 1](evidence/install-step-016-commissioned.png).
- [Planned upgrade](evidence/upgrade-step-005-planned-upgrade.png),
  [offline work](evidence/upgrade-step-014-offline-upgrade.png),
  [commissioned tier 2](evidence/upgrade-step-027-tier-two.png).
- [Cancelled and still offline](evidence/restore-step-002-cancelled-offline.png),
  [paid restoration](evidence/restore-step-012-restored-tier-one.png).

[Evidence index](evidence/index.json) retains original paths/results.
[Install trace](evidence/install-trace.tar.gz),
[upgrade trace](evidence/upgrade-trace.tar.gz) and
[restoration trace](evidence/restore-trace.tar.gz) losslessly retain scenario,
step states, protocol, process logs and result metadata. PNGs are unchanged.
[SHA256SUMS](SHA256SUMS) covers the retained artifacts.

## Limits and transferred review

Human presentation/readability/spatial review remains
[#561](https://github.com/mr-exception/salimon/issues/561): inspect the original
reservation/build/case and online/offline/restoration hints, including physical
case placement. Physical G/E/T/RMB/carry, display and OS behavior remain
[#557](https://github.com/mr-exception/salimon/issues/557). Actual hardware/Metal
budgets remain [#563](https://github.com/mr-exception/salimon/issues/563).
Software Vulkan does not certify those outcomes or physical 60 FPS. No actual
hardware GPU metrics, M1/Metal or Windows qualification was run. Source rights and
future tuning decisions remain #537; deferred Jump Drive delivery is #633.
Chronological offline construction remains #550. The shared non-Core lifecycle
does not imply actual combat, detached wreck physics or general propulsion-module
upgrades were added here.
