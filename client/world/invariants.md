# World Invariants

1. Canonical world positions and camera positions use finite `f64` meters.
2. Camera-relative positions subtract their `f64` origin before any `f32`
   conversion.
3. World code never depends on `winit`, `wgpu`, native handles, or graphics
   clip-space conventions.
4. The renderer never receives or mutates authoritative world state.
5. Camera altitude and near-plane values remain finite and strictly positive
   throughout the tour.
6. Approach and retreat are monotonic, dwell phases hold exact endpoints, a
   restart deterministically returns to the far approach endpoint, and a near
   inspection command selects the exact near-dwell endpoint and pauses there.
7. Pausing changes only tour progression; snapshots and primitive definitions
   remain stable.
8. Prototype primitives are immutable validation references. They are not real
   Phase 0 celestial data and must not leak into Task 5 as canonical content.
