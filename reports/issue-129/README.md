# Issue #129 — Equipment toolbar HUD

Implemented against main `24a9700` on 2026-10-07. The toolbar state/input blocker
(#128) is merged. This change renders its existing five-slot loadout/selection;
individual mining-tool equip integration remains separate work.

## Outcome

- Five compact numbered slots remain visible at bottom center during gameplay.
- Slot 1 has a drill silhouette; slots 2–5 have empty interiors.
- Runtime selection maps directly to a cyan border/background; absent selection
  highlights nothing. Renderer receives typed presentation data, never domains.
- Renderer owns rasterization and caches pixels/uploads by state and DPI density.
  Native scale factors are bounded to 1x–2x, consistent with existing HUD handling.
- Precision tour supplies no toolbar. Resize uniformly fits the current drawable.
- Global/transient action messages reserve the fitted toolbar height plus a
  scaled gap. Tiny viewports hide messages that cannot fit rather than overlap.
- CPU tests cover DTO mapping, all five selection mappings, carrying clear,
  raster icon/empty slots/numbers/highlights, cache behavior, DPI, and joint layout.
- Renderer/runtime guides, invariants, architecture and maintenance map updated.

## Native evidence

Captured from the staged release executable with Linux software Vulkan. Images
were visually inspected. The E2E scenario asserts selected slot 1/2/5 and initial
absence; the native layout harness repeats these assertions and captures clean
window images. `capture_layout.py` also uses OS F2 and resize plus forced native
2x DPI, and exercises normal flight/blocked-door messages.

| State | Evidence |
| --- | --- |
| No selection | [Screenshot](no-selection.png) |
| Mining-tool slot selected | [Screenshot](selected-slot-1.png) |
| Empty slot 2 selected | [Screenshot](selected-slot-2.png) |
| Empty slot 5 selected | [Screenshot](selected-slot-5.png) |
| Global message stacked | [Screenshot](global-message-stacking.png) |
| Transient blocked-door feedback stacked | [Screenshot](transient-message-stacking.png) |
| Precision tour hides toolbar | [Screenshot](precision-tour-hidden.png) |
| Resized 800×600 | [Screenshot](resize-800x600.png) |
| Native 2x DPI | [Screenshot](dpi-2.png) |

## Validation

Environment: Linux x86_64, Cargo 1.99.0, Python 3.12.14, Mesa lavapipe Vulkan,
Xvfb dedicated 1920×1080 display. Root workspace commands:

| Command | Result |
| --- | --- |
| `cargo fmt --all -- --check` | Passed |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | Passed |
| `cargo test --workspace --locked` | Passed |
| `cargo build --workspace --locked` | Passed |
| `python3 -m unittest discover -s scripts -p 'test_*.py'` | Passed, 34 tests |
| `python3 -m unittest discover -s models/tests -v` | Passed, 53 tests; 2 Blender-only skips |
| `python3 scripts/build_game.py --profile release --output artifacts/build/release` | Passed |
| `python3 scripts/salimon-test suite --binary artifacts/build/release/salimon-client` | Passed, 13 baseline scenarios |
| `python3 scripts/salimon-test run scenarios/evidence/equipment-toolbar.json --binary artifacts/build/release/salimon-client --screenshot-command '["python3", "scripts/capture_settled.py", "{path}"]'` | Passed, 14 steps and four captures |
| `python3 scripts/salimon-test run scenarios/evidence/lower-cockpit-windows.json --binary artifacts/build/release/salimon-client --screenshot-command '["python3", "scripts/capture_settled.py", "{path}"]'` | Passed |
| `python3 scripts/salimon-test suite --group resource-collection --evidence --binary artifacts/build/release/salimon-client --screenshot-command '["python3", "scripts/capture_settled.py", "{path}"]'` | Passed, 6 scenarios |
| `python3 scripts/salimon-test suite --group ship-eva --evidence --binary artifacts/build/release/salimon-client --screenshot-command '["python3", "scripts/capture_settled.py", "{path}"]'` | Passed, 4 scenarios |
| `python3 scripts/packaged_smoke.py --binary artifacts/build/release/salimon-client --timeout 90` | Passed, OS F2 round-trip and rendering |
| `python3 reports/issue-129/capture_layout.py` | Passed; selection, layout and visibility screenshots inspected |
| `git diff --check` | Passed |

For graphical commands set `WGPU_BACKEND=vulkan`,
`VK_DRIVER_FILES=/usr/share/vulkan/icd.d/lvp_icd.json` and `DISPLAY` to a running
dedicated X11 display. This environment disallows local Unix socket connections;
initial Unix-display attempts failed before game startup. Successful runs used
`Xvfb :101 -screen 0 1920x1080x24 -listen tcp -ac` and
`DISPLAY=127.0.0.1:101` (layout harness on :103), with a readiness probe before
launch. Use TCP/-ac only on an isolated local test display. Initial model tests
failed before installing `models/tools/requirements.txt`; all passed afterward.

[Structured graphical results](validation-results.json), [Rust tests](tests.log),
[Clippy](clippy.log), [tooling tests](python.log), [model checks](model.log).
Software Vulkan does not establish macOS/Windows GPU fidelity or reference M1
performance. Native macOS/Windows validation and real Blender export were not
run; no model source or export changed.
