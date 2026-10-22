# Missing-data inference — U09 (#410)

`salimon-universe::inference` supplies a portable offline completion boundary for
source-backed, explicitly classified single-main-sequence-star planetary systems.
It produces a new immutable catalog revision and typed age, partial orbit and
sampling-evidence sidecars. It does not modify the source catalog or any present
quantity, uncertainty, validity, source key, ID or definition. Missing measurements
are eligible for completion; a nonmissing value is never silently repaired.

## Callers, provenance and compatibility

`complete(catalog, SystemInput, revision)` requires an existing star, an explicit
`StellarModel::MainSequenceV1`, a fixed u64 seed and exactly one `PlanetInput` per
planet in its system. Each planet has an explicit `RockyV1` or `GasGiantV1`
classification, optional SI semimajor axis/period/eccentricity and optional minimum
mass. Optional stellar age is in seconds. Classification is a reviewed input,
not a deduction from a name, Gaia brightness or missing data. Maximum 64 planets;
additional stars, moons, minor bodies and populations in the selected system
reject as unsupported. Other systems in the catalog pass through unchanged.
Pure fictional population synthesis belongs to U10 and requires its own evidence.

Every input scalar and provenance reference is validated. Missing source evidence,
duplicate/omitted planets, unsupported topology and unchanged catalog revision
reject. A completion includes model version `salimon-inference-v1`, seed, accepted
attempt, classifications, stable planet/evidence ordering and a validated catalog. All
inferred quantities use U01 `Provenance::Inferred` with versioned method/field and
the sorted union of registered source dependencies of the selected system and
explicit constraints. This conservative dependency union includes contextual
source evidence, not only the immediate numerical operand. Input source datasets
retain their release versions. No model dataset is presented as an observation.

The U01/U04 schema remains version 1 and stores inferred mass/radius with normal
provenance. Age, partial orbits, classifications and distributions stay in the
separate typed `Completion`; there is no sidecar decoder/production wire promise.
U12 must explicitly package/version that evidence before production use. Do not
serialize only the catalog and claim the full inference recipe has survived.

`complete_observed` strictly adapts U08 `PlanetObservation` quantities. It first
merges measured host/planet mass/radius into a private input catalog and transfers
measured period/axis/eccentricity and minimum-mass constraints. Equal observations
are preserved; conflicting ones require reconciliation. Upper/lower limits and
unspecified limit flags reject, rather than become point measurements or vanish.
Minimum mass stays a lower bound on true mass and never becomes a measured mass.
U08 raw fields, literature, one-sided errors, orientation and source-day epochs
remain in the original observations; callers must retain them. The frozen Proxima
fixture has an unspecified period limit flag: its direct adapter refusal is
intentional. Tests separately exercise a **test-only** explicit flag review while
preserving the original period/errors; this is not production source approval.

## Version-1 proposal distributions

These are broad, scientifically motivated **heuristic priors**, not fitted Gaia
population frequencies, stellar evolution tracks, composition posteriors or
calibrated credible intervals. SI constants are explicit in source. The adopted
solar mass is 1.98847e30 kg and radius 695700000 m; Earth mass 5.9722e24 kg/radius
6371000 m; Jupiter mass 1.89813e27 kg/radius 71492000 m; AU 149597870700 m,
Julian year 365.25 days and G 6.67430e-11 SI.

| Missing parameter | Proposal / conditional relation |
| --- | --- |
| Main-sequence mass with no radius | Log-uniform 0.08–2 solar masses |
| Main-sequence mass with known radius | Log-uniform 0.8–1.2 times `solar_mass * (R/solar_radius)^(1/0.8)`; physical-domain rejection follows |
| Main-sequence radius | Log-uniform 0.8–1.2 times `solar_radius * (M/solar_mass)^0.8` |
| Stellar/system age | Uniform 1 Myr to min(13.8 Gyr, `10 Gyr * (M/solar_mass)^(-2.5)`) |
| Rocky radius when neither mass nor radius exists | Log-uniform 0.3–1.6 Earth radii |
| Rocky mass or radius | Sample log-uniform bulk density 3000–8000 kg/m³, then use `M = density * 4*pi*R^3/3` or its inverse, preserving whichever quantity exists |
| Giant mass / radius | Independently log-uniform 0.1–10 Jupiter masses / 0.7–1.4 Jupiter radii; physical-domain rejection follows |
| Semimajor axis with no period | Log-uniform 0.02–100 AU |
| Axis with period, or period with axis | Kepler two-body relation including planet mass |
| Eccentricity | Uniform 0–0.3 |

