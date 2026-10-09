# Issue #131 — carrying and equipment exclusivity

## Outcome

Prerequisites #126, #128 and #130 are merged on the inspected base (`2ccab58`).
Those changes already implement successful-pickup deselection, five-slot lockout,
stowing and no automatic restoration. This PR completes the missing failure,
presentation and resource-loop regression coverage without adding another equip
state or changing the permanent one-world-object rule.

- Failed pickup regression covers every selected slot and preserves mining input.
- Equipped pickup checks deselection, cleared mouse mining, hidden held-item DTO,
  and unselected toolbar presentation. All five slot keys are ignored while held.
- Drop stays deselected; selecting an empty slot and then the mining tool works
  immediately without restarting a prior mining hold.
- Redraw and automation share `ClientApplication::held_item`; the additive
  `mining.held_item_visible` inspection field observes the actual DTO gates.
- Carrying and resource-loop baseline/evidence scenarios verify F misses before
  extraction, equipped pickup, lockout, drop and explicit re-selection. The
  resource loop now grabs while equipped rather than starting from an empty slot.
- Runtime, character, resource and automation guides document the shared contract.

## Validation

Environment: Ubuntu 24.04 x86_64, Rust/Cargo 1.99.0 (stable).

| Command | Result |
| --- | --- |
| `cargo fmt --all -- --check` | Passed |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | Passed |
| `cargo test --workspace --locked` | Passed: 282 tests, including 74 runtime tests |
| `python -m unittest discover -s scripts -p 'test_*.py'` | Passed: 34 tests |
| `git diff --check` | Passed |

| `cargo build --workspace --locked` | Passed (debug) |
| `python scripts/build_game.py --profile debug --output artifacts/build/debug` | Passed |
| Native evidence carrying scenario | Passed: 263 steps |
| Native evidence resource-loop scenario | Passed: 249 steps |

The native evidence runs cover all baseline actions/assertions plus screenshots.
Commands used a TCP Xvfb display because Unix-socket connections are unavailable
in this execution environment:

```sh
export WGPU_BACKEND=vulkan VK_DRIVER_FILES=/usr/share/vulkan/icd.d/lvp_icd.json
Xvfb :98 -screen 0 1280x800x24 -listen tcp -ac
# In the same execution session with DISPLAY=127.0.0.1:98:
python scripts/salimon-test run scenarios/evidence/carrying.json --binary artifacts/build/debug/salimon-client --screenshot-command '["python", "scripts/capture_settled.py", "{path}"]'
# Resource-loop used display :97 with the same Xvfb settings:
python scripts/salimon-test run scenarios/evidence/resource-loop.json --binary artifacts/build/debug/salimon-client --screenshot-command '["python", "scripts/capture_settled.py", "{path}"]'
```

[Selected native assertions and results](native-checks.json) preserve the
selection, carrying and mining checks without unrelated world snapshots.
Screenshots were inspected at 1280×800 with Mesa software Vulkan:

- [Carrying: five slots, no highlight, no held mining tool](carrying-toolbar-deselected.png).
- [After drop: still deselected](dropped-toolbar-deselected.png).
- [Explicit slot 1 re-selection: held tool restored](mining-tool-reselected.png).
- [Cabin delivery/drop: toolbar remains deselected](cabin-drop-toolbar-deselected.png).

## Limitations

`python scripts/build_game.py --profile release --output artifacts/build/release`
failed locally in third-party `naga`/`wgpu-naga-bridge`: corrupt metadata, then
zero-length object files after `cargo clean -p naga --release --target
x86_64-unknown-linux-gnu` and a retry with `CARGO_BUILD_JOBS=2`. No release binary
was produced. Initial default Unix-socket Xvfb launches failed before readiness;
the TCP display runs above passed. Debug native validation does not establish
release, macOS/Windows or reference-hardware performance. CI must confirm the
standard cross-platform build gates. Model authoring/export and packaged-input
smoke were not run; no assets, native key mapping or lifecycle behavior changed.
