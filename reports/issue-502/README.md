# #502 — Scout UI cloud matrix

Second delivery in the #498 → #502 → #505 → #515 → #521 stack, based on
[PR #578](https://github.com/mr-exception/salimon/pull/578).
The real wgpu application renders cockpit/Core/Fabricator blackout and restored
states at **1280x800 and 1920x1080**, each at **simulated scales 1, 1.5 and 2**.
Physical Fabricator prompts, crafting exclusivity and pause are captured at all
six viewport/scale combinations. No UI or gameplay rule required a production fix.

## Actual coverage

- 36 original physical-screen GPU frames: three screens × blackout/restored ×
  two resolutions × three simulated scales. The `core-restart` fixture begins
  with no power and one existing 2 kg held fragment; production E Core intake
  removes its identity, and funded simulation ticks restore power. Reference
  cameras are explicit presentation overrides, not claims of physical travel.
- 18 GPU frames from actual player interaction positions: aimed physical
  Fabricator prompt, real crafting menu, and pause at each combination. Q and H
  cannot open competing overlays during crafting; stepping while paused preserves
  ship state. Captures themselves preserve ship/player/Fabricator/survival/machinery.
- Six additional original frames from the existing `world-prompts` route:
  cockpit/door/deposit anchors, changed camera, behind-camera hiding and tool-stowed
  context. This route passes through the production native window/protocol.
- Renderer tests: **15 instrument tests**, **one physical-screen orientation
  test**, **11 projection/layout tests**. These assert viewer-right glyph UVs,
  units and ellipsis for long/extreme readings, amber warning/cache isolation,
  blackout restoration, near/behind/offscreen hiding, scale/toolbar layout and
  uniform fitting. One opt-in CPU raster test remains intentionally ignored.
- Runtime: **10 action-bar tests** and **7 toolbar/modal/input tests** pass;
  one opt-in CPU evidence writer is ignored. Python tooling tests pass.

[All originals, full state, actual adapter/viewport/scale and input logs](evidence/)
are retained, with large JSON losslessly gzip compressed. Scale-1 Core captures
are in `502-scale1`, its successful Fabricator retry in `502-fabricator-scale1`;
scales 1.5 and 2 contain both scenarios. The prompt route is in `502-prompts`.
Per-run manifests pin source revision and binary SHA-256. The environment is the
same Linux x86_64 / Xvfb / Mesa 25.0.7 llvmpipe Vulkan setup documented in
[the preceding report](../issue-498/README.md). `WINIT_X11_SCALE_FACTOR` explicitly
simulates scale; every capture checks the actual reported value and drawable.
Fixed 96 ms protocol requests use production 16 ms ticks. Desktop input is isolated.

## Reproduction and results

Use a built `target/debug/salimon-client`, active X display and installed Vulkan
ICD. On this executor source `/workspace/.salimon-tools/env.sh` and set
`DISPLAY=:99`, `WGPU_BACKEND=vulkan`, and
`VK_DRIVER_FILES=/workspace/.salimon-tools/sysroot/usr/share/vulkan/icd.d/lvp_icd.json`.

```sh
python3 reports/issue-502/verify.py --binary target/debug/salimon-client --output artifacts/new-502-scale1 --scale 1
python3 reports/issue-502/verify.py --binary target/debug/salimon-client --output artifacts/new-502-scale1.5 --scale 1.5
python3 reports/issue-502/verify.py --binary target/debug/salimon-client --output artifacts/new-502-scale2 --scale 2
python3 scripts/salimon-test run scenarios/evidence/world-prompts.json --binary target/debug/salimon-client --artifacts artifacts/new-502-prompts --screenshot-command '["native"]'
cargo test -p salimon-renderer --locked cockpit_instruments
cargo test -p salimon-renderer --locked physical_screens
cargo test -p salimon-renderer --locked overlay
cargo test -p salimon-client --locked action_bar
cargo test -p salimon-client --locked toolbar_tests
python3 -m unittest discover -s scripts -p 'test_*.py'
python3 scripts/check_docs.py
```

[Raw check logs](.) and per-scenario results retain pass/fail status. The first
scale-1 driver incorrectly expected the intake target name `fabricator` after
aiming at the monitor; the production observation correctly returned
`fabricator_screen`. Its failure is retained in [scale1.log](scale1.log) and the
initial scenario result. The corrected Fabricator retry passes; the 12 earlier
Core captures remain valid. No target/range or interaction rule was changed.

## Remaining review and documentation impact

[#561](https://github.com/mr-exception/salimon/issues/561) owns per-image human
readability/accessibility: muted/amber text, glass/housings, text orientation,
units, long readings, anchored prompts/depth and Q/H/crafting/pause presentation.
Inspect originals, recording exact accepted/rejected images and hashes; no
baseline/mask/tolerance is promoted. Device DPI, physical input/focus and
platform-specific behavior remain [#557](https://github.com/mr-exception/salimon/issues/557).
Hardware CPU/render/GPU/frame-tail thresholds, Metal and hardware FPS remain
**unrun/unavailable** under [#563](https://github.com/mr-exception/salimon/issues/563).
Software frame diagnostics do not certify those budgets. Scanner remains #503.

Only a report-local driver, evidence and the validation guide change. The
[runtime architecture](../../client/runtime/architecture.md),
[runtime invariants](../../client/runtime/invariants.md),
[renderer architecture](../../client/renderer/architecture.md),
[renderer invariants](../../client/renderer/invariants.md) and
[Scout UI contract](../../docs/scout-presentation.md) retain unchanged ownership,
formatting, power and interaction semantics. Source/test mapping is given above.
Documentation, Python syntax and whitespace checks pass.