`Evidence` records each sampled/derived quantity and its tagged proposal
`Uniform`, `LogUniform` or `Derived` relation. Qualitative confidence is `LowPrior`
for unconditioned draws or `ModelConditional` for conditional/derived values;
neither is a probability. Sampled quantities carry proposal support bounds with
`confidence: None`; derived uncertainty is `Unknown` (no false error propagation).
After joint rejection, these original proposal distributions/supports are **not**
normalized marginals or posterior confidence intervals. Empirical metallicity,
age-dependent density and population fitting are not implied by these recipes.

Motivation/limits: the main-sequence power laws and lifetime scaling are rough
textbook approximations, restricted to low/intermediate mass and generous radius
scatter; see [OpenStax stellar properties](https://openstax.org/books/astronomy-2e/pages/18-2-measuring-stellar-masses)
and [stellar evolution](https://openstax.org/books/astronomy-2e/pages/22-1-evolution-from-the-main-sequence-to-red-giants).
Density/giant ranges deliberately cover broad familiar planet classes, not
sub-Neptune mixtures; compare [NASA planetary facts](https://nssdc.gsfc.nasa.gov/planetary/factsheet/).
Kepler, fluid Roche and mutual-Hill formulas are conservative analytical screens,
not a replacement for an N-body integration or observational model fit.

## Physical acceptance and deterministic refusal

FNV-1a hashes the model version, exact object ID and field with zero separators;
seed and attempt are combined before the SplitMix64 finalizer. A frozen integer
hash golden protects the recipe. Explicit sorting removes input-order effects.
No OS clock, platform RNG, renderer or runtime state participates. Floating-point
transcendentals are repeatable on the tested toolchain; cross-platform bitwise
scientific-pack identity requires U12 publication goldens/verification.

Up to 256 complete-system proposals are tested; the first passing system is
published atomically. Exhaustion returns an error and leaves inputs intact. Tight
constraints can exhaust even when some mathematical solution exists; no fallback
clamping or invented observation is published. Rejection may resample other
missing fields; the accepted attempt is retained. Constraints use point values,
not the entire observational error support, and do not rewrite those errors:

- Main-sequence mass 0.08–2 solar masses; radius within 0.75–1.25 of the power-law
  relation; age within the above lifetime/universe-age window.
- Rocky mass 0.01–15 Earth masses, radius 0.3–1.6 Earth radii and density
  3000–8000 kg/m³; giant mass/radius as above and density 50–30000 kg/m³.
- True planet mass at least any supplied minimum-mass point constraint.
- Bound eccentricity [0,1), with reported bounds also inside that interval;
  positive finite axis/period; point period/axis consistency within 0.1%.
- Pericenter exceeds both stellar-plus-planet radii and the fluid Roche limit
  `2.44 * planet_radius * (star_mass/planet_mass)^(1/3)`.
- Adjacent radial orbital envelopes do not overlap and their gap exceeds
  3.5 mutual Hill radii, with `R_H = ((m1+m2)/(3*M))^(1/3)*(a1+a2)/2`.

This deliberately rejects some real resonant/eccentric systems. It screens
physical plausibility; it **does not prove long-term dynamical stability**.
Missing epoch, orbital phase, inclination/node/periastron orientation never become
fabricated measured states. U14 owns arbitrary-date ephemerides; this API supplies
only partial elements. No Gaia parallax/distance inference is implemented here.

## Validation and remaining integration

`cargo test -p salimon-universe --locked` covers fixed seeds, source preservation,
conditional branches, minimum mass, source limits/conflicts, Kepler's terrestrial
year/inverse, malformed/nonfinite inputs, system coverage/topology, crossing
orbits and 1024 seeded rocky/giant constraint cases.

`cargo run -p salimon-universe --example inference_sanity --locked` emits JSON for
4096 seeds per class, five-bin histograms, attempt counts, clearance and Kepler
residuals; its nonzero exit gates physical failures or collapsed histogram
coverage. CI repeats and compares the entire report. The ten-to-forty-percent
per-bin coverage screen is intentionally broad, not empirical goodness-of-fit.
See [measured report](../reports/issue-410/README.md).

Production source reuse/full reviewed inputs remain #513/#524/#526; U10/U11 own
population/canonical world generation, U12 packaging, U14 state evaluation and
#436–#439 native integration/verification. Those are existing downstream scopes,
not approvals established by this portable heuristic inference delivery.
