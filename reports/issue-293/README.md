# #293 — compact contextual prompts

Reduced the shared action-bar density from 3–6 pixels per glyph cell to 2–3.
The existing DPI policy still rounds to integer pixels but caps growth at 3,
keeping high-DPI prompts compact without filtering the bitmap font. This applies
to contextual actions, flight guidance and transient feedback, including E/L,
mining, carrying and Core interaction messages.

| OS DPI | Glyph height, before → after | Single-line panel height, before → after | Width/height reduction before viewport fitting |
| --- | --- | --- | --- |
| 1× | 21 → 14 px | 39 → 26 px | 33% |
| 1.5× | 35 → 21 px | 65 → 39 px | 40% |
| 2× | 42 → 21 px | 78 → 39 px | 50% |

For example, `Press L to land` now occupies 198×26 physical pixels at 1×
and 297×39 at 1.5×/2×. The glyphs, proportional padding, colors and border remain
intact. Text selection, object anchors, occlusion, viewport fitting, toolbar
stacking, transient priority and three-second expiry use their existing paths.
Cockpit/Core display and other HUD raster owners were not changed.

## Visual evidence

These images compare actual production CPU rasters against main revision
`56dc800` (2026-10-18), with no image resampling. Each sheet includes landing,
cockpit use and two-line mining prompts. Background alpha is shown opaque.
The panels visibly shrink while retaining complete glyphs and padding.
They are raster evidence, **not game screenshots**; viewport fitting is not
applied, so the widest old mining panel would be reduced further by the renderer.

- [1× DPI comparison](dpi-1.0.png)
- [1.5× DPI comparison](dpi-1.5.png)
- [2× DPI comparison](dpi-2.0.png)

Reproduce with `python reports/issue-293/render_comparison.py` from the repository
root (Rust on PATH and Pillow required). The script compiles the old/current
production module with a temporary output harness; it does not reimplement the
font or sizing policy.

## Validation

Linux x86_64; Rust 1.99.1, Python 3.12.14.

- `rustc --edition=2024 --test client/runtime/src/action_bar.rs -o /tmp/salimon-293-action-tests`
  and `/tmp/salimon-293-action-tests`: passed, 8 tests. Standalone compilation
  warns that `has_transient` has no caller; the normal runtime uses that method.
  Covers dimensions, DPI bounds, contextual/transient parity, multiline updates,
  expiry/visibility, at least one-third size reduction, complete integer raster
  blocks and cache reuse/rebuild when changing DPI in both directions.
- `python reports/issue-293/render_comparison.py`: passed; all three comparison
  sheets generated from production rasters and visually inspected.
- `cargo test --workspace --locked`: passed, 490 tests including doc tests;
  16 existing opt-in tests ignored. Includes runtime interaction/anchor and
  renderer overlay projection, viewport fitting and toolbar stacking regressions.
- `cargo clippy --workspace --all-targets --locked -- -D warnings`: passed.
- `cargo build --workspace --locked`: passed.
- `cargo fmt --all -- --check`: passed.
- `git diff --check`: passed.

## Limits

Live graphical smoke/capture and Apple M1 signoff were not run: this environment
has no graphical display, Xvfb or Vulkan driver. CPU raster tests/evidence do not
certify native readability at viewing distance or GPU composition. Issue #293
explicitly does not require native M1 signoff for cloud completion. Projection,
scene-depth visibility and toolbar fitting are unchanged and retain their
existing renderer regressions.
