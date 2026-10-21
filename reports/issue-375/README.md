# Issue #375 — Persistence and survival cross-system verification

Delivered portable scenario extensions, whole-session disk/launch/continuation
coverage and checkpoint worker/restart failure injection. Production gameplay,
save schemas and balance values are unchanged. Base: `22d5fcc` (engine #493).

## Coverage

| Requirement | Evidence / owner |
| --- | --- |
| Landed/moving/outside-EVA/cargo/blackout/Core/terrain full saves | `complete_disk_roundtrip_matrix_and_deterministic_continuation`: seven production fixtures, active healing/burning, fractional ticks and crafting/inventory, real filesystem checkpoints and normal launch. Full save source and subsequent updates match. EVA explicitly starts outside with inherited velocity; fixture placement does not claim a walked exit. |
| Crafting, tray, Core power and Q consumption | [Portable JSON scenario](../../scenarios/portable/persistence-survival.json): blackout stalls crafting; partial progress reloads; completion/tray/third food slot; pause freeze; Q medkit removal and active-healing continuation through reload. |
| ~500 physical fragments, depletion/streaming and IDs | Extended `persistence_extended_500_fragments_depletion_streaming_and_identity_high_water`: 500 deck fragments plus a consumed source, restored depletion on stream reconciliation, no ID reuse, exact mass/inventory/FIFO and physics continuation. |
| Offline fatigue only; replay and clock rollback | `disk_resume_freezes_everything_except_fatigue_and_replay_is_idempotent`: real normal-launch transactions, six-hour recovery, repeated/rolled-back UTC and exact equality of every saved field except fatigue/watermark. Active effects, motion, energy and crafting stay frozen. |
| Crash during publication / disk full / manual Save & Exit | `checkpoints/scenarios.rs`: errors before publication, torn bytes and complete publication followed by IO error. No false save/exit receipt; visible paused session stays unchanged; fresh process recovers a valid generation with cleared input/modal state. |
| Death, archive and restart atomicity | Same module injects all three fault modes at each of five transaction operations (two terminal copies, archive, two fresh copies). Failed restart leaves visible terminal state; recovery converges to one immutable report/run 2; corrupting either slot after acknowledged restart cannot resurrect run 1. |
| Invalid domain/cross-frame references and versions | Atomic rejection of health/door/tutorial/anchor/motion payloads. Integrity-valid but domain-invalid generation falls back; future schema aborts real launch without replacing visible gameplay. Existing storage tests retain stale-temp-file/crash-restart, readback, leases and platform boundaries. |
| 30-minute hunger, 18-hour fatigue, six-hour recovery, exact ten-second starvation | `persistence_scenarios/rates.rs`: 16/33/100 ms hunger partition + serialization, exact nanosecond starvation edge; repeated 18-hour save boundaries and recovery. Extended matrix compares every 900 seconds of the full 18-hour domain run across all three frame sizes. |
| Fixed-tick/death routing and zero-fatigue permissions | Split session advances with repeated full saves match an uninterrupted moving session at exact death time and final report; exhausted restored pilot/mining/fabrication/carry permissions deny, Q still consumes exactly one ration. |
| Q release/cancel/focus/repeat/click-through (#377) | Workspace retains `session/consumable_wheel.rs` tests and `app/input_tests.rs::wheel_released_pointer_still_admits_q_release_and_focus_loss_cancels_without_replay`, plus pause/death coordination tests. New disk/worker scenarios compose Q with restore and failed saves. |
| Four foods/unlocks (#378), starter grant exactly once (#379) | Existing `session/consumables/tests.rs::four_food_definitions_share_fabricator_core_exact_consumption_and_full_save`, Fabricator recipe tests and `session/starting_supplies_tests.rs`, including verified native-store launch/restart/reload; all ran in workspace tests. |
| Persistent/skip tutorial (#380), pause/manual save UX (#381/#382) | Existing tutorial persistence/migration/input tests, pause full-save freeze, native-host admission tests and checkpoint control tests all ran. New DSL persists tutorial source alongside gameplay and adds full-save pause freeze; worker faults cover manual-save failure. |
| Global accessibility (#383) and balance scenarios (#384) | Existing `condition_feedback/settings.rs` persistence/legacy/corruption tests, feedback/audio owner tests and all six `survival_balance` scenarios ran. Preferences remain separate from run saves. No provisional balance or audiovisual acceptance is granted here. |

The strict versioned [test-only DSL and commands](../../scenarios/portable/README.md)
add checkpoint/reload/continuation operations around existing protocol requests.
They are intentionally absent from the live automation protocol. Temp directories
are isolated and cleaned after tests; player save paths and real clock sleeps are
not used for deterministic assertions. Real manual-save workers still use the
production UTC provider; assertions compare authoritative gameplay at fixed UTC.
Fault injection is at `CheckpointStore`, not an actual process kill or full disk.

## Verification results

Linux x86_64, Rust 1.99.1, Python 3.12.14; CPU-only cloud environment, 2026-10-21.

| Command | Result |
| --- | --- |
| `cargo test --workspace --locked` | PASS: 754 passed, 0 failed, 25 ignored, including 320 runtime tests. [Raw log](workspace-tests.log). |
| `cargo test -p salimon-client --locked persistence_extended -- --ignored` | PASS: both extended tests, 0 ignored, 2.98 seconds on this host. [Raw log](extended-tests.log). |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | PASS. [Raw log](clippy.log). |
| `cargo build --workspace --locked` | PASS. [Raw log](build.log). |
| `cargo fmt --all -- --check` | PASS, [empty success log](format.log). |
| `python -m unittest discover -s scripts -p 'test_*.py'` | PASS: 95 tests. [Raw log](script-tests.log); printed failed/timeout JSON belongs to intentional tooling fixtures. |
| Changed Markdown local targets and `git diff --check` | PASS. |

The two extended cases are among the normal workspace's 25 intentionally ignored
opt-in tests; they were separately executed above. Other ignored cases include
unrelated GPU/evidence/benchmark suites. Full workspace tests ran after the final
regular scenario edits; the final ID-high-water extension touched only the
extended 500-fragment test, which was rerun with both extended tests and Clippy.
Earlier scenario development failures were fixture mistakes (host automation is
not persisted, hover is zero-based, mass needs fixture synchronization), corrected
before these passing runs; no production behavior was relaxed.

## CI and boundaries

Existing PR Ubuntu workspace tests run the bounded matrix automatically. The
existing nightly Rust step additionally executes only `persistence_extended`
with `--ignored`, preserves its raw output under periodic artifacts and fails the
existing Rust gate if it fails. No new hosted service or workflow trigger.

Native reference screenshots, OS input delivery, audible quality, physical-route
balance and M1 performance are UNRUN here. Their existing owners remain
[#348](https://github.com/mr-exception/salimon/issues/348),
[#376](https://github.com/mr-exception/salimon/issues/376),
[#496](https://github.com/mr-exception/salimon/issues/496) and
[#498](https://github.com/mr-exception/salimon/issues/498), with the Scout-specific
native follow-ups unchanged. Portable survival advancement is tested directly for
long durations; it is not an eighteen-hour rendered/native gameplay run. These
are existing verification boundaries, not unresolved portable #375 work.
