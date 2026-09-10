# World Invariants

1. Canonical world positions, camera positions, radii, speeds, and distance math
   use finite `f64` meters.
2. The immutable catalog contains exactly, and in order: Sun, Mercury, Venus,
   Earth, Moon, Mars. IDs and names are unique, and radii are positive/distinct.
3. The Sun is the only visual-only body and exposes no landing radius. The other
   five bodies are solid and every pair of their `1.15R` landing volumes is
   disjoint.
4. The Earth-to-Mars nominal-surface gap is `60,000,000 m`; at the Phase 0
   reference maximum speed of `500,000 m/s`, it takes exactly 120 seconds.
5. Earth's positive-Z surface is `SURFACE_ANCHOR` and is the default camera
   target. Inspection selects a body's positive-Z surface; the near dwell is
   exactly 12 m above the selected surface, including after restart.
6. Camera-relative positions subtract their `f64` origin before any `f32`
   conversion.
7. World code never depends on `winit`, `wgpu`, native handles, graphics
   clip-space conventions, networking, persistence, or backend services.
8. Approach and retreat are monotonic, dwell phases hold exact endpoints, and
   pause/restart/near-inspection commands remain deterministic.
9. Pausing changes only tour progression. The body catalog and three precision
   markers are immutable across snapshots.
10. Precision markers are noncanonical validation references and never count as
    celestial bodies or landing volumes.
