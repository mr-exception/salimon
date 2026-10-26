# Issue #638 — architecture baseline

Implemented a deterministic first-party Rust audit and non-blocking CI artifact,
with path-level size/branch proxies, module/import inventory, crate cycles and
30-PR first-parent co-change counts. The maintained baseline/ownership map is in
[architecture-baseline](../../docs/architecture-baseline.md). No gameplay/save code changed.

Validation on Linux x86_64, Python 3.12.14:
- `python3 -m unittest discover -s scripts -p test_architecture_audit.py -v`: 3 passed.
- `python3 -m unittest discover -s scripts -p 'test_*.py'`: 123 passed.
- `python3 scripts/architecture_audit.py --history-ref 69c8394 --output artifacts/architecture/current.json --summary`: passed; 279 files, no observed crate-reference cycles.
- Repeated audit output at the same working tree/ref: deterministic fixture contract passed.
- `python3 scripts/check_docs.py --base origin/main`: 0 failures.
- `git diff --check`: passed.

Source baseline: `69c8394`. Current report scans the working tree while pinning
history. Scanner limitations and proposed thresholds are documented; local
PR-tagged history is not GitHub merge-status proof. Actual merge-conflict rates,
resolved Rust imports and active-PR overlap are not inferred. CI enforcement is
intentionally deferred to #645. Rust/native runtime checks are not required for
this tooling/docs/workflow-only change; initial environment compilation found
missing ALSA development metadata, now supplied locally for later issues.
