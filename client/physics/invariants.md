# Physics invariants

- CPU positions are f64 metres in one caller-defined frame; velocities are m/s
  and angular velocities are frame radians/s. Mass, scale and bounds are positive;
  orientations are unit XYZW quaternions and immutable hull vertices are finite.
- Ordered input controls every contact pass. Zero delta changes nothing;
  identical inputs and deltas produce identical outputs.
- Gravity is 9.81 m/s², radial for spheres and -Y otherwise. Floor containment
  is a pure caller query, with horizontal hull bounce -0.15.
- Convex face and edge axes determine final pair contact; bounding spheres only
  provide broad phase and conservative containment. Ground support follows the
  rotated hull each pass. Penetration correction uses a 10 µm separation bias.
- Convex contacts use mass-weighted impulses, angular response and Coulomb friction
  capped at 0.65 of normal impulse. Low-speed convex impacts have no restitution;
  faster pair impacts use 0.12. Convex ground contacts are inelastic.
- Linear/angular damping is 0.995/0.98 per substep, with 0.015 speed cutoffs.
  Convex passes number 16. Substeps adapt to scale and speed, target at most
  1/90 s and cap at 4096; all elapsed time is retained. No CCD guarantee is made.
- The optional sphere fallback preserves three passes, the 48-step cap and
  original ground restitution/tangent damping. Missing hulls never spin.
- Floor/non-floor groups and distinct sphere centers do not contact. One floor
  reference frame may appear per call. Removed supports are reevaluated each step.
- Physics owns motion/contact rules only. Caller retains IDs/material/session,
  nearby/carried policy, pose validation and frame conversion. No renderer,
  runtime, GPU, world catalog or asset loader dependency enters this crate.
