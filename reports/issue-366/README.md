# Issue 366 — durable active-run checkpoints

Implemented on the merged #363–#365 foundation (PRs #388–#390); those issues
were already closed when this task started. No duplicate PRs were created.

`salimon-persistence::storage` provides a portable store boundary, alternating
verified generations, CRC32 over generation/length/payload, bounded reads,
schema/domain reconstruction checks and recovery. Native storage uses an
exclusive crash-safe filesystem lease, same-directory temporary writes, file
flush/rename, Unix directory flush and readback. On Windows only the inactive
slot is removed before rename; the current verified slot remains intact.

Runtime `checkpoints.rs` restores before gameplay, captures on first update,
60 simulated active seconds and discrete gameplay events, and verifies final
clean-exit state. One bounded worker performs durable IO/reconstruction checks;
new events coalesce while busy. Capture/serialization remains on the session
thread. Save failures log, warn in the title and retry after five game seconds.
Corrupt latest falls back visibly; no valid generation starts fresh visibly.
Unknown schema/unreadable files reject launch. E2E/benchmarks cannot touch the
user save. Saved admitted backlog survives first renderer creation. No offline
world advancement or input replay occurs. See [canonical contract](../../docs/save-contract.md).

## Validation

Linux x86_64, Rust 1.99.1, debug profile:

- `cargo test --workspace --locked`: **583 passed, 0 failed, 22 existing opt-in
  ignored**, including compile-fail documentation contracts.
- `cargo test -p salimon-client --locked checkpoints`: **3 passed** after the
  final discrete-event signature adjustment. Covers the exact 59.999/60-second
  boundary, equipment event, complete startup/exit/fallback and a real native
  filesystem 500-fragment round-trip with exact IDs/mass/clocks/source bytes.
- `cargo build --workspace --locked`,
  `cargo clippy --workspace --all-targets --locked -- -D warnings`,
  `cargo fmt --all -- --check`, `git diff --check`: passed.
- `python -m unittest discover -s scripts -p 'test_*.py'`: **93 passed**.
- Storage tests inject disk-full/interrupted partial publication, invalid domain
  payloads, unsupported schemas, CRC/payload/generation corruption, oversize,
  orphan temporary files, concurrent writer lease rejection and filesystem restart.
- Linux native packaged smoke: `scripts/packaged_smoke.py --binary
  target/debug/salimon-client --timeout 60 --artifacts artifacts/issue-366-smoke`
  passed window creation, Vulkan presentation, screenshot capture and OS F2
  round-trip. Used Xvfb over local TCP (`DISPLAY=127.0.0.1:91`),
  `WGPU_BACKEND=vulkan`, lavapipe ICD and an isolated `SALIMON_SAVE_DIR`.
  [Screenshot](gameplay-linux.png); 1920×1080 game window on a 1280×800 capture
  display is cropped, so this is launch/input evidence, not layout signoff.
- Separate three-launch normal-play lifecycle probe sent actual
  `WM_DELETE_WINDOW`, verified process exit 0 and checkpoint CRC/generation,
  relaunched the same save, corrupted the newest generation, and verified the
  visible recovery title and clean exit. [Structured results](native-lifecycle.json),
  [recovered-launch log](native-recovery.log). Generations 2 → 4 → 5.

Initial workspace build encountered stale local `salimon_ship` metadata
(`E0463`); `cargo clean -p salimon-ship` followed by build passed. Initial Xvfb
Unix-socket smoke could not open the display in this container; local TCP Xvfb
passed. These were environment/setup failures, not waived gates.

## Remaining owners and limits

Survival/death state does not yet exist. Existing #371 owns terminal checkpoint
and restart integration; #372/#374 own the journey archive and restart screen.
Their integration must retire both prior generations so fallback cannot revive
a dead run. #382 owns Save Now UI. Expanded survival E2E and reference M1/Metal
signoff remain #375/#376; Windows/macOS filesystem/GUI execution is unrun.
No native FPS/performance, reference hardware, fatigue or death behavior is
claimed from these Linux/debug checks. No asset/model source changed.
