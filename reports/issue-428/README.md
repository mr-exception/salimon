# Issue #428 — Scanner contacts and encounter history

Implemented the portable discovery authority in `client/universe/src/scanner.rs`.
Catalog truth alone creates no player contact. Successful powered observations
validate celestial identity, exact sampled split position/velocity and revision
before atomic publication. Freshness ends at exactly three hours; misses and
blackouts preserve historic encounters without refreshing them. Rediscovery
retains first-seen and replaces sampled state. History/batches are bounded.

Contract and source/test mapping: [scanner](../../docs/universe-scanner.md),
[architecture](../../client/universe/architecture.md),
[invariants](../../client/universe/invariants.md).

Environment: cloud Linux x86_64, rustc/cargo 1.99.1. UTC discovery labels are
nondecreasing POSIX gameplay milliseconds, not scientific time-scale conversion.

Validation on this branch:
- `cargo test -p salimon-universe --locked`: 95 passed, zero failed.
- `cargo clippy -p salimon-universe --all-targets --locked -- -D warnings`: passed.
- `cargo fmt --all`: applied.
- `python3 scripts/check_docs.py --base origin/main`: zero failures.

Initial validation corrected a test's CatalogId constructor and one Clippy
slice-reference warning. The first workspace build found missing ALSA development
libraries in the executor; installed libasound2-dev/libasound2t64 into the existing
external tooling sysroot, with no repository dependency changes.

Pulse scheduling, range/power and offline detection are #429; checkpoint wire and
runtime restoration are #431; cockpit rendering is #430; player chart is #469.
No rendered behavior changes in this domain PR, so no screenshots are claimed.
Physical presentation/platform/hardware qualification remains #561/#557/#563.

- `cargo test --workspace --locked`: 1113 passed, zero failed, 38 existing opt-in ignored.
- `cargo build --workspace --locked`: passed.
- `cargo clippy --workspace --all-targets --locked -- -D warnings`: passed.
- `git diff --check`: passed.

Parent: `fb3d798c67bddf888c9260093e539269361cb6ab` (verified origin/main).
