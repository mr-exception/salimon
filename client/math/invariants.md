# Math invariants

1. All public operands/results use `f64`; no implicit GPU cast or origin rebase.
2. Cross products are right-handed: +X cross +Y = +Z. Primitives assign no
   gameplay meaning to axes; runtime +Y up and ship +X forward/-Z starboard
   remain the owning domains' contracts.
3. Dot products sum component products in X/Y/Z order; length is the square
   root of that sum. No `hypot`, rescaling or changed overflow/NaN policy.
4. No normalization fallback, tolerance, finite-input validation, world
   position type, frame transform, platform type or dependency belongs here.
5. Callers subtract absolute `f64` coordinates before any `f32` conversion.

Tests protect handedness, arithmetic, distant-anchor precision and IEEE edge
behavior. Normalization and quaternion tests stay with their domain policies.
