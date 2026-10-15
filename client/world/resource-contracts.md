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
or bounding box; gameplay geometry applies the separate shared size policy.
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
Flat fixtures use solid-body sphere obstruction; enabled terrain uses the
shared bounded short-ray height query, including negative elevation. Runtime supplies the first ship-proxy hit
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

## Spherical mining lifecycle (#230)

`spherical_mining` owns the accepted portable lifecycle. `MiningConfig` defaults
to **10 kg** capacity and **2 kg/s**, configurable per tool (finite cap/rate in
[1e-6, 10,000], no upgrade UX). Runtime supplies held left mouse and the current
valid aimed deposit; the domain receives no key, window, pose, GPU or energy.
It admits at most one **16 ms** simulation tick per call. Runtime must drain its
fixed-step clock backlog; the tool never banks elapsed work or loops to create
unbounded output after a large delta.

| Input / boundary | Contract |
| --- | --- |
| Held + valid target | Reserve positive mass in one attached sphere; its source/material remain immutable. |
| Mouse up | Release immediately before further extraction, even on a cap/depletion tick. |
| Aim lost/obstructed/out of range | Release immediately. Reacquiring with continued hold starts a new sphere. |
| Target switch | Release previous sphere without debiting the new deposit on that tick; start next tick if valid/held. Never merge materials. |
| Capacity | Release automatically. Continued hold starts a fresh sphere next tick, without requiring another button edge. |
| Deposit depleted | Release final smaller sphere, retaining every representable positive mass; no zero-mass object. |
| Unequip/focus/cursor/view/lifecycle interruption | Explicit interrupt releases; no post-resume hidden growth. |
| Zero duration | No growth; mouse-up/aim-loss/interrupt still release. Explicit pause interrupts rather than accumulating catch-up work. |
| Player movement | Runtime updates attachment pose from current tool front-left every tick. Leaving valid aim/range releases at last valid pose. |

Debit only representable before/after deposit differences, bounded by rate,
remaining capacity and source availability. Subtraction rounding never consumes
more than requested. Rounding-only cap gaps (<=1e-12 kg) can seal a piece without
debiting the gap. An unrepresentable debit consumes nothing. Conservation:
removed deposit mass = sum of released mass + currently reserved attached mass
(with f64 accounting tolerance, 1e-9 kg in regressions). No hidden pending mass,
energy cost, abstract inventory or mass discarded for visual convenience.

Radius is `cbrt(3 * mass / (4π * effectiveDensity))`, clamped to **0.005–2 m** for
presentation/support only, never changing mass/solid volume. Material keys and
source `DepositId` remain those of the deposit. A released physical fragment
keeps its identity for movement, one-object carrying and mechanical Core intake;
reacquiring aim never reconnects it. No disk save exists: generation IDs, session
journal and depleted tombstones remain in-memory source of truth.

Implementation sequencing: #230 introduces and tests these domain contracts;
#231 wires session IDs, input and native fixed ticks. The existing `extract` /
`extract_with_spawn` helpers remain a legacy bounded 2 kg output path for saved
fixtures/benchmarks during migration; they are not the new continuous tool
lifecycle. Rendering/contact replacement has separate spherical-mining follow-ups.
Use `cargo test -p salimon-world --locked --test spherical_mining` for the new
lifecycle; existing fragment/streaming tests protect retained legacy APIs.

## Carrying and equipment exclusivity (#131)

