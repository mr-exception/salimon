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
supplies current mass/scale. Stateful activation invalidates removed supports through stable caller IDs.

Convex objects adapt substeps to scale and linear/angular speed, at most 1/90 s
and capped at 4096 steps; the minimum target is 10 µs. This limits ordinary drop
and ejection tunneling, but is not general continuous collision detection.
Legacy sphere-only calls retain the 48-step cap and three contact passes. Both
paths distribute all elapsed time. Convex envelopes intentionally bridge local
concavities, notably the clustered ice asset.

Release helpers retain existing impulses. Identity, material, lifetime, carried
exclusion, the activation cache lifetime and frame conversions stay with the caller. Tests in
`convex.rs` protect narrow phase, rotation, settling, support removal and tipping;
legacy motion regressions remain in `lib.rs`.

Development and test profiles optimize only the physics/math crates at level 2
to keep faceted contact loops responsive; debug assertions and information remain.

`surface_ejection_velocity` accepts caller-selected unit facet normal/radial up
and a stable speed lane. Its outward 2.0–2.6 m/s plus 1.8 m/s radial lift is
combined with 1.8 m/s lateral escape and a bounded ±0.4 m/s fan is
one-shot creation motion; gravity/contact take over immediately. The caller,
not a missing velocity cache entry, decides whether an object is new.

## Opt-in solver observations (#155)

`advance_profiled` uses the same const-specialized solver as `advance`, preserving
operation order and results. Normal calls compile out counters/clocks. `StepStats`
counts substeps, passes, all pair visits, radius candidates, convex narrow-phase
visits, resolved contacts and projection builds. Visits include repeated passes.
Integration includes initial environmental contacts; contact time includes matrix
allocation, pair resolution and repeated ground projection. Damping and matrix
teardown remain in total adapter-call time.

Matrix Vec allocations and requested matrix/projection capacity bytes are
structural allocation observations, with one outer vector and one vector per row, not
process-wide allocator totals. Convex temporary vertices/axes/ground arms and
allocator overhead are excluded. See the [benchmark guide](../../scripts/FRAGMENT_BENCHMARKS.md).

## Persistent contact-island sleep (#156)

`SleepTracker<K>` is caller-owned and keyed by stable identity; native/world
identity types do not enter physics. `advance_with_sleep` restores activation
beside ordered snapshots. The stateless `advance` path remains available as the
baseline. An island sleeps only after every member has linear/angular speeds
below 0.05 m/s and 0.2 rad/s, position within 1 mm and quaternion components
within 0.005 of a fixed dwell anchor for 0.5 simulation seconds. The dwell-wide
pose envelope rejects slow sliding/tipping while allowing bounded faceted-contact
chatter (observed residual velocities can exceed the solver's 0.015 cutoff). At least one member must have
actual environmental support within 1 mm. Fast face rejection followed by full convex SAT with the same 1 mm
gap forms contact edges; bounding-sphere overlap alone cannot anchor an island.

Sleeping poses stay in the solver snapshot as collision participants. Skip their
integration/damping and sleeping-to-sleeping contacts. A fully sleeping update
allocates no contact matrix and performs no solver passes. Mixed updates retain
the existing matrix and ordered pair loop; this is not a broad-phase replacement.
Tracker validation, snapshot adaptation and rendering still cost work.

Wake previous contact-connected neighbors on removal, pickup/selection omission,
changed pose, velocities, mass, scale, hull or surface. Swept conservative spheres
wake contact islands before a fast object's substep; actual contacts also wake
before applying response. This wake bound does not add a CCD guarantee. Missing
identities are pruned; streaming return starts awake and reestablishes support.
Floor containment is checked even for sleepers. Call `wake_all` when a force or
external environmental policy changes without changing a snapshot field.
Sleep is in-memory session state, not disk persistence. Hash maps only look up
identities; input/adjacency order determines stepping and island traversal.

`StepStats.activation_time` separates stable-ID validation plus post-solve
contact graph/dwell work from integration and contact response. Swept wake
checks within the solver remain included in total solver wall cost.
