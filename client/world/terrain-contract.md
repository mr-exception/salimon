# Planetary terrain contract (version 1)

## Status and activation

The version-1 definition/query/codec is integrated into native gameplay by
#179–#181. TerrainMode::Representative supplies the same immutable definitions
to rendering, walking, flight/landing, resource placement and fragment support.
Normal startup uses a validated safe Earth site; existing flat scenarios and
the F2 precision camera retain their old spherical fixtures. Mercury/Venus
remain zero-elevation and Sun stays visual-only. Read the canonical
[integration/validation guide](../../docs/terrain-validation.md) for ownership,
slope/landing policy, patch budgets, approximations and native commands.
The original contract-first validation is historical in
[the stage-1 report](../../reports/issue-170/README.md).

## Definition, coordinates and evaluation

A validated immutable `TerrainDefinition` contains stable body identity,
generator version, u64 seed, baseline radius, up to eight ordered noise layers
and 32 ordered sparse authored features. Center/orbit and visual material do not
belong in this definition. Inputs and outputs use body-local f64 meters. Current
bodies do not rotate; any future rotation must be applied by the frame owner
before/after querying. No player-relative seed or mutable RNG is used.

### Future canonical multi-seed planet definitions (documentation only)

The approved future shared-world contract is [canonical planetary generation](../../docs/canonical-planet-generation.md): each planet's **published** record has independent versioned domain seeds for terrain, resources, visual appearance and future geology/biomes/discoveries, alongside a stable body ID, full-definition hashes and authored/scientific inputs. The current version-1 `TerrainDefinition` **already** has a deterministic `u64` terrain seed and generator version; keep its codec, numerical rules, representative fixture seeds and cache identity unchanged. Do not insert new fields into the version-1 binary envelope or assume it can represent arbitrary future galactic IDs. A future versioned catalog/terrain adapter must make any new domain seeds and body identity explicit, without silently reinterpreting visited worlds or altering resource IDs. Seeded on-demand evaluation is allowed; client/session/clock-driven selection of permanent terrain is not.

`height(unit_direction)` returns signed elevation above nominal radius.
`sample(unit_direction)` also returns body-local surface position, outward unit
normal and dimensionless slope (`tan` of the angle to radial up). Input directions
must be finite and unit length within 1e-12; callers normalize their body-local
position before querying. Do not pass absolute universe coordinates into noise.
Translate sampled positions in f64 and subtract the camera in f64 before casting.

Version 1 samples a Cartesian 3D lattice through the sphere. Explicit wrapping
u64 hashing and smooth trilinear interpolation generate hills; a folded value
noise layer generates ridges. Layer wavelength is lattice spacing, not a strict
sinusoidal period. Ordered amplitudes/layers implement multiscale roughness.
There are no face/pole parameterization seams: the same direction is the same
sample regardless of chunk key. Signed compact-support smooth features implement
pits/craters or hills. Feature radius is body-local chord distance, not geodesic
arc length. Craters are depressions with no automatic raised rim; add a separate
positive feature if needed. Overlapping features add in definition order.

Normals use a fixed 0.25 m central-difference stencil and actual radial scale.
The reported slope is measurement, not a traversal limit. Ridge cusps use that
finite-difference approximation. Mesh/contact approximation tolerances and
walking/step/jump/landing limits belong to the dependent integration issues.
Material/biome masks and erosion are not shipped controls in version 1; adding
new evaluation semantics requires a new generator version.

All layer amplitudes are nonnegative; authored elevations may be negative.
Version 1 supports nominal solid-body radii within [1 m, 1e9 m].
Lengths are finite and within [1 m, body radius], with lattice coordinates capped
at 1e9. Individual and conservatively summed extrema stay within ±0.01R.
`elevation_bounds_meters` sums amplitude envelopes and signed feature extrema,
so it bounds every direction without tessellating the planet. The positive bound
is inside the existing 0.15R landing shell. This preserves catalog layout but
consumers must still update surface-distance and obstruction queries at stage 3.
Invalid configurations, NaNs, nonunit directions, visual-only bodies, unsupported
versions and over-budget definitions return typed errors.

## Compact wire format and cache identity