The session's carried identity remains the sole authority for the permanent
one-world-object limit. Runtime clears equipment only after `pick_up` succeeds;
a rejected pickup preserves selection. While carrying, all five slots remain
visible and deselected and mining cannot stay active or render a held tool.
Dropping restores neither the previous slot nor a held mining input; selecting
1–5 explicitly becomes available immediately. This affects no material, mass,
fragment identity or persistence rule. See the
[runtime regression contract](../runtime/architecture.md#carrying-and-equipment-regression-contract-131).

## Gameplay size policy (#144)

`resource_size.rs` is the single size authority: deposits use `RESOURCE_LINEAR_SCALE = 10`; fragments use `FRAGMENT_LINEAR_SCALE = 5` after #148.
Solid volume is still `mass / density`; fragment gameplay side is
`5 * cbrt(solid volume)` and deposit gameplay radius is
`10 * cbrt(3 * initial solid volume / (4π))`. All authored variants keep the
same vertices; fragments now measure half the previous linear dimensions at equal mass, including growing pieces and all six variants. Deposit dimensions are unchanged.
This is a gameplay exaggeration, not a density change; mass labels, 2 kg/s yield,
legacy 2 kg fragment helper cap and conservation remain unchanged. New spherical lifecycle sizing is defined above and does not use the cube exaggeration.

Growing fragments scale with current mass; picking up seals growth. Deposits
retain initial bounds and placement while partly mined, then disappear at zero
remaining mass. Runtime raises each deposit along the body normal by its actual
variant vertex support plus 5 mm. This deterministic placement is independent
of remaining mass and is restored alongside the same ID/variant on requery.
Generation query membership still uses the original surface anchors; size does
not change seed arithmetic or distribution spacing (25–80 m).

Native emission reserves the full growing fragment envelope at an exposed authored facet;
there are no spawn lanes or ordinal position offsets. Runtime supplies a circumscribed cube sphere for
broad-phase/ship containment only. Authored convex envelopes, rotated by the physical pose, define pair and terrain/deck contacts. World pickup intersects the oriented conservative cube; mining and prompts
consume the enlarged deposit sphere. Carried centers stay horizontally ahead of the player capsule by the cube
sphere radius plus the hand offset at every pitch, and clamp to ground support. Unsafe hull/furniture floor drops retain
the carried identity. Loose fragments remain supported in ship-local metres.

## Legacy facet emission (#150)

`MiningSession::extract_with_spawn` invokes the caller only for a new FragmentId,
passing its initial material and current ordered physical pieces. A validated
pose accepts that piece; `None` stops this step's emission before consuming its
mass or identity. Growth of the unsealed tail skips placement and preserves pose.
Already accepted pieces/growth are retained if a later piece is blocked in a
large step. No output mass is buffered: only accepted physical mass is debited
from the restored deposit and credited to the diagnostic total. The rate remains
2 kg/s when emission is clear; blocked time is not banked. Geometry-free
`extract` uses the radial top of the conservative bound, without rows/grids.

Runtime chooses exposed eye-facing triangles from the immutable authored mesh,
first the mining ray contact, then actual facet centers ordered by distance to
the aimed visible triangle contact (bound contact on a visual miss). It reserves the selected fragment variant's full 2 kg
convex support along the facet normal plus 5 mm; ground support is checked with
the shared sampled terrain point/normal and full authored vertices when enabled. Conservative player-capsule and existing-fragment envelope
checks reject unsafe candidates. Only creation installs an outward/radial impulse;
growth, requery, streaming and missing/restored motion do not re-eject pieces.
The same physical ID/pose then follows normal gravity/contact, carrying and
streaming. No scatter positions are assigned by ordinal or teleported later.

## Terrain anchoring (#180)

Runtime preserves the original seeded nominal lattice and IDs, then orients and
supports authored deposits/held pieces and new emissions on the same terrain
definition as walking, landing and loose contact. Cached placement does not own
mining state; journal restoration always follows materialization. See the
[canonical integration guide](../../docs/terrain-validation.md) for cache keys,
clearances and supported approximation limits.

## Mechanical Core intake and conversion (#189)

Normal E targeting within 3 m of the authored Core center, aimed within 30°,
accepts the one carried fragment indoors, including blackout/zero-G. No
cockpit authority or electricity is required. Ship validates finite mass ×
material yield before removing the stable identity from the world session;
runtime immediately drains removal events into the mass ledger and clears
physical/presentation caches. Busy intake leaves the next fragment untouched.

Ship owns a single automatic process, 200 MJ/s by default. Catalog gameplay
yields default to 1 GJ/kg iron, 0.5 GJ/kg silicate, 2 GJ/kg ice; these are tuning
values, not chemistry. `ConversionConfig` permits finite nonnegative yields
and a positive finite rate. Zero yield consumes immediately. Generated joules
settle with ordinary discharge before capacity clipping; excess is wasted and
full batteries keep processing to completion. Updates split at process completion
so later idle time is not incorrectly charged by an already finished fragment.
