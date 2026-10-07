# Issue #128 — Five-slot equipment toolbar state and input

## Outcome

Implemented runtime-owned portable equipment toolbar state with exactly five
fixed slots: mining tool in slot 1, empty slots 2–5. Initial selection is absent;
any slot may be selected, including empty slots, and selecting the same slot
again does not toggle it off.

Gameplay numeric keys 1–5 share native and automation input translation.
Successful physical pickup immediately clears selection. While carrying,
selection is refused; dropping never restores a previous selection. Precision
view keeps its existing 1–6 body inspection commands without changing equipment.
Native toolbar selection ignores releases, repeats and synthetic focus replay,
and requires captured cursor input like the existing tool keys.

`equipment.rs` owns loadout/selection without renderer dependencies or mining
behavior. Automation adds `slot_1`–`slot_5` and numeric aliases and exposes
`equipment.slots` and one-based `equipment.selected_slot` (null when absent).
The existing carrying baseline now asserts all slot selections, pickup clearing,
carrying lockout and explicit post-drop selection. Focused state, input,
composition and protocol tests cover those contracts, including precision view.
Runtime guides/invariants, maintenance map, root controls and runner docs updated.

## Validation

Environment: Linux x86_64, Rust/Cargo 1.99.0, Python 3.12.14.

- `cargo fmt --all -- --check` — passed.
- `cargo clippy --workspace --all-targets --locked -- -D warnings` — passed.
- `cargo test --workspace --locked` — passed (276 tests, one existing GPU test ignored), including the extended carrying
  scenario executed against production controllers without a native window.
- `cargo build --workspace --locked` — passed.
- `python -m unittest discover -s scripts -p 'test_*.py'` — passed (34 tests).
- `git diff --check` — passed.
- Native graphical baseline/packaged smoke — not run locally: no Xvfb/Vulkan
  runtime installed. Attempts to install the graphical dependencies failed on
  this environment's system package-cache permissions. CI provides these gates.
- Model/Blender validation — not run; no authored/generated asset changes.

## Scope and limitations

This issue is state/input only. There is no toolbar rendering change or new
visual evidence. The existing M mining equip/presentation control remains;
dependent toolbar and tool-presentation issues will consume the new state.
No inventory framework, persistence or automatic post-drop selection is added.
