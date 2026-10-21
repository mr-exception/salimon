# Issue #381 — full gameplay pause

Delivered transient session-owned pause authority, Escape menu / Enter-or-Escape resume,
and explicit F1 cursor release. P/R/N remain engineering-tour controls. Paused ticks
are rejected in native foreground/background and portable automation; gameplay commands,
consumption, crafting and mouse look cannot mutate the paused world. Entry/resume clear
held controls/backlog/history while preserving requested ship throttle. Menu state is
not persisted. HUD, reticle, equipment and prompts are suppressed under pause.

Regression evidence covers a queued ration, full-save byte equality after 24 paused
hours, stale inputs, pending ticks, restore, tour separation, and a flying pilot through
background/focus resets and synthetic Escape replay. Native and portable pause commands
share production routing. #382 still owns Save Now / Save & Exit; #377 owns the wheel
and must use the shared pause/consumption guards.

Validation (Linux x86-64, Rust 1.99.1):
- `cargo test --workspace --locked`: 670 passed, 22 opt-in ignored.
- After final protocol routing: `cargo test -p salimon-client --locked`: 253 passed, 5 ignored.
- `cargo clippy --workspace --all-targets --locked -- -D warnings`: passed.
- `cargo build --workspace --locked`, `cargo fmt --all -- --check`, `git diff --check`: passed.
- `python -m unittest discover -s scripts -p 'test_*.py'`: passed (93 tests).

Native GUI launch cannot create a window in this display-less cloud session. Actual
OS input/cursor, rendered layout and M1/Metal signoff are unrun and remain tracked by
existing #376. Portable tests are not hardware acceptance. The pause menu currently
has keyboard Resume; accessible Settings is added by #383, verified saving by #382.
