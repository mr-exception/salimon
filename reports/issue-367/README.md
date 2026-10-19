# Issue #367 — deterministic survival meters

Implemented portable `salimon-survival`: typed commands/snapshots; exact fixed-point
hunger/fatigue/health; 30-minute hunger, 18-hour fatigue, 10-second starvation;
terminal death boundary; validated numeric configuration; serializable fractional
state; launch-only offline fatigue receipt, UTC rollback watermark and capped
crash policy. See [canonical domain contract](../../client/survival/architecture.md).

Validation on Linux, 2026-10-19:
- `cargo test -p salimon-survival --locked`: PASS, 6 integration tests.
- `cargo clippy -p salimon-survival --all-targets --locked -- -D warnings`: PASS.
- `cargo fmt --all`: applied; workspace gates recorded with the final stack.

Coverage includes exact nanosecond thresholds, split/large deltas, death freeze,
18-hour fed timeline, offline-only fatigue, no duplicate grant, rollback/reload,
forward jump cap, saved fractional continuation and invalid schema values.

Native/session integration, startup transactional publication and whole-world
death freeze are explicitly #371 (after #368–#370); this PR provides the portable
owner and does not silently migrate current saves or activate unfinished survival.
No graphical, Apple M1 or native performance coverage claimed for this domain change.
