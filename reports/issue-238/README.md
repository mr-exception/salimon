# #238 — smaller action/interaction prompts

Reduced the existing shared action-bar bitmap scale from 4–8 to 3–6. At 1× UI
scale, seven-cell glyphs are 21 pixels high instead of 28 (25% smaller);
2× uses 42 instead of 56. At 1.5×, integer rounding gives density 5 instead of 6.
Integer bitmap replication keeps hard glyph edges, high-contrast colors and
padding. No separate E/L string-specific styling was introduced: interaction,
landing/takeoff, mining/carrying guidance, Core actions and blocked-action
transients already use this owner and all inherit the smaller type.

Input labels, interaction/visibility gates, world anchors, screen placement,
three-second transient priority/expiry, toolbar stacking and uniform viewport
fitting are unchanged. Cockpit/Core monitors, diagnostics, other HUD text and
the reticle have independent raster owners and are unaffected by this change.

## Checks

Linux x86_64, Rust 1.99.0, Python 3.12.14. From repository root:

- `cargo test -p salimon-client --locked action_bar::tests`: passed, 6 tests.
  Verifies actual readable glyph pixels at 1×/1.5×/2×, image dimensions/buffer
  size, unchanged-cache revisions, identical contextual/transient typography,
  multiline row placement and dynamic text, finite/bounded DPI handling,
  contextual visibility and exact three-second override/restore behavior.
- `cargo fmt --all -- --check`: passed.
- `cargo clippy --workspace --all-targets --locked -- -D warnings`: passed.
- `cargo test --workspace --locked`: passed on the complete three-issue stack,
  including runtime, renderer overlay/placement, domain and automation tests.
- `cargo build --workspace --locked`: passed.
- `python -m unittest discover -s scripts -p 'test_*.py'`: passed, 58 tests.
  An expected timeout/error fixture prints a failed report; the suite is green.
- `git diff --check`: passed.

## Visual verification

Compare `Press E to use` at the cockpit and `Press L to land` / takeoff guidance;
trigger a blocked-door transient and a two-line deposit/fragment prompt. Text
should be smaller with the same content, colors, anchors and interaction rules.
Verify at 800×600, 1280×800 and 1920×1080 with 1×, 1.5× and 2× OS scaling. Check
that world labels remain centered at their anchors, fit within the existing
60%-viewport-width cap, hide behind occluders, and do not intersect the toolbar.
Long bottom messages retain uniform fitting/stacking; tiny unfittable prompts
retain the renderer's existing hide policy. Switch F2 to verify gameplay text
hides; cockpit displays and F3 diagnostics should retain their original sizes.

Live graphical captures/visual review were not run here (no display/capture
stack). CPU checks do not certify on-screen readability or native GPU output.
No Apple M1 verification is required to close this cloud-capable issue.
