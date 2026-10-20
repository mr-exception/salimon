# Issue #334 — hull/canopy corner junction

Baseline: b6e8c3cfc853c31bab50d0b5c4a705a8a8370db2 (PR #446).

The opaque shoulders extended to X=5.20 m while the tapered glass begins at
X=5.00 m; their forward posts crossed the near-corner visibility rays. The roof
also extended to X=5.16 m. Trim both side ends and roof symmetrically to X=4.98 m,
inside the existing rear window frame (4.94..5.06 m). Reuse the existing end caps
and discard collapsed/duplicate faces rather than adding clipping tessellation.
The original option-A planes/materials remain; no glass opacity or depth workaround.

Only the owning hull Blender library changed. Cockpit/glass sources, narrow
frames, lower glazing, nose, console/monitor and traversal sources are unchanged.
Reviewed collision proxies remain valid conservative pane/corridor and broad-hull
envelopes; no external extent, collision layout, markers, ramp or support changes.
Generated spatial/ramp/landing Rust contracts are byte-identical to the parent.
GLB/interchange/export hashes are regenerated: 6,493 triangles, 123 primitives,
13 materials, 549,732 bytes, all within existing caps.

Added 24 real triangle ray checks from the actual seated eye and standing behind
the chair to both near corners. Existing forward/lower-window/monitor checks pass.
The parent export fails the new corner check; intentionally extending the current
port shoulder to 5.2 m also fails. [Machine-readable witness](corner-evidence.json).

| View | Before | After |
| --- | --- | --- |
| Seated, side -1 | [before](before/seated--1.jpg) | [after](after/seated--1.jpg) |
| Seated, side +1 | [before](before/seated-1.jpg) | [after](after/seated-1.jpg) |
| Standing, side -1 | [before](before/standing--1.jpg) | [after](after/standing--1.jpg) |
| Standing, side +1 | [before](before/standing-1.jpg) | [after](after/standing-1.jpg) |
| Exterior, side -1 | [before](before/exterior--1.jpg) | [after](after/exterior--1.jpg) |
| Exterior, side +1 | [before](before/exterior-1.jpg) | [after](after/exterior-1.jpg) |

Actual Blender 4.5.3 CPU captures use matching cameras and lighting. They establish
source appearance, not Metal/native interaction or in-flight visual signoff;
existing #314 owns native ship/glazing review. That verification is unrun in this
display-less session; geometry rays are independent of ship world pose.

Validation:
- Scout export/category/quality and Blender source parity/modular propagation: passed.
- `BLENDER=... python -m unittest discover -s models/tests -v`: 99 tests, 2 optional skips.
- `cargo test --workspace --locked`: 675 passed, 22 opt-in ignored.
- Workspace build, all-target Clippy, formatting and diff checks: passed.
- Source migration is `repair_cockpit_shoulders.py` (one-time only); read-only
  previews use `preview_seams.py -- corners <output>`.
