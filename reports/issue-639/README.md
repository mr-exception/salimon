# Issue #639 — domain-owned E2E command handlers

Parent: `codex/issue-638-architecture-baseline` (PR #647), used provisionally
pending owner review/merge. Split operation parsing/execution into registered
survival, crafting, construction, lifecycle, clock, movement and input owners.
The common parser, native-thread reader, bounded deadlines, pause/modal guards
and response envelope remain central. Native operations, observations, aliases
and the unchanged scenario corpus have focused modules. Add domain operations
without changing a global dispatch match. No protocol change is intended.

Metrics using `python3 scripts/architecture_audit.py --history-ref 69c8394 --output artifacts/architecture/issue-639.json`:
shared automation file 1950 → 223 lines, branch proxies 162 → 27. The largest
production owner is inspection (238 lines); scenario tests remain a cohesive
1052-line corpus, not a new production hotspot. Historical overlap is pinned,
not claimed to have improved before future PRs exist.

Validation uses Rust 1.99.1, Python 3.12.14, Linux x86_64. Native dependencies
are unpacked under `/workspace/native-deps`, with pkg-config sysroot and library
search path; no dependency changes in Cargo.lock.
- `cargo test -p salimon-client --locked automation`: 31 passed.
- `cargo test --workspace --locked`: 1026 passed, 38 existing opt-in ignored.
- `cargo build --workspace --locked`: passed.
- `cargo fmt --all -- --check`: passed.
- `cargo clippy --workspace --all-targets --locked -- -D warnings`: passed.
- `python3 scripts/check_docs.py --base codex/issue-638-architecture-baseline`: 0 failures.
- `git diff --check`: passed.

The regression corpus covers real production command paths, pause/modal gates,
malformed requests, legacy aliases and bounded timeouts. A new cross-domain
negative test preserves numeric/string IDs and exact save bytes on argument
failures. Initial local setup failed for missing ALSA metadata/development-link
symlinks; those environment issues were corrected. Moving tests initially needed
relative include-path adjustments; corrected before the passing run.

Physical devices, macOS/Metal and hardware performance are unrun; their existing
owners remain #557/#561/#563. Native E2E smoke is optional in this issue; cloud
render checks exercise actual wgpu rather than claiming physical input coverage.

Fresh production wgpu/Xvfb/llvmpipe Vulkan route: 24/24 steps passed using
`python3 scripts/salimon-test run scenarios/evidence/lower-cockpit-windows.json --binary target/debug/salimon-client --artifacts artifacts/issue-639-fresh --screenshot-command '["native"]' --summary`.
Original [lossless result](native/result.json.gz), [protocol](native/protocol.jsonl) and captures
are retained beside this report. An earlier run used a pre-existing old binary;
its pass is excluded from change verification.
