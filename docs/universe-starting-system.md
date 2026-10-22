# Permanent awakening system — U11 (#412)

The portable offline publication freezes **Nacre Reach**, its star **Nacre** and
starting planet **Vesper**. Scientific-style IDs remain separate from names:

| Object | Permanent ID |
| --- | --- |
| System | `salimon-fiction:awakening.s000000` |
| Star | `salimon-fiction:awakening.s000000.star` |
| Starting planet | `salimon-fiction:awakening.s000000.p00` |

Owner: `client/universe/src/starting_system.rs`; regressions:
`client/universe/src/starting_system/tests.rs`. Read the owning
[architecture](../client/universe/architecture.md#permanent-starting-world--u11-412)
and [invariants](../client/universe/invariants.md),
[approved expansion](universe-expansion-approved-design.md) and
[canonical generation policy](canonical-planet-generation.md).

## Reproducible selection and freeze

The committed [candidate request](../client/universe/fixtures/starting-candidates-v1.json)
uses U10 builder v1, seed 412, a single 2 pc cuboid local-disk region, residual
density 1 system/pc³, ages 3–8 Gyr and [Fe/H] -0.2–0.3. It publishes eight
fictional systems and 26 planets from an explicitly empty retained baseline
`awakening-empty-v1`. This is a small **authored candidate cohort**, not a measured
residual-density estimate or a production Gaia catalog. U12 must retain this
cohort and reconcile/merge production observations without regenerating its IDs.

The audit screens individual planets in stable catalog-ID order for distance
>=30 ly, radius 2000–9000 km, density 3500–7000 kg/m³, gravity 2–12 m/s² and model
equilibrium temperature 120–280 K. Five qualify; the first is the fixed Vesper
ID. These thresholds select an accessible cold solid surface, not Earthlike
habitability. U10 already screens stellar lifetime, surface/Roche limits,
sparse planetary Hill separation and admissible moons. The whole cohort and
chosen body remain published; selection does not delete the other systems.

`freeze` admits only the exact full U10 publication SHA-256
`294c3687e044dcd644937b0116b28694f9357be2c5489acc73bb14c58ff8c431`,
the fixed chosen IDs and the exact world digest below. A change rejects rather
than silently choosing another planet. New runs/installations do not choose
seeds or reselect worlds. The builder is an offline tool, not a new-run API.

## Distance and physical plausibility

The publication uses a shared **authored** U02 galactic frame: solar barycenter
at cells `[230000000,0,0]`, zero offset, approximately 8.2 kpc galactocentric
radius. The request's region is centered 12 pc farther along its x axis. This
is not a Gaia fit or precision Solar astrometry. At this publication reference:

- Vesper's system is **37.55006266702156 ly** from that solar anchor.
- Subtract a conservative **2 AU Earth/SSB envelope**, the entire starting
  planet's apocenter and an extra kilometre for numerical rounding. The triangle
  inequality gives **37.55002420841507 ly minimum Earth–Vesper separation**,
  independent of planetary phase. Split-cell subtraction precedes conversion to
  SI distance; this is never a GPU/global-f32 calculation.
- Distance is a publication-reference proof. U14 owns actual epoch, velocity,
  rotation/orientation and evolving states; this is not an eternal separation
  bound or a present-date observed Earth ephemeris.

| Selected property | Frozen value / interpretation |
| --- | --- |
| Host | 0.42 solar masses, 347553315 m radius, main-sequence heuristic |
| Age / metallicity | 4.4568593 Gyr / +0.12837655 dex |
| Planet mass / radius | 5.639346998907795e23 kg / 2985359 m |
| Bulk density / gravity | 5060 kg/m³ / 4.223197713467944 m/s² |
| Circular orbit | 0.432163691171445 AU; 13834173.738447988 s period |
| Thermal estimate | 162.63457107015805 K, coarse `L=M^4`, Bond albedo 0.3, full redistribution |
| Surface | Authored airless frozen mineral world; zero surface pressure |

Mass/radius/age/orbit/seeds preserve U10's actual recipe. Independent unit checks
cover density, gravity, Kepler period, Roche clearance and stellar lifetime.
This is scientifically screened fiction, not climate modeling, a calibrated
stellar population, long-term N-body stability, breathable air or a safe unsuited
environment. Ice/feedstock outcrops are authored shallow deposits; a mean thermal
estimate does not establish exposed-ice stability, biospheres or pharmacology.
The existing organic/medicinal material keys are provisional gameplay feedstocks.
Oxygen and thermal gameplay integration remain downstream; the approved
temporary unlimited suit oxygen policy is not implemented by this leaf module.

## Full starting-world recipe and deterministic surface

The [frozen manifest](../client/universe/fixtures/starting-world-v1.json) contains
versions, provenance, scientific IDs/names, the candidate/catalog/actual source
recipe digests, physical inputs, independent U10 seed lanes, full terrain,
resource and visual parameters, arrival and walk paths. Its canonical compact
JSON SHA-256 (excluding the file's final newline) is
`e2bfc551786763078fbe8f48dff0cd71813da7786eb39a6906ba19541fd87bef`.
`FrozenStart` exposes immutable shared references and validated surface queries.
Bounded decode (64 KiB) rejects unknown/duplicate fields, altered versions,
parameters, names, hashes or seed maps. Canonical reserialization must match the
exact permanent manifest, including floating signed-zero representation.

This world digest is **distinct from U10's source-body-recipe digest**. U01 catalog
body definitions still reference those unchanged U10 recipes. U11 supplies the
expanded selected-world sidecar and subsystem digests; U12 must package and bind
both. Other fictional bodies still have generation recipes, not full surface
definitions. No schema-1 reinterpretation or terrain-v1 codec change occurs.

Terrain algorithm 1 has one authoritative radial-height query in body-local
directions. Normalize finite nonzero vectors with overflow/underflow protection.
Let `c` be chord distance from the north pole. Relief is zero within 80 m; over
the next 500 m apply smoothstep to the normalized chord distance. Outside that
cap, relief is a continuous folded triangular ridge wave with amplitude 1800 m,
frequency `3 + terrain_seed % 5`, and phase from the seed's upper 32 bits. The
wave coordinate is `(x + 0.5*z)*frequency + phase` for normalized direction.
Multiplying this wave by the smoothstep gives bounded nonnegative height.
Pure arithmetic/square roots avoid platform-dependent publication `powf`.
`surface_point` evaluates this same query on a bounded 1 km north-pole tangent
chart; there is no separate renderer or collision height field.

Visual algorithm 1 specifies teal ground `[37,82,87]`, amber ridges `[184,109,49]`
and pale ice `[181,222,229]` in sRGB. These are authored palette overrides, not
measured spectra or a screenshot. Their retained independent seed does not
silently randomize the overrides. Visual edits cannot alter surface/deposits.
Resources similarly retain their U10 seed but v1's explicit authored sites take
precedence over procedural placement. Resource anchoring explicitly depends on
the terrain domain; its compatibility key includes that dependency and the full
world digest. No geology/biome/ecology/discovery subsystem is invented here.

## Arrival and starting resources

Body-local north-pole tangent x/z are meters. Ship center `[0,0]` has a 12 m
clearance disk; traveler `[14,0]` starts outside it. The 80 m cap remains a smooth
sphere of the frozen physical radius, not a render-only flattened surface.

| Material key | Tangent x/z | Original mass | Intermediate approach point |
| --- | --- | --- | --- |
| `iron-ore` | -28,18 | 500 kg | 12,12 |
| `silicate-rock` | 28,18 | 500 kg | direct |
| `water-ice` | 0,35 | 500 kg | direct |
| `organic-biomass` | -24,-28 | 200 kg | 12,-12 |
| `medicinal-compounds` | 24,-28 | 100 kg | direct |

Sites have stable semantic `salimon-start:awakening.p00.<material-site>` IDs,
independent of ordering, display names, palette and depletion. Arrival validation
checks every walk segment analytically against the ship disk, cap and <=55 m
total path. Tests independently sample the paths and common surface. All five
existing mining/Core/fabricator material keys are represented, without changing
their recipes, yields, resource codec or mutable depletion journals. These are
fixed **baseline** resource definitions; U24 #425 must adapt them to real-scale
mining, ship hull/airlock geometry, carrying and persistence before play.

## Commands, checks and remaining boundaries

```sh
cargo run -p salimon-universe --example starting_system --locked
cargo run -p salimon-universe --example starting_system --locked -- --audit
cargo test -p salimon-universe --locked starting_system
```

The first command emits canonical manifest JSON plus newline; `--audit` includes
all 26 assessments, source hashes, subsystem definitions and the manifest.
CI compares repeated Ubuntu output to the committed golden and runs the same
U11 tests on macOS/Windows in debug/release. Retain workspace build/tests,
all-target Clippy/fmt and documentation gates. See [measured report](../reports/issue-412/README.md).

U12 #413 owns production merging/packaging; U14 #415 evolving states; U23 #424
real-scale terrain LOD; U24 #425 playable arrival/resources integration.
Reference hardware/native gates #436–#439 remain separate. There is no runtime
startup, save, scanner/chart, Earth-route, backend or asset change. Renderer/native
appearance, environmental survival and real hull contact are not validated by
this portable definition. Explicit all-player-consistent fictional revision
migration remains mandatory; no implicit update of a visited world is allowed.
