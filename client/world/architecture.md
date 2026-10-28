# World Architecture

## Source map

`src/lib.rs` is the public facade: existing root imports stay stable, while
non-resource implementation modules remain private.

| Source | Responsibility and existing tests |
| --- | --- |
| `src/coordinates.rs` | `WorldPosition`, absolute `f64` arithmetic and subtract-before-cast regression |
| `src/terrain/` | Validated/versioned terrain definitions, binary codec, pure height/normal queries and reproducibility tests; [contract](terrain-contract.md) |
| `src/catalog.rs` | Body IDs/roles/definitions, ordered static catalog, anchor/reference speed, membership and travel tuning tests |
| `src/geometry.rs` | Body surface/landing distances, radial speed, sphere separation and landing-volume tests |
| `src/proximity.rs` | Nearest-surface and nearby-solid selection, inclusive threshold and influence tests |
| `src/precision.rs` | Three immutable noncanonical markers, scalar spacing reports and numeric precision regressions |
| `src/camera.rs` | Camera commands/state/snapshots, timeline sampling and deterministic tour tests |

Resource modules keep their existing ownership and public module paths.

## Responsibility

The `resources` module owns the [planetary resource contracts](resource-contracts.md):
stable material/entity identities, SI mass/volume properties, bounded deposit
state, and physical fragment poses. It has no rendering or backend dependencies.

`salimon-world` owns portable absolute coordinates, the immutable six-body
catalog and geometry helpers, the deterministic engineering camera tour, three
noncanonical precision markers, and numeric precision reporting. It does not own
platform input, frame scheduling, GPU buffers, projection matrices, depth
textures, presentation policy, orbital simulation, or backend behavior.

```text
runtime monotonic delta + typed CameraCommand
    -> salimon-world CameraPrototype
        -> borrowed WorldSnapshot
           (f64 camera + six bodies + three precision markers)
    -> runtime maps domain structs into renderer sphere/light/cuboid DTOs and bounded terrain patches
       and diagnostics BodyDistance values
    -> renderer subtracts camera in f64 and casts relative values to f32
```

The catalog is a static ordered slice. `CelestialBody` carries stable identity,
name, absolute center, compressed radius, display color, and `BodyRole`.
Geometry helpers calculate center distance, signed nominal-surface separation,
optional solid landing-volume separation, and nonnegative point-to-surface
distance. They also provide deterministic nearest-surface selection and signed
radial point velocity without assuming a ship or renderer. `VisualOnly`
deliberately makes the Sun's landing radius absent but not absent from proximity.

The runtime is the composition root and translates `winit` events into
`CameraCommand`. The renderer does not depend on this crate. It receives generic
renderer DTOs produced by field-by-field runtime mapping: a body becomes a
sphere with an absolute `f64` radius, while precision markers remain separate
cuboids. Generic material selection and Sun-to-point-light mapping belong to
runtime; textured sphere drawing and lighting belong to renderer.

The camera retains a selected catalog identity. `InspectBody` restarts the same
validation timeline at that body's positive-Z surface; default is Earth, and
restart retains selection. No catalog positions or gameplay behavior change.

`WorldPosition::camera_relative_f32` documents and tests the numeric conversion
order but does not move presentation conversion into world. Projection matrices,
clip-space conventions, depth targets, and reverse-Z policy remain renderer
concerns.

The portable `mining` module owns normalized aimed rays, first-deposit selection,
solid-body obstruction, bounded time-based extraction, and modified-deposit
session state. Runtime provides a character ray and ship obstruction distance,
then applies session deltas to freshly generated candidates for inspection and
presentation. No renderer types, input events, or abstract inventory enter world.

`resource_fragments` receives the actual extracted mass inside `MiningSession`
and owns physical output, stable session IDs and bounded pieces. Its optional
placement callback accepts a new finite pose before debiting output mass; blocked
placement preserves unconsumed mass and identity. Growth never invokes placement. Fragment queries are read-only and independent of deposit
streaming. Runtime maps these domain objects into nearby inspection and authored mesh DTOs.

## Carrying and session lifetime

`carrying.rs` owns the one-object slot and occluded fragment aiming.
`MiningSession` guards pickup/release and validated pose mutation, sealing output
on pickup. Runtime composes hand poses, ship support and fragment motion/contact
with character collision geometry. Generated deposits can stream out and back
in without losing modified mass; fragments remain the same physical entities.
All retention is in memory for the current session, with no disk/backend save.

## Integrated terrain

Portable definitions, sampled surfaces and compact codecs live in src/terrain/;
TerrainMode selects the immutable native catalog profiles. See
[terrain-contract.md](terrain-contract.md) and the canonical
[integration guide](../../docs/terrain-validation.md). Runtime owns patch
generation, resource anchoring and supplying portable ground queries to consumers.

## Terrain integration (#179–#181)

Follow the canonical [integration/budgets/validation guide](../../docs/terrain-validation.md)
and [world definition contract](../world/terrain-contract.md). Keep local rules
and their consumers aligned; flat deterministic fixtures are explicit.

## Physical mass accounting (#187)

Ship's `ShipMass` caches 100,000 kg dry hull plus positive finite physical masses
by stable `MassObjectId` (separate fragment/equipment namespaces). Upsert/removal
is idempotent; handling and storage do not add another identity. Installed
components can use the same ledger; the current asset has no separately defined
installed-equipment masses beyond dry hull. No inventory/container implementation
or invented component weights is added. `reconciles` is an explicit diagnostic,
not flight work. `set_total_mass_kg` preserves force capacities and velocity.

