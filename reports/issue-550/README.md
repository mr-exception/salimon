# #550 — durable construction and eligible closed-game work

Second entry in the requested stack, based exactly on #549
[PR #635](https://github.com/mr-exception/salimon/pull/635) at
`2c174f1343e0452042303e424b5df94d0274c004`. Merge #635 before this PR.
The #543/#544/#546 prerequisites are merged; #547's utility commissioning code
and its resolved defaults are present in the inherited main reconciliation.
The deferred Jump Drive recipes/tuning remain #633 and are not invented here.

The [canonical contract](../../docs/construction-offline.md) records schema 22,
real Core funding, safety/commissioning and atomic launch publication. Runtime
`session/offline.rs` stages chronology through existing work owners; ship
`controller/offline.rs` settles closed power without impulse/steering or new
engine/assist/jump authority. Saved clock/version/Core source/safety records,
original receipts and exact work survive checkpoints. Rollback/repeated resume
cannot buy time twice; old schemas migrate without retroactive work.

## Checks and corrected results

Use `. /workspace/.salimon-tools/env.sh` on this Linux x86_64 executor. Rust
1.99.1, Python 3.12.14, kernel 6.18.55. The existing sysroot contains native build,
ALSA, Xvfb and Mesa inputs; no dependencies or numerical tolerances changed.

| Check | Actual result |
| --- | --- |
| `cargo test --workspace --locked --offline` | 1018 passed, 38 existing opt-in ignored, zero failures |
| `cargo test -p salimon-client --locked --offline session::offline` | Final 13 passed |
| Checkpoint tests / Recovery tests | 20 passed + 1 existing ignored / 8 passed + 1 existing ignored |
| `cargo test -p salimon-ship --locked --offline offline` | 1 passed; included in workspace |
| `cargo build --workspace --locked --offline` | Passed on final code |
| `cargo clippy --workspace --all-targets --locked --offline -- -D warnings` | Passed on final code |
| `cargo fmt --all -- --check` | Passed |
| `python3 -m unittest discover -s scripts -p 'test_*.py'` | 120 passed |
| `python3 scripts/check_docs.py` and `--base codex/issue-549-construction-damage` | Passed, zero findings |
| `git diff --check` | Passed |

[Raw logs](checks/) preserve the results. The full workspace run precedes the
last derived gravity/audio cleanup; all 13 offline regressions, final build and
all-target Clippy include it. The exhausted-Core regression asserts disabled
ship gravity and no surviving live construction-audio cue before exposure.

The tests establish exact live-vs-closed progress/energy, elapsed partitions,
retained partial ticks, early reserve depletion, real conversion restart,
competing Fabricator demand and isolated captured assist reservations, precise
damage/departure/landing boundaries, no landing auto-resume, pause/close separation,
utility upgrade commissioning, 300-second/four-cell room publication, stable mass,
malformed/legacy clocks and real native-store publication failure/retry/replay.
The long blocked-absence test settles one year without a progress cap or free work.

Initial failures are retained in `checks/`: a comparison initially omitted the
unbilled 8 ms remainder after a verified event; old whole-save tests expected
schema-20 machine/Core freezing and lacked the new rollback watermark; an old
schema-7 fixture retained an impossible schema-22 safety record. Corrected tests
now check actual timed power/Fabricator/Recovery behavior while preserving exact
cargo, survival, identity and rollback assertions. The earlier Clippy initializer
style failure is retained separately. These failures are not counted as passes.

## Production renderer evidence

Final routes passed **13/13** construction and **14/14** upgrade steps. Both use
the production checkpoint capture/launch-reconciliation boundary and wgpu GPU
readbacks, with Vulkan/llvmpipe (Mesa 25.0.7, LLVM 19), Xvfb at 1280×800 and
scale 1. [Context/binary hash](evidence/context.json),
[source hashes](source-sha256.txt), [construction trace](evidence/offline-trace.tar.gz)
and [upgrade trace](evidence/upgrade-trace.tar.gz) retain provenance.

```sh
. /workspace/.salimon-tools/env.sh
export DISPLAY=:98 LP_NUM_THREADS=2 WGPU_BACKEND=vulkan
export VK_DRIVER_FILES=/workspace/.salimon-tools/sysroot/usr/share/vulkan/icd.d/lvp_icd.json
python3 scripts/salimon-test run scenarios/evidence/construction-offline.json --binary target/debug/salimon-client --artifacts artifacts/new-550-offline --screenshot-command '["native"]' --summary
python3 scripts/salimon-test run scenarios/evidence/construction-offline-upgrade.json --binary target/debug/salimon-client --artifacts artifacts/new-550-upgrade --screenshot-command '["native"]' --summary
```

The first route supplies a declared carried 1 kg fragment with E, pays 4.992 live
seconds, captures a checkpoint and reconciles six closed seconds. It verifies the
completed object and watermark 106. The second starts from the declared paid
utility tier-1 fixture, uses actual G/E upgrade activation/downtime and reconciles
96 closed seconds before verifying tier 2. These are explicit seeded/protocol
fixtures, not physical wall-clock waits or disk-failure evidence. The separate
native-store test establishes failed-publication isolation and durable retry.

Original final images:

- [Construction completed after resume](evidence/offline-step-013-completed-after-verified-resume.png).
- [Tier 2 commissioned after resume](evidence/upgrade-step-014-commissioned-after-verified-resume.png).

No image was promoted to an approved baseline. Both final routes passed without
capture or protocol failures.

## Scope and remaining owners

Current native catch-up retains the saved ship/region frame, cargo and captured
assist paths. It advances actual service, conversion, Fabricator/Recovery and
eligible construction power/work; health/hunger/effects, active survival/world
clock and journey distance remain frozen, with the existing separate fatigue
policy. Offline power never applies thrust or enables an engine/Jump Drive.

The typed ordered-event seam accepts damage and verified motion boundaries.
Future [#433](https://github.com/mr-exception/salimon/issues/433) remains the owner
of authoritative Universe trajectory/region events, scanner windows, ephemerides
and captured-path chronology. This PR neither duplicates those implementations
nor claims that event producer exists in the current compressed-world host.
No event or safe landing is guessed from wall time; supplied invalid/late events
reject the whole candidate. This retains #433's existing scope and queue gate.

Human appearance/readability review of post-resume geometry/status remains
[#561](https://github.com/mr-exception/salimon/issues/561); physical keyboard,
real close/OS/platform behavior remains [#557](https://github.com/mr-exception/salimon/issues/557).
Actual hardware/Metal budgets remain [#563](https://github.com/mr-exception/salimon/issues/563).
Those criteria remain unrun here; software Vulkan timings are not hardware FPS.
