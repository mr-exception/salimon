# #549 — construction damage, physical salvage and safe detachment

Implementation branch: `codex/issue-549-construction-damage`, based on main
`35a5093`. The listed prerequisites #541/#543/#544/#545 are delivered; current
main also contains #546 and the #547 utility commissioning implementation.

The [canonical contract](../../docs/construction-damage.md) records impact,
surviving-site repair, exact salvage, controls, schema 21 and source/test owners.
Original Core/authored hull identities are protected. Damage and cancellation
use separate accounting; destroyed/detached material becomes ordinary physical
fragments, with stable lost-cell tombstones and fresh debris IDs. Restore
rebuilds surviving visible/collision geometry and mass without replaying output.

## Verification record

Linux x86_64 with the existing workspace Rust/sysroot environment:
`. /workspace/.salimon-tools/env.sh`. Required checks use locked dependencies.
The initial missing ALSA libraries were installed in that workspace sysroot;
[initial setup failure](checks/initial-alsa-setup-failure.log) is retained.

The first focused run found live-vs-restored containment/mass disagreement after
detachment. Damage now refreshes geometry and invalidates containment before
mass reconciliation. [Original failures](checks/initial-focused-failures.log)
are retained; corrected focused damage/construction regressions pass.

| Check | Actual result |
| --- | --- |
| `cargo test --workspace --locked --offline` | 1003 passed, 38 existing opt-in ignored, zero failures |
| `cargo test -p salimon-client --locked --offline construction` | Final 49 passed, including the added crash-cell recovery regression |
| `cargo build --workspace --locked --offline` | Passed on final code |
| `cargo clippy --workspace --all-targets --locked --offline -- -D warnings` | Passed on final code |
| `cargo fmt --all -- --check` | Passed |
| `python3 -m unittest discover -s scripts -p 'test_*.py'` | Final 120 passed |
| `python3 scripts/check_docs.py` and `--base origin/main` | Passed, zero findings |
| `git diff --check` | Passed |

[Raw checks](checks/) retain the results. The full workspace run precedes the
final crash-condition correction; the final focused run covers that correction
and the module-repair regression. Final build/Clippy include both, plus the
shorter help hint. A Python runner fixture initially expired its 100 ms subprocess
deadline under concurrent build load; its unloaded rerun passed all 120 tests.
The test suite intentionally prints a mocked failed-profile result after `OK`;
that is fixture output, not a failed test.

## Production renderer evidence

Final routes both passed: **damage 19/19**, **detachment 17/17**. Use the commands
below after starting Xvfb with the existing workspace sysroot/fonts:

```sh
. /workspace/.salimon-tools/env.sh
export DISPLAY=:98 LP_NUM_THREADS=2 WGPU_BACKEND=vulkan
export VK_DRIVER_FILES=/workspace/.salimon-tools/sysroot/usr/share/vulkan/icd.d/lvp_icd.json
python3 scripts/salimon-test run scenarios/evidence/construction-damage.json --binary target/debug/salimon-client --artifacts artifacts/new-549-damage --screenshot-command '["native"]' --summary
python3 scripts/salimon-test run scenarios/evidence/construction-detachment.json --binary target/debug/salimon-client --artifacts artifacts/new-549-detachment --screenshot-command '["native"]' --summary
```

Original GPU readbacks use Vulkan/llvmpipe (Mesa 25.0.7, LLVM 19), 1280×800,
scale 1. [Context and binary hash](evidence/context.json),
[source hashes](source-sha256.txt), [damage trace](evidence/damage-trace.tar.gz)
and [detachment trace](evidence/detachment-trace.tar.gz) retain exact provenance.
The damage fixture starts with one carried 1 kg iron fragment and a reservation;
E physically supplies it, 4.992 seconds are paid, an injected nonfatal hit freezes
progress, H resecures it, paid work completes and a destructive hit releases
0.75 kg. The room fixture supplies the actual 40 kg input and pays 300 seconds;
cutting its attached port cell releases 35 kg across four ordinary physical
pieces, removes all room solids and preserves the original Core.

Original captures:

- [Damaged/paused](evidence/damage-step-011-damaged-and-paused.png),
  [resecured completion](evidence/damage-step-016-resecured-and-completed.png),
  [physical salvage](evidence/damage-step-019-physical-salvage.png).
- [Completed room](evidence/detachment-step-014-completed-before-impact.png),
  [detached debris](evidence/detachment-step-017-detached-debris.png).

[Initial graphical failures](evidence/initial-render-failures.tar.gz) retain the
missing runner-op registration, temporary display loss and capture timeout under
heavy concurrent tests. A persistent display and unloaded rerun pass. Inspection
also found clipped builder help at 1280×800; the final shorter hint is fully visible.
The fixtures are explicit setup/protocol events, not physical keyboard or combat
encounter evidence. No images were promoted to an approved visual baseline.

## Review boundaries

The physical salvage representation is material debris rather than an intact
rigid wreck mesh. The existing crash transition interrupts surviving sites;
localized destruction uses the authoritative typed impact boundary, also injected
by explicitly declared E2E fixtures. No new weapon, pressure network or combat
encounter is claimed. Mechanical H recovery cannot recreate lost material or
restore an unavailable module bonus without its paid restoration lifecycle.

Human presentation/readability review remains
[#561](https://github.com/mr-exception/salimon/issues/561), physical H/X/input and
platform/display checks remain [#557](https://github.com/mr-exception/salimon/issues/557),
and hardware GPU/Metal budgets remain [#563](https://github.com/mr-exception/salimon/issues/563).
Software Vulkan captures do not establish those outcomes or physical 60 FPS.
Chronological offline work is delivered separately in the following #550 stack
entry; scanner/orbital integration remains #433.
