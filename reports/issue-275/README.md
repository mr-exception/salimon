# Issue 275 — offline advisory style review

Added optional category-aware `styleProfile` schema, deterministic normal/micro
geometry/reuse/LOD measurements and reasoned exceptions. Reuses #227 costs and
existing generic/category validation. Existing assets remain unmodified and
valid until explicit opt-in. Hard budgets and spatial contracts are unchanged.
The existing processing report includes opt-in style before/after sections.
Subjective shape, color and reference acceptance remain human review.

[Sample same-asset comparison](sample-comparison.json) records actual tool costs
and intentional zero deltas; #276 supplies a real production before/after report.
[Commands and definitions](../../models/tools/README.md#offline-style-review-275).

Validation: Python 3.12, pinned jsonschema 4.26.0. Six focused deterministic tests
pass (opt-in, normals/missing normals, micro geometry, reuse, LOD, comparison,
schema rejection, unchanged hard budgets and processing-report integration).
`python models/tools/style_review.py item.mining-tool --output artifacts/style/tool.json`
passes with existing non-opted-in manifest. `git diff --check` passes.
No native shader/M1 aesthetic claim. Triangle areas are mesh-local measurements,
not projected screen-size inference; profile thresholds are review proposals.

Full model suite: `PATH=/root/.cargo/bin:$PATH BLENDER=/tmp/blender-4.5.3-linux-x64/blender python -m unittest discover -s models/tests -v`:
**92 tests passed**, including real Blender export/source checks and installed
Khronos checks. Initial prerequisite run failed while rustfmt was installing;
rerun after installation passed. No assets changed in this issue.
