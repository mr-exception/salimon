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
