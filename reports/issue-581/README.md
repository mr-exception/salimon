# Issue #581 — persistent usage-driven surface age

Delivered October 26, 2026, from main `3bc36c58c2985445685180f864ef3a5ac341b02c`.
The architecture, fixture, material, starter-layout and offline dependencies are
merged. #549 initially returned an open issue snapshot, but its implementation
[PR #635](https://github.com/mr-exception/salimon/pull/635) was verified merged and
present on main, as was #550/PR #636. No active PR stack existed at base selection;
this branch targets main. Nothing was merged by this task.

## Outcome

Persistent integer exposure separates slow passive time from successful component
use. Actual latch/shutter/bunk travel, fabric contact, galley handling, walking,
funded Fabricator work, productive mining and successful mount/rail operations
feed stable physical identities. Fixed local material patterns provide moderate
scuffs, polishing, fading, grip wear and heat discoloration. No texture/material
proliferation, dirt, cleaning chore, mass/capability penalty or age-only failure.

Schema 23 preserves exact exposure and rejects missing/invalid/unknown source;
versions 1–22 migrate pristine without retroactive credit. Recolors retain age;
clean repair removes the real-condition damage variant and retains age. Physical
replacement gets a new ID and fresh exposure. Existing #549 source remains the
only damage/detachment authority; it still protects the permanent starter hull.
Closed-game age and funded work use the existing verified, atomic offline
transaction, with no closed-game occupant contact or ordinary-restore time credit.

[Maintained ownership, rules, limits, source/test mapping and reproduction](../../docs/surface-age.md).
[Source inventory](source-sha256.json) and [compiler/binary/backend context](environment.json)
identify the captured build. Crate maintenance/architecture/invariants, save,
finish, fixture, damage, validation and automation routing were updated together.

## Checks

| Command / evidence | Actual result |
| --- | --- |
| `cargo test --workspace --locked` | 1,024 passed, 38 existing opt-in ignored; [log](checks/workspace.log) |
| `cargo test -p salimon-client --locked surface_age` after final replacement coverage | 5 passed, including actual material-backed replacement; [log](checks/surface-age.log) |
| `cargo test -p salimon-client --locked core_cutoff_pauses_exactly` | Passed; extra assertion proves unfunded Fabricator adds passive age only; [log](checks/funding.log) |
| `cargo build --workspace --locked` | Passed; [log](checks/build.log) |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | Passed; [log](checks/clippy.log) |
| `cargo fmt --all -- --check` | Passed |
| `python -m unittest discover -s scripts -p 'test_*.py'` | 120 passed; [log](checks/python.log) |
| `python scripts/check_docs.py` and `--base origin/main` | Passed; [logs](checks/docs.log), [base impact](checks/docs-base.log) |
| `SALIMON_FIXTURE_EVIDENCE_DIR="$PWD/reports/issue-581/renderer" cargo test -p salimon-renderer --locked modular_fixture_gpu_evidence -- --ignored --nocapture` | Passed; fresh/aged/damage/repair/blackout and 500-fixture controls; frozen-repeat PNG checks; [log](checks/renderer.log) |
| Ten production game capture routes | 20/20 steps passed; [state/pixel comparisons](native/acceptance.json) |

The full workspace run preceded the final added replacement fixture/test. The
final focused suite, build, Clippy and all ten game routes include that addition.
The renderer benchmark source did not change afterward. Ignored tests are not
reported as passes; the explicit renderer evidence run is separate.

## Original production game captures

Each command is `python scripts/salimon-test run scenarios/evidence/surface-age-CASE.json
--binary target/debug/salimon-client --artifacts artifacts/issue-581/delivery-native
--screenshot-command '["native"]' --summary`, under
`/workspace/.salimon-tools/start-display.sh`. This supplies Xvfb `:99`, 1280×800,
Vulkan/llvmpipe and the selected toolchain/sysroot; fixed gameplay step is 16 ms.
These are direct production wgpu readbacks, not Blender or CPU illustrations.

| Declared initial condition | Original capture |
| --- | --- |
| Fresh installed locker | [New](native/new/step-002-surface-age-new.png) |
| 600 actual admitted open/close cycles | [Used](native/used/step-002-surface-age-used.png) |
| One day passive exposure only | [Idle](native/idle/step-002-surface-age-idle.png) |
| Used locker recolored | [Recolor](native/recolor/step-002-surface-age-recolor.png) |
| Used locker during blackout | [Blackout](native/blackout/step-002-surface-age-blackout.png) |
| Exact save/restore of used source | [Reload](native/reload/step-002-surface-age-reload.png) |
| Aged completed stool, authoritative declared impact | [Damage](native/damage/step-002-surface-age-damage.png) |
| Same age, clean repair via production command | [Repair](native/repair/step-002-surface-age-repair.png) |
| Dismantle old stool, reuse exact salvage, complete new ID | [Replacement](native/replacement/step-002-surface-age-replacement.png) |
| One-day closed-game reconciliation, including real Core depletion | [Offline](native/offline/step-002-surface-age-offline.png) |

Each directory retains the final readable `step-002.json` plus losslessly gzipped
result/protocol/first-step/stdout records, setup and original stderr. Comparison
checks show identical used/reloaded component pixels and exposure, different
fresh/used component pixels, identical aged/damaged versus aged/repaired exposure,
cleared real damage and a destroyed old project plus fresh completed replacement.
The fixture renderer additionally requires exact aged/repaired PNG equality.

The replacement setup explicitly moves the camera clear of the existing removal
safety envelope, calls the typed physical dismantling boundary and seeds its exact
recovered fragment into the carry slot; ordinary placement/intake/completion follows.
This proves material and identity accounting, not a physical OS-input pickup route.
The passive seed and declared closed interval do not pretend a real day elapsed.

## Measured renderer workload

Original [adapter context](renderer/adapter.txt), [raw 16-frame cases](renderer/timings.csv)
and [summary](renderer/summary.json). Median samples 2–15, debug build, software
Vulkan, fixture-only offscreen 1280×800:

| 500 independent fixtures | Fresh | Independently aged |
| --- | ---: | ---: |
| CPU prepare, µs | 1,430.193 | 1,443.148 |
| Software GPU timestamp, ms | 1.523586 | 1.545774 |
| Draws / visible instances / triangles | 5 / 500 / 8,400 | 5 / 500 / 8,400 |
| Resident shared geometry, bytes | 18,128 | 18,128 |
| Instance capacity, bytes | 41,760 | 41,760 |
| Frozen-frame upload, bytes | 64 | 64 |
| Staging capacity, bytes | 80,960 | 80,960 |

The matched workload changes age parameters only. No extra age texture, material,
mesh or draw is created per independently aged fixture. Fixture records are
80 bytes (previously 64); ship vertices remain 80 bytes, ship uniform is 608 bytes
(previously 576), and handheld wear uses a spare lane. The actual locker game
frame reports two ship draws, 1,625,040 resident ship vertex bytes and the same
18,128-byte fixture library. Normal/damaged service covers share two existing
mesh batches; destroyed/lost parts submit no attached cover.

## Retained failures and external review

Initial workspace validation exposed four historical-envelope fixtures retaining
new age source; those fixtures now omit source their older schema could not write.
The [original failing run](checks/initial-workspace-failures.log) is retained.
An initial default-sandbox network/ALSA setup failure was resolved with normal
network-enabled dependency access and the user-space ALSA development package;
[log](checks/initial-alsa-setup-failure.log). A Clippy item-order failure was fixed
by moving test-only declarations after production code;
[original log](checks/initial-clippy-failure.log). Later unused-import/compile
iteration failures were corrected before final gates.

The initial native runner lacked its new setup allowlist, then passed after the
supported setup list was extended. The first removal camera overlapped the
conservative safety envelope; a second aim missed reachable physical source.
Those failed setups remain under [initial failures](initial-failures/).
The [rejected floating cover](initial-failures/rejected-floating-cover.png) motivated
attaching the small furniture cover to the actual seat edge; final captures above
use the correction. The rejected image's earlier binary was not archived.
Software Mesa disabled its unwritable home shader cache; rendering still passed.

Human appearance/readability/comfort acceptance remains
[#561](https://github.com/mr-exception/salimon/issues/561): assess moderate localized
wear, clean-repair equality, blackout legibility and cover placement using these
originals. Physical OS input/device/Metal behavior remains
[#557](https://github.com/mr-exception/salimon/issues/557). Actual reference-device
budgets remain [#563](https://github.com/mr-exception/salimon/issues/563); debug
software-Vulkan timestamps do not establish hardware FPS, GPU counters or Metal
performance. These external gates are unrun, not passed or waived.
