# Character AI Maintenance

Read `architecture.md` and `invariants.md` before edits. Keep this crate portable:
no `winit`, `wgpu`, renderer, diagnostics, or ship-crate dependency. Native input
translation and domain composition belong to `client/runtime`.

Important regression areas are diagonal-speed normalization, edge-triggered
jumping, closed/flying doorway containment, the exact 250 ms gravity blend,
camera-relative W/S/A/D doorway crossings and direction-based surface re-entry,
instant cockpit transitions, horizontal mouse-delta direction in both walking
and cockpit views, the default forward cockpit-window sightline, central Core
and furniture collision and sliding, both cockpit side routes, pilot-chair and
console/monitor collision, forward-hull containment, window sill clearance,
ceiling and lintel containment, rear doorway body clearance, safe cockpit exit,
exterior hull containment with the gate open or closed, corner sliding and gate
approach from outside, vertical separation from the hull,
door closure during a blend or after a stopped exit, repeated open/close
crossings at the center and both jambs, closed-gate contact in rotated frames at
the catalog's 1e12 m anchor,
and constant-radius full-sphere movement. Add a focused unit test whenever one
of these rules changes. Surface camera and movement direction must continue to
share the same yaw-derived local tangent basis rather than using fixed ship/world
axes.
