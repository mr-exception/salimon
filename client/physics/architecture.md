# Physics architecture

`ObjectState` is an ordered, caller-owned snapshot with f64 metre position,
linear velocity, unit XYZW orientation, angular velocity, mass, uniform scale,
conservative radius, optional shared immutable convex hull and explicit surface.
Physics advances motion and returns poses without allocating identities or
reading assets, native events, renderer state or resource sessions.

`convex.rs` constructs convex envelopes once from finite authored vertex sets.
Duplicate vertices and coplanar faces are merged; supporting face normals and
true edge directions supply the separating-axis test. Pair projections subtract
centers in f64 before testing axes. Radii only reject distant pairs and constrain
ship hull clearance; they do not determine convex pair separation. A missing
hull retains the previous sphere response for compatible callers.

`Surface::Sphere` supplies radial 9.81 m/s² gravity and local tangent support.
`Floor` supplies +Y gravity and one ship-local deck height; a pure containment
callback retains hull/furniture policy with the caller. Hull support is refreshed
from current orientation during every ground pass. Unsupported objects receive
-Y gravity without terrain projection. Floor/non-floor groups and separate
planet centers do not contact. Different ship frames require separate calls.

The solver integrates orientation and motion, performs 16 ordered convex contact
passes, projects penetration and applies mass-weighted normal/friction impulses
with angular response. Inertia is an isotropic approximation from mass and
bounding radius, not an exact volume integral. Ground contacts use authored
support points, allowing tilted objects to tip; damping bounds residual motion.
Geometry growth is safe because the hull stays immutable and each snapshot
supplies current mass/scale. No cached sleep state can retain removed supports.

Convex objects adapt substeps to scale and linear/angular speed, at most 1/90 s
and capped at 4096 steps; the minimum target is 10 µs. This limits ordinary drop
and ejection tunneling, but is not general continuous collision detection.
Legacy sphere-only calls retain the 48-step cap and three contact passes. Both
paths distribute all elapsed time. Convex envelopes intentionally bridge local
concavities, notably the clustered ice asset.

Release helpers retain existing impulses. Identity, material, lifetime, carried
exclusion, activation and frame conversions stay with the caller. Tests in
`convex.rs` protect narrow phase, rotation, settling, support removal and tipping;
legacy motion regressions remain in `lib.rs`.
