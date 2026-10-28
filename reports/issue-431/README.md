# Issue #431 — Scanner checkpoint and freshness restoration

Added bounded version-1 contact checkpoints with exact catalog revision, celestial
IDs, explicit coordinate frames, sampled states, first/last-seen timestamps and
refresh counts. Reconstruction checks duplicate IDs, timestamp order, counts,
state and limits before publication. Catalog mismatch is a fatal incompatible
catalog error rather than silent reseeding or a fresh galaxy reveal.

Runtime schema 25 records scanner parameters, logical UTC and sub-millisecond
remainder for canonical sessions. Missing current source rejects. Canonical
schemas 1–24 migrate empty at checkpoint UTC; historical compressed fixtures
omit scanner source. New native startup/restart binds an empty scanner clock to
its supplied checkpoint UTC. Derived power/error/presentation state is transient.

Launch advances freshness through nonnegative elapsed time once after existing
powered-work reconciliation; rollback/repeated launch does not replay credit.
History survives expiry; ordinary paid scanning rediscovers expired contacts.
The absent #433 trajectory/funding host never grants imaginary offline contacts.

Contract/source mapping: [scanner](../../docs/universe-scanner.md),
[save contract](../../docs/save-contract.md), owning universe/runtime/persistence
architecture and invariants updated together. Cloud Linux x86_64, Rust 1.99.1.
No rendered changes; physical/platform/hardware qualification stays #561/#557/#563.

Focused verification:
- `cargo test -p salimon-universe --locked scanner`: 7 passed.
- `cargo test -p salimon-client --locked scanner`: 3 passed, including real
  save/restore, launch expiry/no replay/rediscovery and atomic corrupt-source rejection.
- `cargo test -p salimon-persistence --locked`: 11 passed.
- `cargo clippy --workspace --all-targets --locked -- -D warnings`: passed.
- `cargo test --workspace --locked`: 1,121 passed, zero failed, 38 existing opt-in ignored.
- `cargo build --workspace --locked`: passed.
- `cargo fmt --all -- --check`, `git diff --check`: passed.
- `python3 scripts/check_docs.py --base codex/issue-429-scanner-pulses`: zero failures.
