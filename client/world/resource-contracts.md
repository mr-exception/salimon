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
stable key and catalog entry. No per-planet distribution is defined here (#41).

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
