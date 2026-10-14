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
Pair projection axes remove numerical parallel repeats in generation order with
1e-9 component bins. Support-graph queries test the same face/crossed-edge SAT
axes, stopping at the first separating axis without allocating a response
manifold. Support detection reuses the last substep's cached SAT spans when
available: orientation and scale remain constant through positional contact
corrections, and the final graph checks the updated center separation. Missing
cached pairs still use full SAT. These preserve convex separation; no sphere
approximation is introduced.

`Surface::Sphere` supplies radial 9.81 m/s² gravity and local tangent support.
`Floor` supplies +Y gravity and one ship-local deck height; a pure containment
callback retains hull/furniture policy with the caller. Hull support is refreshed
from current orientation during every ground pass. Unsupported objects receive
-Y gravity without terrain projection. Floor/non-floor groups and separate
planet centers do not contact. Different ship frames require separate calls.

For convex objects the solver applies gravity and current-pose contacts before
advancing position/orientation. Sixteen ordered contact passes solve each current
pose; a moved pose receives another sixteen passes after integration. Rotation
invalidates affected SAT caches. Penetration projection and mass-weighted
normal/friction impulses include angular response. Inertia is an isotropic approximation from mass and
bounding radius, not an exact volume integral. Ground contacts use authored
support points, allowing tilted objects to tip; damping bounds residual motion.
Convex rest requires an upward reaction exceeding half the substep gravity
impulse and combined surface speed `|v| + |angular velocity| * radius` below
0.015 m/s. Both velocities then become zero, after all contact passes for the
substep. Cutting between passes would invalidate accumulated constraint impulses.
Translation is not cut separately
while rolling remains meaningful; unsupported bodies accumulate gravity even
with very short updates. The existing standalone 0.015 rad/s angular cutoff and
0.995 / 0.98 linear/angular damping remain.
Supporting ground faces use their near-coplanar vertices. Pair face contacts
clip the two support polygons in the contact tangent plane, then solve normal
constraints at the intersection corners. Sixteen local projected iterations
accumulate nonnegative impulses and can undo an excess corner impulse; a single
center reaction cannot balance an off-center load in a stack. Face/edge contacts retain the clipped segment's endpoints; vertex contacts
retain one reaction arm. Parallel edge contacts use both overlap endpoints;
crossing edges use the actual closest points rather than the midpoint of their
tangent bounding boxes. Ground edges use both supporting vertices. Distributed
friction uses the updated contact velocity at each point. Sixteen local projected
iterations alternate normal and tangent constraints, with a Coulomb disk capped
at 0.65 of that point's accumulated normal impulse. Normal/tangent multipliers
persist across global passes within a substep while the normal is unchanged
(to 1e-8) and both arms remain within 100 µm of their saved feature; changed
features start fresh. Ground constraints retain multipliers in the same way.
Across substeps, warm starting transports saved impulses to the current normal
and arms only when the normal dot product exceeds 0.999 and both arms move less
than 1 mm. Tangent impulses are projected onto the new tangent plane and clamped
to the current Coulomb disk. The same transport retains pre-advancement pressure
through the post-advancement solve while rotated SAT geometry is rebuilt exactly.
These response matching tolerances do not change SAT separation or hull geometry. Faces and edges resist twist as well as translation. Ground friction
uses the solved normal load, including supported cargo, rather than only the
body's own weight. Rough contact rolling/spin loss is a separate accumulated
angular impulse capped at 0.1 times normal impulse times bounding radius (the
smaller radius for pairs). It exchanges equal/opposite pair torques and requires
a current contact load; unsupported objects receive no rolling drag. This is a
uniform approximate contact law, not a material-specific deformation model. The legacy sphere response is unchanged. Pair velocity constraints remain active across a 20 µm
contact skin around the 10 µm positional bias, preventing correction from
prematurely dropping a loaded velocity constraint.
Positional separation respects a convex body's existing deck support: downward
correction is projected onto its available horizontal directions, and remaining
normal separation goes to its neighbor. This prevents subsequent deck projection
from undoing pair separation in loaded stacks. Mass, inertia and velocity impulses
remain physical; this constraint does not apply to unsupported or sphere-fallback
bodies.
The pair minimum-axis cache retains a candidate set and only skips excluded
axes when a displacement bound proves they cannot become the minimum. It
falls back to the full exact SAT scan whenever the bound is inconclusive.
The stateful solver retains sparse exact pair projections across substeps and
updates. Rotating a body invalidates its affected pairs; external changes,
reordered/omitted IDs or even an orientation ULP rebuild the retained pair cache.
Fully sleeping islands release it. This reuses unchanged geometry without
changing SAT axes, contact response, iteration order or physical poses.
Retained pairs rerun the first-use face rejection in each substep; unused
records neither replace pending reactions nor enter the cached support path.
Ground and pair reaction arms include vertices within 1 mm per unit scale of
the extreme supporting feature. This submillimetre tolerance at normal fragment
sizes avoids switching to a lone vertex under resting-face contact noise while
retaining off-center tipping and exact SAT separation.
Geometry growth is safe because the hull stays immutable and each snapshot
supplies current mass/scale. Stateful activation invalidates removed supports through stable caller IDs.

