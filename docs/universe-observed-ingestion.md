# JPL and observed exoplanet ingestion — U08 (#409)

`client/universe/src/ingestion/horizons.rs` and `exoplanets.rs` are portable local
science adapters. They consume the [U06 verified sources](universe-sources.md),
without downloads, clocks, runtime/renderer coupling or publication permission.
They extend [U07 Gaia matching](universe-gaia-ingestion.md) and leave schema-1
catalog/storage and playable saves unchanged. Observation sidecars are distinct
from canonical records; U12 must package both deliberately.

## Horizons states, physical quantities and validity

`Ephemeris::decode` accepts a bounded (16 MiB) Horizons API 1.2 JSON response with
one nonempty vector-table-2 block. It requires geometric ICRF equatorial states,
Solar System barycenter (0), BODY CENTER, KM-S and TDB. Other centers, axes, units,
API versions, extra JSON fields or ambiguous/duplicate context headers fail.
Positions/velocities convert to meters/meters-per-second; each state carries the
explicit `IcrfSolarSystemBarycenterGeometric` frame label and split TDB `Epoch`.
Target numeric codes become `jpl:<code>` without display-name identity.

AD calendar rows from 1583 onward are parsed and cross-checked against their
printed Julian days to 1e-9 day (~86 microseconds), covering decimal/f64 rounding,
not a scientific state-error budget. Earlier mixed Julian/Gregorian dates reject
until a separate adapter exists. Invalid/leap-second/duplicate/unsorted epochs,
nonfinite or overflowed vectors and malformed physical fields fail. Full response
text retains model, headers, printed values, retrieval context and citation;
state covariance is not supplied and no certainty is fabricated.

Physical import currently recognizes `Vol. Mean Radius (km)` and
`Mass x10^N (kg)` header forms, with optional printed `+-` error. It converts values
and errors to SI `Quantity`, preserves unknown confidence/temporal validity, and
never replaces missing/unsupported source formats with defaults or estimates.
Unrecognized physical columns remain in raw text. The frozen Earth response
provides 6,371,010 m mean radius (+/-20 m) and 5.97219e24 kg mass (+/-6e20 kg).
These header measurements are not assigned the vector-table interval as their
physical validity. `body` exports those quantities with caller-supplied reviewed
system/parent/kind; `Catalog::new` remains the referential publication gate.
No Sun, moons or parent hierarchy are invented from target numbers.