World coalesces creation/growth, pickup, carried/loose pose changes and physical
removal into an ordered mutation journal. `remove_fragment` is the future Core
consumption/despawn boundary and never reuses IDs. Streaming/read-only queries
emit nothing. Runtime drains changes after carrying/physics synchronization,
classifies centers in the current authored cabin/nose frame, and updates cached
mass for the next ship step. Door animation/furniture/handling do not change
containment. Existing physics visits and anchored frame writebacks provide pose
events when poses actually change. Identical poses emit nothing; runtime
reclassifies retained centers only when the containment frame changes (#306). Explicit scene reconciliation is available for validation. Full inertial
cabin motion, arbitrary future physical items and Core intake are separate tasks.

## Spherical mining domain contract (#230)

`spherical_mining.rs` owns validated per-tool capacity/rate, attached reserved
mass, deterministic bounded ticks and typed releases. It does not allocate
physical IDs, place objects or simulate contacts. Session/runtime integration
is #231; legacy extraction fixtures remain available. The canonical
[lifecycle and radius rules](resource-contracts.md#spherical-mining-lifecycle-230)
separate material conservation from replaceable visual geometry.

## Fragment identity lookup (#305)

The fragment Vec remains canonical creation/solver order. A private ID-to-slot
hash index serves lookup/writeback and is never iterated for gameplay ordering.
Growth preserves slots; ordered deletion repairs shifted slots rather than
swapping entities. Removal is O(retained count), lookup is expected O(1), and IDs
are monotonic across consumption. Range selection still visits retained entities;
CPU attribution in the #305/#306 reports determines whether another spatial
membership structure is justified. Streaming does not remove retained identities.


## Consumable raw resources (#337)

See the canonical [resource catalog and compatibility contract](../world/resource-contracts.md#consumable-raw-resources-337) for new feedstock generation, physical lifecycle, distinct presentation and Core yields.

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

`WorldObjectId::Fixture` shares the existing CarrySlot with fragments. External fixture owners persist/validate the fixture and reconstruct the slot during whole-session restore. See [the canonical contract](../../docs/physical-mounts.md).


Instant Core upgrades #548: [canonical input, source, mass and schema-14 contract](../../docs/core-upgrades.md). Ship owns validated capacity-only transactions; world returns original caller-owned reserved entities through the one carry slot; runtime maps the data catalog/physical controls and reconstructs pending+installed material mass. No timer, joule grant, Core replacement or flight command.


Construction receipts/refunds #543: [canonical transaction, mass, controls and schema-15 contract](../../docs/construction-materials.md). Funding respects the explicit economy approval gate; progress/geometry retain #544/#545 owners.


## Construction damage (#549)

Follow the [canonical damage, physical salvage and schema-21 contract](../../docs/construction-damage.md). Surviving work preserves paid time and receipts; lost cells and destroyed projects cannot respawn or duplicate salvage. Core identity is protected. Fresh debris IDs, exact material shares, collision/containment and installed mass publish together; restore reconstructs source without reallocating debris.


## Canonical terrain streaming (#424)

See the [canonical terrain contract](../../docs/universe-terrain.md) for the exact U11
evaluator, separate frozen identity/codec, planet-wide quadtree, stitching and
CPU/GPU bounds. Compressed six-body fixtures remain unchanged. Gameplay adoption
and normal alien-world startup belong to #425/#426.


## Canonical surface gameplay and schema 24 (#425)

Follow the [canonical surface contract](../../docs/universe-gameplay.md):
selected body/terrain context, immutable U11 deposits, shared physical ground,
conserved fragment/cargo/Fabricator ownership and exact U05 definition binding.
Historical fixtures remain explicit; new-run startup is #426.


## Canonical geology (#612)

World `terrain::geology` owns versioned terrain-bound province/history/substrate
queries. The returned physical sample is the exact existing terrain; geology
never displaces ground or reshuffles resource identities. See the
[geology contract](../../docs/planetary-geology.md) for codec, profiles, compatibility and tests.
Vesper remains a homogeneous silicate interpretation with unchanged published
terrain/resources; rendering and later environmental systems retain their owners.


## Regional climate and weather (#615)

Follow the [climate contract](../../docs/planetary-climate.md) for immutable geology-bound climate inputs,
strict versioned publication and stateless TT epoch queries. World owns pressure,
temperature, coherent storm/cloud/precipitation metadata and body-fixed tangent wind;
presentation cannot rewrite them. Vacuum has no airborne effects. Existing terrain,
resource identities, saves and drag physics remain unchanged. Weather visuals are #616.


## Active geological sources (#617)

The [active geology contract](../../docs/active-geology.md) owns explicit heat/mechanism inputs,
stable terrain-bound sites, strict publication restore and shared-TT source levels.
Spatial queries cannot create/rekey sources. Vacuum permits ballistic plumes and
contact coupling, never atmospheric wind/sound. Source radii do not add colliders.


## Rare biospheres and ecological regions (#618)

Follow the [ecology contract](../../docs/planetary-ecology.md) for independent fixed life decisions,
geology/climate-bound potential, continuous realms and stable patch/grammar IDs.
Suitability never grants life; observed bodies retain no procedural life claim
and Vesper stays sterile. Strict catalog/save bindings reject incompatible
publications. Organism meshes, physical instances and harvesting remain #619–#621.