Convex objects adapt substeps to scale and linear/angular speed, at most 1/90 s
and capped at 4096 steps; the minimum target is 10 µs. The convex motion target
is recomputed after each substep using current awake linear/angular motion plus
a conservative gravity-speed increment. A fast impact can therefore release
its initial timestep pressure during the same update. The base subdivision
remains at most 1/90 s, with identical 8 ms subdivisions for ordinary 16 ms
updates. Sleeping objects constrain no motion until waking; swept wake/contact
checks and shared time progression remain in every substep. This limits ordinary drop
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

`ConvexHull::complexity` exposes immutable vertex, supporting-face, unique
face-axis and unique edge-axis counts for geometry audits. Faces and face axes
differ when facets are parallel. For hulls A/B, the raw pair SAT axis upper bound
is A.face_axes + B.face_axes + A.edge_axes * B.edge_axes; parallel deduplication,
early rejection and retained projections reduce actual work. These counts do
not change contact geometry or measure per-frame projection work.

The [#160 evaluation](../../reports/issue-160/README.md) retains today's authored
envelopes. Extent-derived eight-vertex box controls reduce active pair cost but
introduce unacceptable additional support clearance. No replacement collision
proxy or visibility-dependent simulation policy is adopted; smaller authored
proxies need a declared accuracy budget and measured benefit before selection.

`advance_profiled` uses the same const-specialized solver as `advance`, preserving
operation order and results. Normal calls compile out counters/clocks. `StepStats`
counts substeps, passes, all pair visits, radius candidates, convex narrow-phase
visits, resolved contacts and projection builds. Visits include repeated passes.
Integration includes gravity, pose advancement, damping and environmental contacts;
contact time includes spatial queries, pair resolution and repeated ground projection.
Cache invalidation and teardown remain in total adapter-call time.

The legacy `matrix_allocations` field now counts sparse pair-table capacity growth
at solve boundaries; `projection_storage_bytes` counts requested pair-entry,
boxed geometry, projection-span and SAT candidate-index capacity growth. Hash
control bytes, grid buckets, transformed hulls, ground/response/motion scratch and
allocator overhead are excluded. These are structural observations, not total
allocator counts or resident memory. Retained capacity is not counted again.
Historical matrix measurements have a different representation; compare work,
individual storage components and wall times separately. See the
[benchmark guide](../../scripts/FRAGMENT_BENCHMARKS.md).

## Deterministic spatial candidates (#157)

`broad_phase.rs` uses conservative radius AABBs and a uniform grid with cell width
of twice the population's median radius. Queries check actual AABB overlap,
sort/deduplicate input indices and resolve `(i,j)` in the original lexicographic
snapshot order. The caller keeps stable identity order. Separate ship frames use
separate calls; the existing surface/planet-center gates still apply. Grid
coordinates subtract a local origin in f64; radii include contact skin and a
conservative rounding guard. Rotations remain covered by the bounding radius.
Very large bodies or swept query regions (over 512 cells) use an exact linear
fallback instead of enumerating unbounded cells; there is no population cap.

Every current-pose solve rebuilds membership, retaining bucket/vector capacity.
Every pair's positional/ground corrections immediately update affected bounds
and requery the current row for later IDs. Correction-created contacts are
therefore visited in the same pass as brute force; earlier pairs wait until the
next pass. Integrated poses get refreshed membership before the post-motion
solve. This preserves adaptive-substep behavior without claiming CCD. Wake
queries expand bounds by linear/angular/gravity travel before integration;
support graphs query the final pose with the existing 1 mm contact gap.

Pair geometry is stored only for surviving convex candidates, pruned on pose
refresh and solve completion. Candidate buffers, grid buckets, transformed hull
vertices/normals/edges and SAT minimum-axis indices reuse capacity. Exact
orientation/scale/hull-pointer changes rebuild transformed geometry; rotation
invalidates affected pair spans while transporting warm reactions as before.
Center corrections retain relative projections. Stable-ID snapshot validation
invalidates external pose/support/topology changes. Hash iteration never controls
contact order. Fully sleeping islands release projection storage.

Sparse/separated populations avoid all-pairs scans/storage. Dense piles and
oversize fallbacks can still require quadratic work; exact correction refresh
adds work per moved contact. Ground/manifold/response temporary allocation is
not eliminated. Tests retain the old projection arithmetic and an all-pairs
candidate oracle, comparing exact poses through dense/random/rotated/mixed-size,
fast and correction-created-contact cases.

## Persistent contact-island sleep (#156)

The tracker also retains solved ground/pair reactions by stable identity for warm
starting. Matching body snapshots are required; changed pose, velocity, angular
velocity, mass, scale, hull or environment discards affected reactions. Omitted
IDs and `wake_all` invalidate them; rejected floor support clears affected cached
reactions. Reordered pairs start fresh when their ordered direction reverses.
Each substep scales previous impulses by its duration ratio, capped at 2, and
applies them once before solving. New/current SAT and ground support still gate
every reaction. Fully asleep updates do not apply cached reactions or integrate.


`SleepTracker<K>` is caller-owned and keyed by stable identity; native/world
identity types do not enter physics. `advance_with_sleep` restores activation
beside ordered snapshots. The stateless `advance` path remains available as the
baseline. Each member of a supported island must pass a 0.5 simulation-second
observation window. Every observation remains below 0.15 m/s / 1.0 rad/s and
belongs to a component with actual environmental support within 1 mm. Full
convex SAT with a 1 mm gap forms contact edges; bounding-sphere overlap alone
cannot anchor an island. Convex pair penetration above 1 mm prevents sleep.

Rest observation tracks the center and every authored convex-proxy vertex
(six axial surface points for the legacy sphere fallback). Each point's actual
position history must fit a growing enclosing sphere with radius below
`0.03 m/s * 0.5 s = 15 mm`. A moving enclosure accommodates bounded contact
chatter while translation and rotation share one geometric, scale-aware rule.
This is a radius-growth budget: steady travel spanning 30 mm in the window
reaches the limit. Quaternion-component thresholds and post-solver velocity
integration are not used to infer pose movement. Measurements use relative
coordinates to retain large-world precision; boundary quaternion interpolation
uses the shortest sign and unit normalization.

The 0.03 m/s / 0.5 s reference follows the point-envelope approach described by
[Jolt's sleep settings](https://github.com/jrouwe/JoltPhysics/blob/master/Jolt/Physics/PhysicsSettings.h)
and [body sleep test](https://github.com/jrouwe/JoltPhysics/blob/master/Jolt/Physics/Body/Body.cpp).
The implementation here uses actual proxy vertices and the existing custom
Rust solver, support graph and velocity guards. No engine dependency is added.
Motion above the envelope, unsupported observations and speed failures keep the
island awake. A first observation receives no preceding quiet-time credit.
Changed snapshots and explicit waking restart the window. Enclosing centers
are diagnostics only; simulation poses are never replaced by them.

Sleeping poses stay in the solver snapshot as collision participants. Skip their
integration/damping and sleeping-to-sleeping contacts. A fully sleeping update
allocates no pair projection storage and performs no solver passes. Mixed updates use the spatial candidates above while keeping sleeping objects
in the bounds and collision snapshot.
Tracker validation, snapshot adaptation and rendering still cost work.

Wake previous contact-connected neighbors on removal, pickup/selection omission,
changed pose, velocities, mass, scale, hull or surface. Swept conservative spheres
wake contact islands before a fast object's substep; actual contacts also wake
before applying response. For convex hulls the pre-integration wake also requires
SAT proximity expanded by linear/angular travel and gravity over the substep;
overlapping conservative spheres alone cannot repeatedly wake separated hulls.
This wake bound does not add a CCD guarantee. Missing
identities are pruned; streaming return starts awake and reestablishes support.
Floor containment is checked even for sleepers. Call `wake_all` when a force or
external environmental policy changes without changing a snapshot field.
Sleep is in-memory session state, not disk persistence. Hash maps only look up
identities; input/adjacency order determines stepping and island traversal.

`StepStats.activation_time` separates stable-ID validation plus post-solve
contact graph/dwell work from integration and contact response. Swept wake
checks within the solver remain included in total solver wall cost.
`contact_neighbors(id)` and `environment_supported(id, floor_contains)` observe
the last completed contact/support snapshot without changing it; absent entries
have no contacts or support. E2E uses these to choose an actual single-support
removal, rather than guessing from bounding distances.
`quiet_objects` reports bodies that passed dwell, including those kept awake by
another unstable island member. `is_sleeping(id)` observes the last completed
update; absent identities report false.

## Active-group evaluation (#159)

Independently stepping a previous contact island is not safe here: swept travel
can join islands, and exact positional corrections can create new contacts in
the same pass. A fixed expansion of initial contact AABBs does not bound arbitrary
loaded-pile corrections. A safe island scheduler would need conservative swept
components, correction-driven merges and cache/index remapping, or rollback.
This change retains synchronized progression and correction-refresh semantics.
Sleeping islands skip integration and sleeping-to-sleeping response; recomputed
motion targets stop a fast impact from imposing its *initial* subdivision after
its response has slowed it. An unrelated awake object still shares the minimum
current target. No independent active-island stepping or dense-pile linear-cost
claim is made. This remains a candidate for a separately measured follow-up.

Runtime bounds native calls to 16 ms and at most two calls/render, with explicit
backlog; see [the shared-clock contract](../runtime/architecture.md#bounded-gameplay-ticks-and-fragment-interpolation-159).
The portable standalone APIs retain complete caller delta semantics and the
existing 4096/48 safeguards, so very large standalone calls can still enlarge
substeps at the cap. Neither path is general continuous collision detection.

## Solver selection (#161)

Keep the custom solver for the accepted Phase 0 settled 500-cargo iteration.
The [evaluation decision](../../reports/issue-161/README.md) defers Rapier's
isolated comparison and adoption; it does not establish superiority, current
reference-machine budgets or efficient dense active behavior. No prototype or
dependency is introduced. Reopen comparison on repeated active-budget failures,
ordinary interaction stalls, CCD/stability requirements or excessive solver
maintenance. Preserve authored support accuracy, stable caller IDs, one-object
carrying and frame/session ownership when comparing an f64 candidate.

## Terrain integration (#179–#181)

Follow the canonical [integration/budgets/validation guide](../../docs/terrain-validation.md)
and [world definition contract](../world/terrain-contract.md). Keep local rules
and their consumers aligned; flat deterministic fixtures are explicit.
