# Issue #642 — explicit session domain composition

Parent: `codex/issue-641-domain-save-adapters` (PR #650). Its baseline, E2E,
input and save prerequisites are provisionally usable on the same ancestry;
review/merge remains the owner's responsibility.

`session.rs` retains typed contracts and explicit domain fields. Empty/default
and new-run initialization move to `init.rs`; command admission/delegation moves
to `commands.rs`; interaction details move to `interaction/actions.rs`. Held
movement/steering are owned by input state; outside observers receive copies.
Nonserialized host stepping/interpolation/measurement policy is one private-field
value, copied consistently through restore, offline reconciliation and restart.

The 79-line `tick.rs` scheduler calls focused power/character/resource adapters
and presentation history helpers in explicit order. Survival admits the actual
duration before presentation; age/camera/steering, Fabricator/ship, crash handling,
Recovery and construction precede completed-frame character movement and mining/
loose physics. Journey, carrying/mass, focus, terminal state and render history
finalize in the original sequence. No unordered callbacks, hidden registry,
new domain dependency or gameplay/save contract change.

Audit: `python3 scripts/architecture_audit.py --history-ref 69c8394 --output artifacts/architecture/issue-642.json`.
Central session 967 → 375 lines, branch proxies 115 → 25. Commands are 169 lines/
39 branch tokens, interactions 152/17 and scheduler 79/6. Compare all owner files;
this reduces shared edit surfaces, not total domain complexity. Explicit domain
fields and new shared contracts still require composition review; future measured
PR overlap can establish the actual conflict reduction.

Validation environment: Linux x86_64, Rust 1.99.1, Python 3.12.14; local unpacked
native libraries and Xvfb/llvmpipe Vulkan as in preceding reports.
- `cargo test -p salimon-client --locked session::tick`: 2 passed. A construction save captured from parent `8c13f72` before extraction matches byte-for-byte after 12 seconds, both whole and partitioned admission. Restore preserves a distinct host policy, clears held input and produces the same authoritative source as ordinary launch.
- Existing schema-23 new-run byte golden remains in the full regression suite.
- `cargo clippy --workspace --all-targets --locked -- -D warnings`: passed.
- `cargo build --workspace --locked`: passed.

Initial extraction needed scoped-import/doc-comment corrections; those were fixed
before passing checks. The first host-policy test incorrectly expected launch
reconciliation to preserve pre-launch metadata bytes. Its final comparison uses
an ordinary launch of the same save, preserving the existing reconciliation
contract instead of changing production behavior.

Physical desktop/OS input, native Metal and hardware performance remain unrun.
Existing #557 owns physical integration checks, #561 subjective image/audio review
and #563 hardware budgets. Cloud evidence does not turn those into passes.


Final validation:
- `cargo test --workspace --locked`: 1031 passed, 0 failed, 38 existing opt-in ignored.
- `cargo fmt --all -- --check`, `git diff --check`: passed.
- `python3 scripts/check_docs.py --base codex/issue-641-domain-save-adapters`: 0 failures.
- `python3 scripts/salimon-test run scenarios/evidence/construction-interlocks.json --binary target/debug/salimon-client --artifacts artifacts/issue-642-construction --screenshot-command '["native"]' --summary`: 21/21 production wgpu/Xvfb/llvmpipe Vulkan steps passed.
- `python3 scripts/salimon-test run scenarios/evidence/lower-cockpit-windows.json --binary target/debug/salimon-client --artifacts artifacts/issue-642-flight --screenshot-command '["native"]' --summary`: 24/24 steps passed.

Original construction [result](native-construction/result.json.gz),
[protocol](native-construction/protocol.jsonl) and captures; flight
[result](native-flight/result.json.gz), [protocol](native-flight/protocol.jsonl)
and captures are retained losslessly. SHA-256 byte comparison of all eight flight
PNGs against issue #639's retained route captures is identical (8/8). This proves
that route's captured output remained unchanged, not unobserved hardware behavior.

The first four stack PRs (#647–#650) have successful Ubuntu source build/tests
and macOS/Windows fiction golden checks at their recorded branch heads. Review
and merge in order; no merge or default-branch push was performed.
