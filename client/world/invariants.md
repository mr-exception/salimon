# World Invariants

1. Canonical world positions, camera positions, radii, speeds, and distance math
   use finite `f64` meters.
2. The immutable catalog contains exactly, and in order: Sun, Mercury, Venus,
   Earth, Moon, Mars. IDs and names are unique, and radii are positive/distinct.
3. The Sun is the only visual-only body and exposes no landing radius. The other
   five bodies are solid and every pair of their `1.15R` landing volumes is
   disjoint.
4. The Earth-to-Mars nominal-surface gap is `60,000,000 m`; at the catalog
   reference maximum speed of `2,500,000 m/s`, it takes exactly 24 seconds.
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
11. Nearest-body queries compare nominal surface distance, include all catalog
    roles, accept an exact finite threshold, and preserve catalog order for ties.
12. Radial speed is the signed center-distance derivative: negative approaches,
    positive recedes, zero is stationary/tangential or an undefined center axis.
13. Resource keys are stable and independent of display names/catalog order.
    Raw material mass/derived volume are positive and finite; remaining deposit
    mass is finite, nonnegative, bounded by original mass, and cannot increase
    during local extraction updates.
14. Resource entity identities and material properties do not change when a
    fragment moves. Resource poses use finite absolute meters/unit quaternions;
    resource state carries no renderer data or abstract inventory counts.

- Mining targets require a forward ray intersection within 4 m, ahead of all
  supplied solid obstructions. Depleted deposits cannot be targeted.
- Extraction is 2 kg/s of simulation time capped to remaining mass; it cannot
  replenish deposits or credit abstract inventory.
- Every positive session extraction is represented in physical fragments of the
  source material, capped at 2 kg per piece. Their total mass equals removed
  deposit mass within floating-point tolerance; zero/depleted extraction creates
  no objects. IDs/poses survive growth and read-only requery.

- A local world session retains modified deposit mass by stable ID, including
  depleted tombstones, independently of active area queries. Restoration is
  idempotent, never increases mass, and precedes extraction even for stale copies.
  Stream-out/reload never regenerates mined mass or emits duplicate fragments.