`validity()` is the **imported sample coverage**, not DE441's entire model validity.
`state_at` exposes exact imported epochs only. It rejects both outside-coverage
queries and in-coverage dates without a sample; UTC/TT inputs are not converted to
TDB. There is no frozen-edge fallback, implicit interpolation or extrapolation.
[U14 #415](https://github.com/mr-exception/salimon/issues/415) owns arbitrary-date
model evaluation and explicitly flagged lower-accuracy models outside validity.
The refusal here is an import policy, not delivery of that later orbital service.

## Exoplanet PS measurements and identity

`exoplanets::import` consumes verified PS **default-row** CSV (not mixed-reference
PSCompPars). The pinned query selects `default_flag=1`; if the column is included,
every row must contain `1`. Required columns are `pl_name`, `hostname` and
`pl_refname`; all others may be missing and remain explicit `None`. Shared bounded
CSV decoding retains every original column, quoted literature reference, exact
Archive table key, signed source error and quality/limit metadata.

Gaia DR3 host strings map to the exact U07 `gaia_dr3:<integer>` identity, then to
caller-reviewed canonical aliases. An existing base catalog star is reused with
its original system/physical fields; a wrong-kind match refuses publication.
Without Gaia, reversible UTF-8 hex encoding of the Archive's exact `hostname`
table key in `nea_host` supplies a source identity; `pl_name` is similarly encoded
in `nea_planet`. These are literal source table keys, not normalized display
aliases. Length bounds still apply. Renames require explicit reviewed identity
maps/migrations; no silent name/proximity merge. One hostname resolving to different
canonical hosts, conflicting default planet rows, duplicate/colliding catalog IDs
or dataset revision conflicts fail. Identical repeated rows are counted once.

Catalog generation iterates sorted canonical planet IDs; results are input-order
independent. New single-star systems use U07's injective `system_<namespace>:<key>`
rule. Multiple planets reuse that host. The lowest sorted planet ID supplies only
new-host header provenance/display text; each planet's measured host quantities
stay in its own observation instead of being arbitrarily composited into one
canonical mass/radius. Existing base stars are immutable. Multiple-star host
hierarchy resolution is not inferred from the Archive's names.

Typed measurements cover distance (pc -> m), stellar mass/radius (solar -> SI),
true planet mass and minimum mass separately (Jupiter -> kg), radius (Jupiter -> m),
period (days -> seconds), semimajor axis (AU -> m), eccentricity, inclination and
argument of periastron (degrees -> radians). Periastron source days stay labeled
as such: no BJD/time-scale/frame assumption is made. Missing elements remain
missing; no complete state/orbit, plane orientation or phase is invented.

Adopted conversion constants: day 86400 s, AU 149597870700 m, parsec
3.085677581491367e16 m, solar mass 1.98847e30 kg/radius 695700000 m, Jupiter mass
1.89813e27 kg/radius 71492000 m. These explicit unit conventions are not new
measurements or an inferred mass model; raw source numbers remain available.

PS `err1` must be nonnegative and `err2` nonpositive. Both converted error sides
remain independently optional; a missing side is never fabricated. Two sides
also populate `Uncertainty::Bounds` with unknown confidence; one-sided errors
leave that aggregate uncertainty unknown. Limits (`lim`: 1 upper, -1 lower, 0
measurement; missing flag: `Unspecified`) remain typed. True planet mass/radius
are promoted into canonical body quantities only with explicit measurement flag
0 and bounds admitted by U01. Limits,
minimum masses and bounds crossing physical zero remain sidecars. `pl_bmassj`
(best mass, potentially Msin(i)) is retained raw and never misread as true mass.
Invalid numeric/limit/error flags, orphan metadata, nonpositive physical values,
eccentricity outside [0,1) and inclination outside [0,pi] reject.

## Offline fixture build and checks

```sh
python scripts/import_universe_sources.py --output artifacts/universe-sources-u08
cargo test -p salimon-universe --locked
cargo run -p salimon-universe --example observed_import --locked > artifacts/observed-v1.json
cargo run -p salimon-universe --example observed_import --locked > artifacts/observed-v1-repeat.json
cmp artifacts/observed-v1.json artifacts/observed-v1-repeat.json
python scripts/check_docs.py
```

Use a new U06 output directory. `observed_import` takes no arguments, embeds the
frozen development manifest/artifacts, and emits a deterministic **fixture** JSON
bundle labeled `production_eligible=false`. It combines the two synthetic Gaia
rows, real frozen Earth states/physical header and real frozen Proxima default
row, retaining all source metadata and observation sidecars. There are nine
canonical records and three datasets, not a complete Solar System or galaxy.
The Solar fixture parents Earth to its system without manufacturing a Sun record.
The fixture bundle is not a production wire format or runtime pack. U06 hash
checks validate the same frozen bytes before CI builds the executable; CI then
runs two builds and compares exact output. No raw snapshots/manifest permissions
were modified by U08.

Tests in `src/ingestion/{horizons,exoplanets}/tests.rs` compare the production
importer directly to independent U03 Earth SI/epoch goldens, physical quantities,
Proxima period/distance/errors/references, synthetic Gaia reconciliation, missing
orbits, limits/minimum masses, deterministic duplicates and failure paths.
See [report](../reports/issue-409/README.md) for actual results. Full reviewed
release coverage, source-specific format extensions and production scientific
acceptance remain [#526](https://github.com/mr-exception/salimon/issues/526), after
[#513](https://github.com/mr-exception/salimon/issues/513); pack integration remains
U12, arbitrary-date evaluation U14, native/playable integration U34 and its own
evidence. Portable fixture closure is not production/native approval.
