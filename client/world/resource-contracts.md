# Planetary resource contracts

`salimon_world::resources` owns portable raw-material, deposit, and physical
fragment contracts. It has no GPU, window, asset, inventory, or backend dependency.
Runtime systems map these values into presentation and physics objects later.

| Stable material key | Raw material | Effective density (kg/m³) |
| --- | --- | --- |
| `iron-ore` | Iron-bearing ore | 3500 |
| `silicate-rock` | Silicate rock | 2700 |
| `water-ice` | Water ice | 920 |

These are initial gameplay approximations, not purity/chemistry simulations.
Persist or communicate `ResourceId::key()`, never catalog indices or Rust enum
discriminants. Unknown keys return `None`. Catalog entries are immutable; display
names may change without changing identity. Adding a material requires a new
stable key and catalog entry. Per-planet generation inputs live in `salimon_world::resource_distribution`.

`RawMaterial::new` validates positive finite kilograms and derives positive
finite solid volume as mass / density. Fragment volume does not describe a mesh
or bounding box; later physical handling chooses a shape that fits this volume.
Mass, volume, and material identity have no public mutation path.

`DepositId` combines celestial-body identity with a generator-assigned local ID.
Generators must keep IDs deterministic and unique per body. `ResourceDeposit`
stores original material, fixed absolute position, and remaining mass in
`[0, initial mass]`. Its state is derived as untouched, partially mined, or
depleted; depletion retains the ID and original properties. Construction accepts
validated restored state. Local updates cannot increase mass and invalid updates
leave state unchanged. This setter alone does not guarantee
conservation across a deposit and independently constructed fragments.

`ResourceFragment` holds a session-unique `FragmentId`, source `DepositId`,
immutable raw material, and mutable validated pose. ID allocation/uniqueness and
matching source material are the responsibility of the future extraction system.
A pose uses absolute `f64` meters and a finite unit quaternion in `[x,y,z,w]`
order. Constructors reject invalid positions/rotations; rotations within `1e-9`
of unit squared length are normalized. No visual scale or resource inventory
count is stored. `MiningSession` now owns permanent single-object carrying
and guarded loose-entity pose updates. Runtime composes surface/ship transfer and
ship-local support anchors. Cargo containment and item velocity remain separate work.

Run focused contracts with `cargo test --locked -p salimon-world --test resources`.

## Planetary resource distribution

`default_resource_distribution(body_id)` supplies immutable validated profiles
for Mercury, Venus, Earth, Moon, and Mars; Sun is empty. These are gameplay
approximations, not real geological maps. Custom `BodyResourceDistribution`
values may declare zero or more resources, including an empty solid body.

Each `ResourceDistribution` has a stable material ID, positive finite relative
selection weight, nominal surface spacing in meters, and inclusive positive
finite initial deposit mass bounds in kilograms. Weights need not sum to one;
generation must normalize them when selecting materials. Duplicate material
entries are rejected. Actual placement and deposit sampling belong to #42.

`generation_inputs(world_seed, biome)` returns material-key-sorted entries with
per-material seeds. Seeds use explicit little-endian world/body seeds, stable
body/material keys, and fixed FNV-1a wrapping arithmetic; no process hash, clock,
catalog index, or RNG state is used. Reordering entries or adding another
material does not perturb an existing material's seed. Changing parameters
changes the returned input even though its seed stays fixed. The generation
algorithm must use these parameters and its own documented deterministic sampler.

Named `BiomeResourceOverride` lists replace the entire base resource list,
including with an empty list. Unknown or absent biome keys fall back to base
inputs and seeds; recognized biome keys partition seeds. Duplicate or empty/
whitespace-padded biome keys are rejected. No terrain/biome detection is
implemented. This extension point lets later terrain work choose a biome
without redesigning the resource configuration.

Run configuration coverage with
`cargo test --locked -p salimon-world --test resource_distribution`.

## Nearby deterministic deposits

`resource_generation::materialize_nearby_deposits` is the renderer-neutral
materialization boundary for #42. Call with the active body/profile, stable world
seed, optional biome, player position, and positive radius; replace the owned
active list on success. Only centers inside that nearby ball are returned.
Far-space and empty profiles return empty lists. Invalid geometry, mismatched
body/profile, excessive resolution, and excessive candidate work return typed
errors, with no partial result.

