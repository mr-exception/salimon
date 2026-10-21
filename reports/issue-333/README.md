# Issue #333 — aft door/ramp seam repair

Baseline: 7c8c0773b1f58768ef7868e8b6ddf58e48efbb20 (PR #445).

The closed source contained 81 positive-area coplanar triangle pairs between the
leaf, tray, header and stationary hull aperture. Previous quality exemptions
allowed them; mixed-material support surfaces also competed at full extension.
These are reproducible geometric causes of depth competition, rather than a UI
z-index problem. Repair the owning exit-door/hull Blender components with hidden
10 mm aperture/header clearance, tapered tray caps and single ownership of the
extended support seam. A 0.1 mm edge gap prevents float32 tessellation overlap.
The leaf owns the concealed overlap; there is no missing exposed ramp section.

No renderer depth bias or glass/material/scale changes. The 4.4 m ramp, hinge,
extension, markers, walking plane and colliders remain unchanged; generated
spatial/ramp/landing Rust files are byte-identical. Exported source/interchange
match: 6,493 triangles, 123 primitives, 13 materials, 549,668 bytes (within caps).

The new production export gate checks nine closed/opening/extension/reversal
samples: all targeted coplanar pairs and competing material pairs are zero.
[Structured samples](seam-samples.json). An intentionally duplicated tray fails.
Removed the four obsolete articulated overlap exemptions.

| View | Before | After |
| --- | --- | --- |
| Interior, oblique | [before](before/inside-oblique.jpg) | [after](after/inside-oblique.jpg) |
| Interior, straight | [before](before/inside-closed.jpg) | [after](after/inside-closed.jpg) |
| Opening | [before](before/opening.jpg) | [after](after/opening.jpg) |
| Extending | [before](before/extending.jpg) | [after](after/extending.jpg) |
| Deployed | [before](before/deployed.jpg) | [after](after/deployed.jpg) |
| Exterior | [before](before/outside-closed.jpg) | [after](after/outside-closed.jpg) |

These are actual Blender 4.5.3 CPU renders with matching cameras/lights and
runtime hinge/extension phases; static previews do not prove native temporal
fidelity. Actual OS traversal/animation reversal and Metal appearance remain
unrun in this display-less cloud session and are covered by existing #314
(door/ramp appearance/interpolation signoff). No native pass is claimed.

Validation:
- Scout export and category/quality validation: passed.
- Blender `verify_source.py` and `verify_modular.py`: passed, 9-component edit propagation.
- `BLENDER=... python -m unittest discover -s models/tests -v`: 97 tests, 2 optional skips.
- Character/renderer/ship Rust tests: 202 passed, 13 opt-in ignored.
- Workspace build, all-target Clippy, formatting and diff checks: passed.
- Reproduction: `repair_door_seams.py` is a deliberate one-time source migration;
  `preview_seams.py -- door <output>` is read-only. Normal builds need neither.
