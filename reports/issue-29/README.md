# Issue #29 validation

Validated on 2026-09-30 on Linux with a dedicated 1280×800 Xvfb display and
software Vulkan. The worker completed this task in the current session; the
issue's GPT-6 Astra/Ultra suggestion could not be selected in-place.

- Python runner contract tests: 15 passed, including CLI exit propagation,
  full logs beyond the stderr tail, state/actual-value evidence, unique run
  directories, parse/launch failures, capture timeout/nonzero/invalid-output
  failures, unavailable capture, and capture while the game process is alive.
- `cargo build --workspace --locked`: passed with Rust 1.98.1.
- Rust 1.95.0 `cargo fmt --all -- --check`,
  `cargo clippy --workspace --all-targets --locked -- -D warnings`, and
  `cargo test --workspace --locked`: passed.
- Existing native suite: landed-earth and orbit-earth passed.
- Native `scenarios/evidence/landed-earth.json`: all nine steps passed;
  named PNG checkpoints captured before and after walking. Both images were
  inspected and contained the rendered cabin; the views differ after movement.
- A temporary copy with an intentionally incorrect door assertion failed at
  step six, reported actual `Closed` versus expected `IntentionallyWrong`,
  captured a rendered failure PNG before termination, preserved state/logs,
  and returned the failure result. The deliberately failing case is not part
  of the normal suite.

[Machine-readable validation summary](validation.json),
[native checkpoint image](landed-cabin.png), and
[automatic failure image](landed-cabin.png) record the acceptance evidence.
Run commands and artifact inspection instructions are in
[the runner documentation](../../scripts/README.md).

Rust 1.98.1's strict Clippy run exposed an existing
`clippy::chunks_exact_to_as_chunks` warning at
`client/diagnostics/src/lib.rs:592`. Rust 1.95.0's strict Clippy check passes;
this task does not modify Rust gameplay/renderer code or hide the new lint.
Native macOS/Windows capture helpers have not been exercised here. Desktop
capture requires an unobscured game display; images represent the last
presented frame, while assertions use authoritative inspection state. These
runs validate evidence generation and gameplay contracts, not hardware GPU
performance or the macOS manual-playtest checklist.

Documentation cleanup (#98): the originally checked-in failure and cabin PNGs
were byte-identical. Both links now resolve to the retained cabin image; this
does not establish a distinct failure view. Historical assertions remain unchanged.