Generation version 1 partitions six cube faces into cells at nominal configured
spacing and radially projects seeded, inset cell samples onto the body's sphere.
Cube projection causes modest spacing variation; this is a gameplay distribution,
not terrain or a geological map. Each material gets an independent cell lattice;
normalized relative weights thin candidate cells, while each material's spacing
controls its potential density. Seeds and IDs use explicit FNV-1a arithmetic over
configuration seeds, version, resolution, face, and cell indices; random lanes
use fixed SplitMix64 arithmetic. No clock, process RNG, catalog order, or query
origin affects a deposit. IDs are 64-bit content keys scoped to body; as with any
64-bit hash, they are not a mathematical collision-free encoding of an entire
planet. Changing seed/configuration defines a new generated world.

A conservative cube-face interval calculation visits only cells intersecting the
nearby ball, including poles and face seams. Requests examine at most 65,536 cells
across all resources, and resolution is capped at 100 million cells per face edge;
no full-planet cache or allocation exists. Use smaller nearby radii for dense
custom profiles. `SurfaceDeposit` retains authoritative f64 body-local position,
a validated resource deposit with initial/remaining mass, and a spherical bound
whose volume equals initial mass divided by material density. Absolute coordinates
are formed only by translating the body-local position at the boundary.

Unloading and rematerializing untouched deposits reproduces their IDs, material,
mass, geometry, and bounds. Apply the local world MiningSession journal after generation to retain modified
and depleted state; generation itself remains stateless. Run `cargo test --locked -p salimon-world --test resource_generation`.

## Handheld extraction

`mining` owns aimed target validation and time-based extraction. A normalized
`MiningRay` selects the first nondepleted spherical deposit bound within **4 m**
of the eye ray; missed, behind-eye, out-of-range, and obscured targets are rejected.
Solid body spheres occlude the ray. Runtime supplies the first ship-proxy hit
from the character domain in the same meter units, so hull/closed gates/engines
cannot be mined through. Ship glazing is conservatively solid for this tool.

`extract` removes **2 kg/s × simulation delta**, capped to remaining mass; zero
and depleted extraction return zero. It returns the actual before/after mass
difference; `MiningSession` represents all of that mass as physical fragments.
`MiningSession` owns modified-deposit mass by stable ID and a diagnostic total.
All inspection/presentation queries apply that state. Its lifetime is one local
world/seed configuration. The sparse journal retains only changed deposit IDs
and remaining kilograms, including zero-mass tombstones. Dropping active lists,
stowing the tool, or rebuilding the renderer never clears it. Restoration is
idempotent and only decreases mass; extraction reconciles fresh/stale copies
before emitting fragments, preventing re-extraction of already removed mass.
Untouched queries need no journal entry. Starting a new world creates a new
session; there is no disk, backend, or cross-session persistence. Runtime hosts
the session but does not own the extraction rules. See `resource_streaming`
tests for generated untouched/partial/depleted deposits across repeated cycles
on all five solid bodies and stale-copy conservation.

Run `cargo test --locked -p salimon-world --test mining` for aim, obstruction,
large-coordinate, extraction timing, and depletion checks. The native
`scenarios/mining.json` follows the real surface walkthrough and tool inputs;
its evidence variant adds screenshot checkpoints.

## Physical extraction output (#45)

`resource_fragments` owns bounded splitting and deterministic emergence near the deposit.
Each source deposit's most recent piece grows to **2 kg**, then a new piece is
created. A fractional piece exists immediately, including on the first positive
simulation step; there is no pending output mass or general resource inventory.
Piece IDs are monotonically allocated within the session. Growth preserves ID,
source, material key, and pose, and derives the new solid volume from density.
Finished pieces are unchanged by later extraction. Picking up a growing piece
must end its participation in the output tail when #46 adds carrying.

Pieces have finite absolute poses and identity orientation. They emerge near
the source with enough height for a full-size piece. Side length is the cube
root of solid volume. Runtime applies an ejection impulse, gravity, and
fragment contacts while the world crate owns mass, identity, and provenance.
Runtime maps the same authoritative pieces into distinct material silhouettes
and nearby-entity inspection.
The session retains pieces independently of deposit materialization and tool
equipment; querying or re-rendering never emits additional pieces.

Run `cargo test --locked -p salimon-world --test resource_fragments` for material
identity, unique IDs, partial growth, conservation at different timesteps, zero
output, large-step splitting, depletion, and requery behavior. The mining E2E
route now checks piece mass/volume/pose and the complete depleted output.
