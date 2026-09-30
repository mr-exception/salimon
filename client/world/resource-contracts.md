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
