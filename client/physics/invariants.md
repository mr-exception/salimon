# Physics invariants

- CPU positions are f64 metres in one caller-defined frame; velocities are m/s
  and angular velocities are frame radians/s. Mass, scale and bounds are positive;
  orientations are unit XYZW quaternions and immutable hull vertices are finite.
- Ordered input controls every contact pass. Zero delta changes nothing;
  identical inputs and deltas produce identical outputs.
- Gravity is 9.81 m/s², radial for spheres/sampled planetary surfaces and -Y otherwise. Floor containment
  is a pure caller query, with horizontal hull bounce -0.15.
- Convex face and edge axes determine final pair contact; bounding spheres only
  provide broad phase and conservative containment. Ground support follows the
  rotated hull each pass. Penetration correction uses a 10 µm separation bias.
  Supported convex deck bodies cannot take downward positional corrections;
  the neighbor receives unavailable normal separation. Mass/impulses stay unchanged.
  Pair velocity contacts admit a 20 µm skin; negative depth never attracts bodies.
  Resting faces use unilateral corner constraints on the actual overlapping
  support polygon (ground faces use supporting vertices), with 16 local iterations.
  Corner impulses can decrease but their accumulated totals remain nonnegative.
  Normal and tangential constraints alternate within each local iteration;
  unchanged features retain their multipliers across global substep passes.
  Warm starting uses current contact geometry and matching stable-ID snapshots;
  removed/changed bodies or explicit changed forces discard cached reactions.
  Response transport does not reuse rotated SAT spans or change separation.
  Distributed tangential impulses use 16 local iterations and a per-point
  Coulomb disk with radius 0.65 times its solved normal impulse; resting faces
  and edges therefore resist twist. Analytic spheres use one radial arm with the same Coulomb limit.
- Convex contacts use mass-weighted impulses, angular response and Coulomb friction
  capped at 0.65 of normal impulse. Low-speed convex impacts have no restitution;
  faster pair impacts use 0.12. Convex ground contacts are inelastic.
  Rough-contact rolling/spin torque is bounded by 0.1 of normal impulse times
  radius. Pair torques are equal/opposite; contact-free motion has no rolling drag.
- Linear/angular retention is `0.995^(dt/0.008)` / `0.98^(dt/0.008)`
  for elapsed seconds dt, calibrated to ordinary 8 ms substeps. Unpowered
  floors remain undamped. Convex rest requires an actual
  upward reaction and combined surface speed below 0.015 m/s; both velocities
  stop together after all contact iterations. The standalone angular cutoff
  remains 0.015 rad/s. Unsupported
  short updates cannot erase gravity. Convex contacts receive 16 passes before
  advancement and another 16 when the pose moves. Substeps adapt to scale and speed, target at most
  1/90 s and cap at 4096; the awake motion target is reevaluated after each
  response, including gravity speed. All elapsed time is retained. No CCD guarantee is made.
- Analytic spheres use exact radius support, 2/5 mr² inertia, sixteen pre/post
  passes and adaptive motion steps capped at 4096. Friction generates spin;
  contact-loaded rolling resistance dissipates it. Deep sphere overlaps cannot sleep.
- Floor/non-floor groups and distinct sphere centers do not contact. One floor
  reference frame may appear per call. Removed supports are reevaluated each step.
- Physics owns motion/contact rules only. Caller retains IDs/material/session,
  nearby/carried policy, pose validation and frame conversion. No renderer,
  runtime, GPU, world catalog or asset loader dependency enters this crate.
- Stateful sleep requires a supported contact island, low linear/angular motion,
  bounded actual material-point movement and 0.5 seconds observed dwell;
  the first observation cannot credit a preceding interval. Airborne rest is
  never sleep. See the
  [canonical sleep policy](architecture.md#persistent-contact-island-sleep-156).
- Sleeping objects retain visible/pickable/collision poses. Contact-island waking
  precedes collision response; removed/changed supports cannot remain cached.
- Stable IDs belong to the caller. Omitted snapshots invalidate cached sleep;
  retained SAT spans require unchanged exact orientation/shape and ID order.
  Rotation invalidates affected pairs; external changes rebuild the sparse cache.
  Fully sleeping groups release retained projection storage.
  explicit environmental/force changes outside snapshot fields call `wake_all`.
  Rigid movement of an unchanged local frame does not wake its contents.

- Spatial candidates must conservatively cover current contacts and swept wake
  reach. Corrections update membership before selecting the next ordered pair.
  Candidate filtering cannot reorder input identities or change exact SAT/response.
  Dense populations and oversize fallbacks retain quadratic worst-case cost.

## Terrain integration (#179–#181)

Follow the canonical [integration/budgets/validation guide](../../docs/terrain-validation.md)
and [world definition contract](../world/terrain-contract.md). Keep local rules
and their consumers aligned; flat deterministic fixtures are explicit.

- Unpowered floor contacts retain geometry and containment with zero artificial
  gravity; mode changes invalidate environmental sleep state.

## Logical save boundary (#364)

The [canonical save inventory](../../docs/save-contract.md) defines schema-1
source state and excluded caches. Domain-owned save state is validated before
restore; runtime composes temporary owners and rebuilds mass, immutable terrain
queries and presentation/query/solver caches without advancing simulation.
Stable IDs, depletion, carrying, motion, Core/assist progress and logical
contact/sleep/dwell state survive. No input latch, OS handle or GPU state is
restored. An attached mining sphere becomes a detached existing physical entity,
without allocating a new ID or losing mass. Whole-session/Fabricator publication
is composed in #365; disk checkpoints remain #366.

- Spatial query epoch stamps only deduplicate one query's snapshot indices.
  Corrections, swept reach, large-body fallback and sorted contact order retain
  the exact candidate set; rollover clears stamps before reuse.


## Exact impulse fixed points (#592)

One-point ground/pair response loops retain the sixteen-iteration policy. After
one complete iteration, the implementation may elide the remaining iterations
only if every mutable iterative input has identical IEEE bits: both linear and
angular velocities (one body for ground), accumulated normal impulse, tangent
impulse and rolling torque. Normal/arms/targets, mass/inertia and all geometric
inputs remain constant within this pure loop; position/environment processing
stays outside it. An exact fixed point deterministically repeats the same state.
No epsilon, approximate convergence, active/sleep/contact rule, damping or time
budget changes. Multi-point manifolds retain every original iteration.

`iteration_oracle.rs` retains the old sixteen-iteration ground/pair response for
exact serialized-state and warm-continuation checks. Existing spatial all-pairs,
correction-created contact, retained projection and sleep regressions remain.
Small shared arithmetic primitives are inlinable across the crate boundary;
their expressions and IEEE ordering are unchanged. See the #592 report for
archived mass/radius-bit pools, partial/timeouts, profiles and exact comparisons.