No networking, disk persistence or serialization dependency is added. `to_bytes`
and `from_bytes(known_body, bytes)` are a portable bounded binary codec. Decode
checks body ID and exact radius against the caller's known body; no remote center
can silently change catalog layout. All numeric fields use little-endian bytes;
floats are IEEE f64. Counts/total length are checked before allocation or reads.

| Field | Bytes |
| --- | ---: |
| ASCII `SLMTERR1` | 8 |
| Generator version (`u16`, currently 1) | 2 |
| Body tag (Sun=0, Mercury=1, Venus=2, Earth=3, Moon=4, Mars=5) | 1 |
| Seed (`u64`) | 8 |
| Nominal radius (`f64`, meters) | 8 |
| Layer count, feature count (`u8` each) | 2 |
| Reserved, must be zero | 3 |
| Each layer: kind (`u8`, hills=0/ridges=1), amplitude, wavelength (`f64`) | 17 |
| Each feature: unit direction xyz, chord radius, elevation (five `f64`) | 40 |

The maximum record is **1,448 bytes**, with 32-byte empty definitions.
Earth/Moon/Mars representative records are **106 bytes** each. Example:

```rust
use salimon_world::{CELESTIAL_BODIES, CelestialBodyId, terrain::TerrainDefinition};
let earth = *CELESTIAL_BODIES.iter().find(|b| b.id == CelestialBodyId::Earth).unwrap();
let definition = TerrainDefinition::representative(earth).unwrap();
let payload = definition.to_bytes(); // 106 bytes; seed 17001, radius 6,000,000 m
let restored = TerrainDefinition::from_bytes(earth, &payload).unwrap();
let ground = restored.sample([0.0, 0.0, 1.0]).unwrap();
```

Earth: 35 m hills at 500 m spacing, 140 m ridges at 2,000 m spacing and a
-40 m/180 m pit at +Z. Moon uses 8/25/-100 m; Mars uses 25/220/-65 m.
Seeds are 17001/17002/17003 respectively. These are representative profiles, not final biome art direction. Runtime uses
a bounded footprint-aware safe-site search before activating a spawn; the old
+Z pit fixture must not be used as an unchecked landing endpoint.

Use the entire encoded definition as cache identity, or a collision-resistant
content digest of it, together with body identity and patch coordinates/LOD.
A seed alone is insufficient. Version/config changes invalidate mesh caches and
require an explicit decision about existing resource/session positions; never
reinterpret old definitions or session state under new terrain silently.
Unknown versions fail rather than falling back. The wire magic is an envelope
version distinct from the generator version. Preserve version-1 evaluation
when adding a version-2 evaluator or explicitly reject old worlds.

Same executable/input reproduces identical samples, independently of evaluation
order/thread/translation. Floating-point interpolation/normalization can differ
in last bits across toolchains/architectures; compatibility vectors must stay
within 1e-7 m in height and 1e-6 normal components. This contract does not promise
bit-identical floating-point normals across all machines. For platform
verification, manually run the versioned golden vectors on macOS/Windows/Linux.
PR CI only compiles source. The stage-1 report verified Linux, and the
integrated reports record native arm64 macOS validation separately.
A future server sends this tiny definition (or its hash plus a known definition),
not per-vertex meshes. Sparse feature changes replace/version the record.

## Integrated rendering and physical boundaries

Runtime generates bounded cube-sphere quadtree patches and renderer-neutral DTOs;
renderer owns GPU buffers and near/far presentation. Character and physics
receive caller-supplied queries; ship consumes TerrainMode and an authored
footprint supplied by runtime. Runtime anchors/orients resource meshes without
changing their original generation IDs or the in-memory journal. See the
[integration guide](../../docs/terrain-validation.md) for the quadtree/clipmap
decision, exact budgets, mixed-LOD stitching, subtract-before-cast anchors,
physical policies, measured tolerances and distant-envelope approximation.

World queries allocate no planet geometry and retain no cache. Height cost is
bounded by eight layers × eight lattice corners plus 32 sparse features; a full
normal sample evaluates five heights. Runtime caches are separate and bounded.
Native/reference evidence is in reports/issue-179 through issue-181; stage-1
Linux-only evidence is not evidence for the integrated M1 implementation.

Caves, tunnels, overhangs and holes through rock require volumetric geometry;
they remain deferred until after Phase 0, with a separate design and
implementation phase. No placeholder cave mechanics are introduced.
