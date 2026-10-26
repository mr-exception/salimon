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
   clip-space conventions, networking, disk storage, or backend services. Logical
   world save DTOs remain world-owned.
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
- Every positive legacy helper extraction is represented in physical fragments of the
  source material, capped at 2 kg per piece. Their total mass equals removed
  deposit mass within floating-point tolerance; zero/depleted extraction creates
  no objects. IDs/poses survive growth and read-only requery.

- A local world session retains modified deposit mass by stable ID, including
  depleted tombstones, independently of active area queries. Restoration is
  idempotent, never increases mass, and precedes extraction even for stale copies.
  Stream-out/reload never regenerates mined mass or emits duplicate fragments.

- Resource gameplay dimensions use the shared world `resource_size` policy;
  enlarged bounds never substitute for density-derived solid material volume.
  See [resource size contract](../world/resource-contracts.md#gameplay-size-policy-144).

- New-piece placement can reject a crowded surface before mass/ID allocation;
  accepted growth and earlier pieces remain conserved. Placement runs once per
  new identity, never on growth, pickup/drop or streaming restoration.

- Terrain definitions are immutable, versioned, bounded and body-local.
  Identical definitions/directions give order-independent heights; conservative
  extrema stay within ±0.01R. Sun has no terrain. See [terrain contract](terrain-contract.md)
  for codec limits and numerical compatibility. Nominal-sphere helpers remain for catalog/tour/legacy fixtures; enabled gameplay
  uses the shared sampled query. See [integration](../../docs/terrain-validation.md).

## Terrain integration (#179–#181)

Follow the canonical [integration/budgets/validation guide](../../docs/terrain-validation.md)
and [world definition contract](../world/terrain-contract.md). Keep local rules
and their consumers aligned; flat deterministic fixtures are explicit.

- Physical fragment mutations emit coalesced stable-ID events; despawn/Core
  consumption releases carrying and IDs are never reused. Querying/streaming
  does not mutate or remove cargo. Historical extracted mass remains diagnostic.

- Continuous spherical mining reserves one attached mass with a configurable
  10 kg default cap, releases on input/aim/cap/depletion/interruption, and starts
  the next piece on the next held valid tick. Removed mass equals released plus
  reserved mass. See [the lifecycle contract](resource-contracts.md#spherical-mining-lifecycle-230).

- Every retained fragment has exactly one ID-to-Vec-slot entry. Ordered removal
  repairs shifted slots; growth/pickup/drop never reorder survivors. Index hash
  iteration cannot determine journal or solver order. Removed IDs are never reused.

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

## Physical mounts (#576)

Fixture pickup and fragment pickup use the same single CarrySlot; source owners cannot reconstruct two carried objects. Resource identity and mining debit remain unchanged. See [the canonical contract](../../docs/physical-mounts.md).


Instant Core upgrades #548: [canonical input, source, mass and schema-14 contract](../../docs/core-upgrades.md). Ship owns validated capacity-only transactions; world returns original caller-owned reserved entities through the one carry slot; runtime maps the data catalog/physical controls and reconstructs pending+installed material mass. No timer, joule grant, Core replacement or flight command.


Construction receipts/refunds #543: [canonical transaction, mass, controls and schema-15 contract](../../docs/construction-materials.md). Funding respects the explicit economy approval gate; progress/geometry retain #544/#545 owners.


## Construction damage (#549)

Follow the [canonical damage, physical salvage and schema-21 contract](../../docs/construction-damage.md). Surviving work preserves paid time and receipts; lost cells and destroyed projects cannot respawn or duplicate salvage. Core identity is protected. Fresh debris IDs, exact material shares, collision/containment and installed mass publish together; restore reconstructs source without reallocating debris.
