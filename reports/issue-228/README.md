# Issue 228 — portable production model visual regression

Added production GPU capture fixtures (static repeats, moving camera, multi-angle
cockpit/glass/exterior, floor/ramp, monitor/Core, tool/resources and contact piles),
offline baseline comparison, per-checkpoint temporal comparison, bounded reviewed
masks/tolerances, actionable PNG diffs and JSON pixel metrics. Uses existing
fixed-step automation and direct pre-presentation wgpu capture. CI preserves
candidates, diffs and production protocol/state logs.

Validation: `python -m unittest discover -s scripts -p 'test_*.py'`: 61 passed.
Injected single-pixel regression fails with a highlight; masked/threshold-bounded
noise passes; moving views match corresponding baselines while static flicker
fails; absent approval fails. Python 3.12 / Pillow 12.3.0.

See [fixture workflow and approval policy](../../scenarios/visual/README.md).
Approved production aesthetic baselines cannot be invented by automated tooling:
current output is review candidates, not approved Option A art. Native/M1 visual
approval remains #229. Graphics and Rust execution were initially unavailable in
this workspace; CI executes software-Vulkan capture and repeat checks. No native
hardware coverage or performance equivalence is claimed. Option A/B replacement
comparison shares fixtures and spatial assertions; no full B asset redesign.

## Follow-up — complete fixture evidence

The original implementation is merged in PR #250. A review of the current
temporal gate found that a partial sequence could pass: removing a complete
camera pose, or one repeat while leaving two, did not fail `--temporal-only`.
Capture and comparison now share scenario expansion and require the exact
configured PNG set before comparing pixels. Diagnostics list missing/unexpected
files. Duplicate/unsafe checkpoint names, unsupported modes and fixtures without
enough repeats fail explicitly. No runtime, asset or pixel threshold changed.

Validation on Linux, Python 3.12.14 and Pillow 12.3.0:

- `python -m unittest discover -s scripts -p 'test_*.py'`: 78 tests passed.
  Regressions cover missing repeats, missing camera poses, stale PNGs, invalid
  definitions and all nine production fixture contracts; existing injected visual
  change, noise/mask, stable and flicker checks continue to pass.
- `git diff --check`: passed.

No graphics or Rust execution was run for this Python tooling change. These
offline tests validate evidence completeness and pixel-diff behavior, not actual
production appearance. Production image approval and native/M1 coverage remain
pending as described above and in #229; this follow-up does not close #228.
