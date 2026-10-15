# #226 — meter-space quality gate

Generic staged export/validation now enforces declared quality policies, separate
from visual budgets. All 21 production assets declare bounds/anchor tolerances;
scout retains its independent authoritative category checks. Added cumulative
transform triangle decoding, winding/normal checks, degeneracy, duplicate faces,
optional welded manifold checks, conservative clearance boxes and accelerated
near-coplanar positive-area overlap diagnostics. Flat shading/open panels remain
valid. Sorted JSON includes object, primitive/triangle, location and tolerance.

[Asset summary](asset-summary.json) and [full scout diagnostics](scout-quality.json)
record measured outcomes. All resources pass without overlap exceptions. Existing
scout/container/tool overlap candidates are named, counted temporary exceptions;
mining-tool's 32 degenerate faces are capped warnings. Remediation is explicitly
tracked in [#244](https://github.com/mr-exception/salimon/issues/244). New pairs,
excess pair counts or excess degenerates fail. No geometry, dimensions, budgets
or generated runtime assets were silently changed.

## Validation

Linux x86_64, Python 3.12, jsonschema 4.26.0.

- `python -m unittest discover -s models/tests -v`: passed 73 tests, three
  explicit Blender skips (before two additional focused tests).
- `python -m unittest discover -s models/tests -p test_geometry_quality.py -v`:
  seven focused tests pass, including the actual generic validation failure gate.
- `python models/tools/validate_quality.py ship.salimon-scout`: passes with
  counted existing exceptions; full output above.
- `python models/assets/ships/salimon-scout/validate.py`: passes unchanged budgets,
  source/export provenance, pivots/colliders, monitor and spatial contracts.

Fixtures exercise malformed faces, plane-boundary overlap, shared-edge control,
world-parent transforms, bounds/anchors/clearance, closed topology, counted
exceptions, and an Option A/Option B tessellation swap retaining collider/marker
interfaces. 2,000 separated triangles yield zero broad-phase candidates.

## Limits

Candidates do not prove visual flicker. Sweep-and-prune still has quadratic dense
worst cases. Clearance checks conservatively use per-object AABBs; concave aisle
obstacles need explicit category geometry. Non-manifold checks apply only to
nodes declared closed; glTF flat-shaded vertex splits are welded geometrically.
No full self-intersection or native M1 visual claim. #244 must classify/remove
existing exceptions; #229 owns native visual signoff.
