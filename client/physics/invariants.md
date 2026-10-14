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
  and edges therefore resist twist. Sphere fallback friction stays unchanged.
- Convex contacts use mass-weighted impulses, angular response and Coulomb friction
  capped at 0.65 of normal impulse. Low-speed convex impacts have no restitution;
  faster pair impacts use 0.12. Convex ground contacts are inelastic.
  Rough-contact rolling/spin torque is bounded by 0.1 of normal impulse times
  radius. Pair torques are equal/opposite; contact-free motion has no rolling drag.
- Linear/angular damping is 0.995/0.98 per substep. Convex rest requires an actual
  upward reaction and combined surface speed below 0.015 m/s; both velocities
  stop together after all contact iterations. The standalone angular cutoff
  remains 0.015 rad/s. Unsupported
  short updates cannot erase gravity. Convex contacts receive 16 passes before
  advancement and another 16 when the pose moves. Substeps adapt to scale and speed, target at most
  1/90 s and cap at 4096; the awake motion target is reevaluated after each
  response, including gravity speed. All elapsed time is retained. No CCD guarantee is made.
- The optional sphere fallback preserves three passes, the 48-step cap and
  original ground restitution/tangent damping. Missing hulls never spin.
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
