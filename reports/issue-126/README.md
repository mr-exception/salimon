# Issue #126 — F physical grab/drop

Implemented on 2026-10-07 against main `71891ddd117c41a5eb82388994d88df19a74b52c`.

## Outcome

F grabs an aimed reachable fragment or drops the carried object. E operates
cockpit/exit-door interactions only. The existing world session still enforces
one carried world object; identity, material, mass and motion/transfer are unchanged.
A carrying action consumes the complete F press. Otherwise held F mines. Native
repeats/focus replay cannot grab/drop. Left mouse mining is independent, including
simultaneous input and release. Focus loss, Escape, view changes, suspension and
stowing clear mining input and its F latch.

Removed Q/G carrying routes. Automation `grab_drop` and legacy `pickup`, `drop`,
`mine` names share contextual F semantics. Carrying/transfer/resource-loop baseline
and evidence scenarios and prompts/control/maintenance documents are aligned.

## Validation

Environment: Linux x86_64, Rust 1.99.0, Python 3.12.14.

| Command | Result |
| --- | --- |
| `cargo fmt --all -- --check` | Passed |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | Passed |
| `cargo test --workspace --locked` | Passed; 64 runtime tests, existing renderer GPU test ignored by its normal gate |
| `cargo build --workspace --locked` | Passed |
| `python3 -m unittest discover -s scripts -p 'test_*.py'` | Passed; 34 tests |
| `git diff --check` | Passed |

Regression coverage exercises real carrying/mining/transfer/resource-loop actions,
E while targeting/carrying, F repeat suppression, pickup/drop with equipped tool,
no extraction from a consumed F press, mouse/F independence, clear/stow latch resets
and synthetic/repeated native key filtering. The carrying evidence scenario also
runs through CPU production controllers; it does not capture images in CPU tests.

## Limitations

Native graphical suites, screenshots, packaged OS-input smoke and reference
hardware performance were not run: this workspace lacks Xvfb and Vulkan drivers.
Installing prerequisites failed due to package-cache permissions. Existing Linux
CI must establish graphical/capture coverage; no GPU/screenshot result is claimed.
No asset/domain contracts changed, so Blender/model validation is not applicable.
