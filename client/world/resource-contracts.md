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
leave state unchanged. Regeneration, streaming storage, mining rates, and actual
fragment creation belong to later issues; this setter alone does not guarantee
conservation across a deposit and independently constructed fragments.

`ResourceFragment` holds a session-unique `FragmentId`, source `DepositId`,
immutable raw material, and mutable validated pose. ID allocation/uniqueness and
matching source material are the responsibility of the future extraction system.
A pose uses absolute `f64` meters and a finite unit quaternion in `[x,y,z,w]`
order. Constructors reject invalid positions/rotations; rotations within `1e-9`
of unit squared length are normalized. No visual scale or resource inventory
count is stored. Carrying ownership, gravity, velocity, cargo containment, and
the permanent one-object carrying invariant will be implemented by their tasks.

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
mass, geometry, and bounds. Modified/depleted-state retention belongs to #50;
visuals belong to #43 and mining belongs to #44. No rendering or mining behavior
is introduced here. Run `cargo test --locked -p salimon-world --test resource_generation`.
