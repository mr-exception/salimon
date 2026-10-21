# Character Invariants

1. Powered ship-floor and solid-body walking use fixed 9.81 m/s² gravity.
2. Ship power owns artificial gravity inside/seated: powered gives floor gravity,
   unpowered gives zero gravity, irrespective of nearby planetary surfaces.
3. A closed door blocks passage in either direction. An open flying airlock
   admits space EVA; landed crossings use continuous ramp support. Closing
   during a doorway blend ends it before movement and resolves overlap to the
   nearer physical side; an existing surface walker always remains outside.
4. A landed fully deployed ramp blends up by distance along the support, retaining contact indefinitely at rest.
   WASD stays camera-relative throughout the blend; re-entry requires movement
   toward the cabin rather than a particular key.
5. Grounded surface positions use the supplied ground query plus radial eye height.
   Terrain jumping/traversal obey the canonical integration policy; flat fixtures
   preserve their original complete-sphere projection.
6. Cockpit entry/exit is instant and does not mutate ship velocity or orientation.
7. The crate receives no native events and exposes no GPU types.
8. Positive horizontal mouse delta turns toward the camera's screen-right direction;
   vertical mouse delta retains the conventional down-is-positive device mapping.
9. The default seated cockpit view remains primarily forward and clears the
   console/nose into the modeled cockpit glazing.
10. Ship-local floor, walk bounds, doorway, player start, and cockpit
    camera remain aligned to the wider 4.00 m-tall asset; player body height is
    1.80 m and standing eye height is 1.75 m.
11. The central Core pedestal, pilot chair, and unified starboard
    cockpit console with three monitors are solid to walking movement, with
    body-radius clearance and edge sliding. The aisles on both sides reach the nose.
12. Rear window bulkheads remain solid with the door open. Doorway crossings
    require enough lateral clearance for the player's complete body, including
    oblique movement throughout ramp traversal.
13. Spawn and cockpit exit place the player in the starboard aisle without
    intersecting the Core, chair, or walking boundary.
14. Jumping keeps the complete player body below the ceiling fixtures and the
    lower door lintel; reaching the ceiling cancels upward speed.
15. Surface W/S movement follows the camera's horizontal forward/back direction,
    A/D follows its tangent-plane right/left direction, and all four directions
    remain tangent to the active solid body at every sphere orientation.
16. The broad cockpit deck ends at X 6.56 m and a narrow nose floor continues
    to X 7.85 m. Body-expanded shoulder and side-hull proxies keep walkers on
    those floors and out of the exterior shell.
17. Surface walkers collide with the front, sides, and aft bulkheads regardless
    of door state. The open aft gate is the only entry route, with the
    same body clearance as interior traversal. Hull contact preserves surface
    eye radius and permits sliding; distant or vertically separated surface
    walkers cannot be captured by the doorway.
18. Closing around a surface walker in the gate clears the overlap before
    walking or jump input is processed. A closed gate cannot grant ship gravity,
    interior state, or cockpit entry. Repeated reopening restores the ordinary
    distance-based up blend and body-clear ramp passage in either direction.
19. Both engine bodies and their raised fins block exterior surface movement
    from all planar approaches while vertically overlapping the player's body.
    Their boxes follow the ship frame and leave clearance beyond their bounds.

20. Space exits inherit world position, linear velocity and orientation from the
    current ship frame. No-input drift preserves that inherited velocity;
    subsequent ship thrust or steering cannot silently attach the player again.
21. EVA translation is normalized in 3D and integrated using elapsed seconds.
    Flight assist cancels only the commanded motion on release. Re-entry resets
    EVA control state and adopts interior gravity without changing ship velocity.

22. Nearby-body airborne state preserves position and drift on influence changes.
    Only one selected solid body applies fixed radial gravity. Ship interior
    gravity owns the player after re-entry; body contact uses that selected
    surface and never a distant ship's nearest-body frame.

## Implementation owners

These numbered rules remain unchanged by the internal decomposition. Interior
and cockpit rules are in `src/controller/interior.rs`; landed transitions and
closure recovery are in `controller/doorway.rs`; surface rules and tangent/height
projection are in `controller/surface.rs`; drift, assist and nearby-body rules
are in `controller/eva.rs`. Look/snapshots live in `controller/camera.rs`.
`src/layout.rs` and `collision.rs` supply shared geometry and sliding; `queries.rs`
uses the same proxies for sight/placement. Mode regressions remain alongside
owners, with hull/appendage integration coverage in `controller/collision_tests.rs`.
See the [source ownership table](architecture.md#internal-ownership).

23. Scout-specific floor, doorway, interior/exterior envelopes, fixtures,
    cockpit sides and appendage geometry come from exporter-generated
    `src/spatial_contracts.rs`. `layout.rs` derives body-clear proxies and
    retains gameplay policy; never reintroduce manual ship dimensions.
    The source [geometry/policy mapping](../../models/assets/ships/salimon-scout/README.md#geometry-and-gameplay-policy)
    defines the coarse proxy semantics and supported symmetry/level-floor assumptions.

24. Fixed landing feet/legs have generated finite-height collision boxes. Raised
    hull clearance does not become an invisible solid wall beneath the ship.
    The aft boarding corridor remains body-clear, closed-door gated and restricted
    to the existing doorway; the authored bottom-hinged ramp supplies continuous sloping support.

## Terrain integration (#179–#181)

Follow the canonical [integration/budgets/validation guide](../../docs/terrain-validation.md)
and [world definition contract](../world/terrain-contract.md). Keep local rules
and their consumers aligned; flat deterministic fixtures are explicit.

## Interior zero-G (#191)

On outage, interior character movement uses ship-local view-relative three-axis
acceleration (WASD, Space up, Shift down), retaining walking/jump momentum.
Released controls coast; controlled speed is bounded by the existing 3.8 m/s
movement speed without reducing faster inherited momentum. Body-clear fixture/wall/deck/ceiling contacts remove blocked
velocity components. Exterior planetary gravity never enters this path. Power
return preserves position and vertical velocity before ordinary deck gravity
settles the player. Airlock EVA inherits ship plus local drift; seated occupancy
remains mechanically fixed. Held fragments follow existing carrying rules in
both powered and weightless interior modes.

Loose cargo's unpowered surface preserves velocity without ambient damping and
adds conservative radius-based ceiling containment; horizontal hull/fixture
containment and precise deck support remain unchanged. Surface changes invalidate
sleep support through existing snapshot matching. No acceleration-induced cabin
forces, crew inertial simulation or new contact engine is introduced.

## Wall Fabricator (#342)

The generated Fabricator box expands by the existing player radius for interior
walking, zero-G and loose-object floor placement. Both Core aisles and the aft
door route remain clear. Three generated markers own input, screen and tray
positions. Ordinary sight queries treat the module as solid; the port query
omits only its own conservative envelope because the input face is recessed
behind the tray. Runtime still checks interior authority, front face, range and aim.

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

- Voluntary speed scaling (#369) applies consistently in cabin, ramp, surface and
  EVA without scaling physical clocks, gravity or inherited motion. Derived scale
  is transient and must be recomputed by the host after save restoration.
