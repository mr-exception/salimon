# Issue #641 — domain-owned save sections and version admission

Parent: `codex/issue-640-domain-input` (PR #649). Prerequisites are provisionally
available on this stack, pending owner review/merge. Runtime domain `save.rs`
adapters now own historical schema gates, reconstruction, Recovery/Fabricator
ownership checks, lighting context and carried-mount mass reconciliation. Physical
composition/tests and legacy propulsion migration have focused modules.

The serialized `SessionState` layout, field order, schema 23, strict unknown-field
policy, durable codec/storage and final single publication remain unchanged.
Central persistence is 833 → 250 lines, branch proxies 111 → 14 using
`python3 scripts/architecture_audit.py --history-ref 69c8394 --output artifacts/architecture/issue-641.json`.
New fields/sections still need deliberate typed schema composition; domain rule
changes normally edit the owner adapter. No generic save registry or flattening.

Validation: Linux x86_64, Rust 1.99.1, Python 3.12.14; local native libraries from
the earlier reports, no lockfile/dependency change.
- `cargo test --workspace --locked`: 1029 passed, 38 existing opt-in ignored, including real disk roundtrip/continuation, versions/migrations, corruption, rollback and restart fault injection.
- `cargo test -p salimon-client --locked persistence::contracts`: 2 passed. Fixture captured from parent `bd9a8c2` before extraction; exact new-run and restore/recapture bytes match. Unknown fields reject in all 16 sections. A mount reference passes DTO validation but fails late cross-domain reconstruction without publishing any live state.
- `cargo clippy --workspace --all-targets --locked -- -D warnings`: passed after moving the journal corpus to its own module (initial extraction triggered items-after-test-module lint).
- `cargo build --workspace --locked`: passed.
- `cargo fmt --all -- --check`, `git diff --check`: passed.
- `python3 scripts/check_docs.py --base codex/issue-640-domain-input`: 0 failures.
- `python3 scripts/salimon-test run scenarios/evidence/construction-offline-upgrade.json --binary target/debug/salimon-client --artifacts artifacts/issue-641-native --screenshot-command '["native"]' --summary`: production wgpu/Xvfb/llvmpipe Vulkan passed 14/14 steps, including actual checkpoint encode/restore, offline upgrade and commissioning.

Original [lossless result](native/result.json.gz), [protocol](native/protocol.jsonl)
and PNG captures are retained. Physical desktop restart/input, Metal and hardware
budgets remain unrun under #557/#561/#563; cloud disk tests and protocol-to-native
captures do not assert those passes. The latest source-age version-range cleanup
preserves the only currently admitted version (23); focused contracts, build,
Clippy and the native route were verified on that final source.
