# Issue #640 — domain-owned input bindings

Parent: `codex/issue-639-domain-automation` (PR #648). Its prerequisite baseline
and automation code are provisionally available, pending owner review/merge.
Native `app/input.rs` now translates physical events into portable typed keys.
Construction, lighting, ship, equipment, interior, Archive, wheel and crafting
handlers own their bindings and guards. Explicit modal priority, centralized
input reset and native focus/capture/repeat/synthetic-event admission are preserved.
No winit type crosses into the portable session input modules.

Audit command: `python3 scripts/architecture_audit.py --history-ref 69c8394 --output artifacts/architecture/issue-640.json`.
Native input file 672 → 467 lines and branch proxies 103 → 34; portable router
104 lines/14 branch tokens. Existing native policy tests remain with the adapter.
Domain-specific behavior can change in its owner; new physical keys still need
an explicit native mapping. No speculative event bus or change to action authority.

Validation on Linux x86_64, Rust 1.99.1, Python 3.12.14:
- `cargo test -p salimon-client --locked app::input`: 22 passed.
- `cargo test -p salimon-client --locked automation`: 31 passed.
- `cargo test -p salimon-client --locked session::input`: 1 passed; wheel blocks construction, opening the wheel clears movement, construction Escape precedes pause, resume does not replay holds.
- `cargo test --workspace --locked`: 1027 passed, 38 existing opt-in ignored.
- `cargo fmt --all -- --check`: passed.
- `git diff --check`: passed.
- `cargo clippy --workspace --all-targets --locked -- -D warnings`: passed.
- `cargo build --workspace --locked`: passed.
- `python3 scripts/check_docs.py --base codex/issue-639-domain-automation`: 0 failures.
- `python3 scripts/salimon-test run scenarios/evidence/construction-interlocks-cockpit.json --binary target/debug/salimon-client --artifacts artifacts/issue-640-native --screenshot-command '["native"]' --summary`: 5/5 steps passed using production wgpu/Xvfb/llvmpipe Vulkan.

Original [lossless result](native/result.json.gz), [protocol](native/protocol.jsonl)
and image are retained. This is protocol-to-native admission evidence, not actual
keyboard/mouse qualification. Physical input/focus/device and native restart
review remain #557; subjective imagery #561, hardware/Metal metrics #563.
Initial compile/import and Clippy manual-map diagnostics during extraction were
corrected before passing checks. Runtime/save behavior is intended unchanged.
