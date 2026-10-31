# Canonical fictional systems — U10 (#411)

`salimon-universe::synthesis::build(base, Request)` produces one immutable,
offline publication: a validated schema-1 catalog, stellar position/age/metallicity
sidecars, body/orbit recipes and compact minor-population recipes. Existing
observed records, quantities, datasets, uncertainties and source keys pass through
unchanged. The client does not run the builder or choose seeds on arrival.
This is a bounded production-tool boundary with synthetic validation, not a
published Milky Way pack or calibrated stellar census.

## Inputs, identity and publication

Request version 1 pins seed, explicit new revision, reviewed authoring/science
rationale and keyed, nonoverlapping cuboid regions. Region centers use U02 split
galactic coordinates; extents are parsecs on those same axes. Ages span 0.01–13.8
Gyr, metallicities [Fe/H] -2.5–0.5 dex, and density is 0–1000 **residual unobserved
single-star systems per cubic parsec**. Callers reconcile observed selection and
spatial completeness first; adding total density on top of Gaia is invalid use.
Geometric proximity cannot cross-identify a real source. No Gaia distance
inference, automatic empirical Milky Way density fit or astrometric replacement
is performed. The synthetic request is not a production authoring approval.

Expected count is cuboid volume times residual density. Deterministic stochastic
rounding adds one system with probability equal to the fractional remainder.
The resulting count is within one of expectation; zero density emits no fiction.
Placements are uniform inside each supplied support. At most 32 regions, 2048
systems and 1000 pc per axis are accepted. Final schema JSON remains <=16 MiB;
full publication <=64 MiB. These independent bounds can reject before the system
limit. U12 must batch large catalogs and package sidecars; no GB-scale memory,
throughput or 2–5 GB installation forecast follows from the small workload.

Reserved namespace `salimon-fiction` never shares observed IDs. Region key
(1–32 lowercase ASCII letters/digits/hyphens), permanent local system slot and
semantic child key form identity, e.g. `salimon-fiction:disk.s000000.p00.m00`.
Input order, other regions, display names and installation state do not rekey
records. Increasing density retains the existing slot prefix. Changing seed or
model can change topology and content and requires an explicit new publication,
never an implicit migration. Any base record **or dataset** in the reserved
namespace rejects the entire build; regeneration against a previously generated
catalog is deliberately refused. Rebuild from the retained source baseline.

SHA-256 lanes hash seed LE-u64 followed by length-prefixed UTF8 builder version,
exact key and named field. Seeds and domain names are independent of consumption
order. Samples use exact 32-bit dyadic interior fractions. Basic IEEE arithmetic,
integer-metre radii, an integer search for stellar power-law radius and square
roots avoid platform `powf`/`cbrt` in publication generation. Cubes use the explicit
ordered product `(x*x)*x`; `f64::powi` is excluded because its precision can vary
by platform even for integer exponents. Canonical struct
field order, sorted regions/sources/catalog records and BTreeMap recipe ordering
produce deterministic JSON. The pinned sha2 dependency supplies actual SHA-256;
hashes are not CRCs or language hash values. Linux repeated/reordered checks and
macOS/Windows debug and release portable CI golden tests verify the contract; native game evidence
is separate. A changed golden must be reviewed as a version/publication change.

Every generated record and mass/radius uses explicit `Fiction` builder, revision
and rationale; uncertainties remain unknown. Catalog and complete publication
hashes cover distinct byte sequences; the latter includes normalized request,
input-catalog hash, catalog and all sidecars. Body definition references hash the
actual serialized body recipe; population references hash the actual population
recipe. Terrain/resources/visual domains hash their reserved version-1 input
tuples `(builder, object ID, domain name, seed, radius, density)` and have no
implicit dependencies. The selected world now has a full U11 definition/query;
other worlds still require full physical/terrain definitions before evaluation.
These U10 inputs are **generation recipe references**,
not complete landable worlds, meshes or a new terrain codec. U01/U04 wire is
unchanged. Publication JSON is bounded builder output with no promised production
decoder; U12 owns the final pack/version/checksum protocol.

## Version-1 scientific heuristics

