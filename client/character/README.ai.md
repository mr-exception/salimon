# Character AI Maintenance

Read `architecture.md` and `invariants.md` before edits. Keep this crate portable:
no `winit`, `wgpu`, renderer, diagnostics, or ship-crate dependency. Native input
translation and domain composition belong to `client/runtime`.

Important regression areas are diagonal-speed normalization, edge-triggered
jumping, closed/flying doorway containment, the exact 250 ms gravity blend,
camera-relative W/S/A/D doorway crossings and direction-based surface re-entry,
instant cockpit transitions, horizontal mouse-delta direction in both walking
and cockpit views, the default forward cockpit-window sightline, central Core
and Core collision and sliding, the port cockpit route into the nose,
pilot-chair and unified console/monitor collision, forward-hull containment,
window sill clearance,
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

## Shared maintenance rules

Follow the root [coding conventions](../../docs/coding-conventions.md),
[validation matrix](../../docs/validation.md) and
[feature map](../../docs/maintenance-map.md). Update affected contracts/guides
with behavior changes and record completion evidence under root `reports/`.

## Source map

`src/lib.rs` preserves the public re-exports and constants. Read
`src/controller/mod.rs` for private controller state and update dispatch, then
`controller/interior.rs`, `doorway.rs`, `surface.rs`, or `eva.rs` for the active
movement mode. `controller/camera.rs` owns look and snapshots. Regression tests
stay with these mode owners; `controller/collision_tests.rs` checks movement
through shared proxies and `controller/test_support.rs` contains test fixtures.
`state.rs` owns frame/snapshot/location contracts and private position variants;
`input.rs` owns typed controls and planar normalization. `layout.rs` centralizes
handwritten dimensions/proxies, `collision.rs` owns sweep/slide helpers,
`queries.rs` owns public sight/placement queries, and `math.rs` owns character
normalization/projection policy. Generated `ship_anchors.rs` and
`thruster_collision.rs` remain exporter-owned.
