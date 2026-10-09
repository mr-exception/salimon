# Physics invariants

- Positions/surface centers are f64 metres in a caller-defined consistent frame;
  velocity is m/s. Callers supply finite values and positive radii.
- Caller order controls pair iteration; no hash iteration or random source.
- Zero delta leaves state untouched. Substeps target 1/90 s, cap at 48 and retain
  all elapsed time. Repeat runs with identical ordered inputs produce identical outputs.
- Gravity remains 9.81 m/s², radial for spheres and -Y otherwise. Restitution is
  0.12; hull horizontal bounce is -0.15; ground tangent damping is 0.88;
  per-step damping is 0.995 with a 0.015 m/s sleep cutoff.
- Three pair passes use equal contact weights and a 0.0001 m separation bias.
  Floor/non-floor groups and distinct sphere centers never contact each other.
- Floor containment is a pure geometry query; physical response stays here.
  Only one floor frame may appear per call.
- Identity, mass, material, orientation, carried state, nearby activation, world
  pose validation and frame conversion remain caller-owned.
- No runtime, renderer, winit, GPU, world catalog or resource dependency.

- Caller-supplied ground support is finite and positive, independent of contact
  radius; terrain/floor projection uses support, pairs/containment use radius.