This intentionally narrow model covers single main-sequence stars, circular
planetary systems and simple spherical bodies. It excludes binaries, remnants,
giants, resonances, calibrated formation histories and N-body evolution. It
does not claim a statistically fitted initial mass function or long-term orbital
stability. Motivation follows the U09 [stellar/planetary references](universe-inference.md#version-1-proposal-distributions)
and its constants: SI solar mass/radius, AU, Julian year and gravitational constant.
Fiction synthesis is separate from U09 source-backed missing-data completion;
it never fabricates an observed dataset to bypass that boundary.

| Feature | Frozen recipe and physical screen |
| --- | --- |
| Main-sequence stars | Coarse low-mass-dominated proxy: 80% uniform 0.1–0.5, 18% 0.5–1, 2% 1–2 solar masses, in integer milli-solar units; age is uniform within supplied bounds. Decrement any mass above its `10 Gyr / mass^2.5` main-sequence lifetime until admissible. This produces a truncation pileup, not a fitted surviving-star distribution. Radius is solar radius times mass^0.8, floored to integer meters. |
| Metallicity-conditioned planets | Richness `([Fe/H]+2.5)/3`; 1–3 uniform planets plus one Bernoulli(richness) planet. At slots >=2, giant probability is `0.05+0.6*richness`; inner slots are rocky. |
| Mass and radius | Rocky radius 2–9 million m/density 3500–7000 kg/m³; giant radius 50–85 million m/density 500–2000 kg/m³. Round radius/density to integer units; mass is density times spherical volume. |
| Planet orbits | Axis `0.4 AU * 6^slot * uniform(1,1.1)`, zero eccentricity; Kepler period includes both masses. Widely separated orbits meet the >3.5 mutual-Hill gap screen for admitted masses; tests check the actual values. |
| Major moons | One per planet when admissible; radius 8% of rocky or 2.5% of giant radius, density 3300 kg/m³, circular axis 10 planet radii. Orbit must be inside 0.2 Hill radius and outside surface/fluid Roche limits. |
| Important small body | One individually stored 250 km-radius, 2200 kg/m³ belt object at 150 AU per system. This is a selection policy, not a claim that every large real asteroid has been imported. |
| Aggregate asteroid belt | Total estimate 100000 members, minus the one selected body: 99999 unmaterialized. Radial support 140–180 AU, radii 1–250000 m, density 2200 kg/m³. |
| Aggregate comet cloud | 1000000 unmaterialized, 1000–10000 AU, radii 1–50000 m, density 600 kg/m³. Both aggregates retain a seed, uniform radial support and differential radius number proportional to radius^-3.5; no individual member IDs or simulation is allocated. |

All individual circular orbits must clear parent radius plus body radius and
fluid Roche bound. Moon Hill checks use the parent planet's stellar orbit.
Source anchors, age, metallicity and partial Kepler orbits are retained, but
velocity, epoch, phase, orbital orientation and evolving states remain absent.
U14 owns complete state evaluation. Recipes describe a plausible deliberately
sparse fixture family, not the full diversity or age evolution of the Galaxy.

## Commands and validation

From repository root:

```sh
cargo run -p salimon-universe --example fiction_build --locked -- client/universe/fixtures/schema-v1.json client/universe/fixtures/fiction-request-v1.json artifacts/fiction-v1.json
cargo run -p salimon-universe --example fiction_sanity --release --locked
cargo test -p salimon-universe --locked
```

Create the output directory first. The adapter bounds input bytes, validates the
entire publication before writing, syncs a sibling temporary file and renames it
into place; failures preserve prior output. It refuses input/output aliases.
On platforms that refuse replacing an existing destination, choose a new output
filename. Do not automatically promote development output into a game install.

The sanity workload checks four 64-system age/metallicity cohorts, count,
low-mass dominance, position histogram coverage and lifetime constraints. It
emits catalog/full-publication SHA-256, raw/zlib-6 byte counts and distributions,
and compares full repeated/reordered bytes. Unit tests additionally cover real
fixture preservation, SHA/PRF and publication goldens, domain separation, local
identity stability, collisions, invalid/bounded requests, Roche/Hill/Kepler
constraints and 512 fixed-seed systems with expected age/metallicity trends.
These broad heuristic screens are not empirical goodness-of-fit tests.

See [U10 evidence](https://github.com/mr-exception/salimon/blob/d003bd58d506e04e3df97744f29f68449b6e2b3e/reports/issue-411/README.md). U11 now publishes the [selected-world definition](universe-starting-system.md);
other worlds' full definitions,
U12 production batching/packaging, U14 dynamics, source reuse/coverage
#659/#524/#526, and cloud integration #436–#439 remain their existing
scopes. There is no playable migration, scanner revelation, save change, Earth
route disclosure, surface migration or new native performance claim.
