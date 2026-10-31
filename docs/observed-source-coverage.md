# Selected observed-source coverage (#526)

The private [frozen input](../catalogs/sources/observed-526-v1/manifest.json)
contains all 6,451 NASA Exoplanet Archive PS `default_flag=1` rows returned on
2026-10-31, plus 18 explicitly selected Solar System centers from Horizons API
1.2. A separately retained `COUNT(*)` response agrees with the PS row count.
Exact query URLs, acquisition date, bytes, SHA256s, credits, model headers and
original per-row literature references are retained. This is private development
under the [owner authorization](universe-sources.md#current-private-development-authorization-2026-10-27).
Provider terms remain `unreviewed`; public/commercial release review is #659.

## Selection and identity

The Solar selection is Sun, eight planets, Moon, Io, Europa, Ganymede, Callisto,
Titan, Triton, Pluto and Charon. Body centers use exact JPL IDs; the Sun is a star,
planets/Pluto parent to Sun and moons to their individual primary. Pluto is an
individual minor body. No other small-body or moon completeness is asserted.
Vectors are geometric ICRF equatorial, Solar System barycenter, KM-S, TDB,
2000-01-01 through 2000-01-02 at 12-hour intervals (54 samples). Header physical
quantities keep separate unknown validity; sample coverage does not become the
DE441/satellite model validity. Epoch evaluation remains U14.

PS includes source identity, reference, multiplicity and all supported
measurement/error/limit fields. Exact integer Gaia DR3 host IDs are retained;
missing Gaia IDs retain reversible literal Archive hostname keys. No alias or
rename is inferred. Existing explicit cross-match and pack migration gates
remain in force; the frozen snapshot introduces no renames. Golden real examples
include Proxima, TRAPPIST-1, Kepler-16 and PSR B1257+12. Multi-star multiplicity
stays in raw metadata: an Archive host identifier is not a reconstructed binary
hierarchy. Missing phase, frame, epoch, astrometry and orbital elements remain
missing. Full Gaia matching/coverage and galaxy storage acceptance stay #524.

## Supported physical headers

The [Horizons adapter](../client/universe/src/ingestion/horizons.rs) deliberately
recognizes case/spacing variants of volume-mean/mean radius in km, the Charon
IAU2015 radius label and the selected mass labels with explicit kg powers of ten.
Spaced `+-` errors are preserved. Sun's printed approximate mass retains unknown
uncertainty and its original `~` in the raw response. Blank Charon mass remains
missing; GM-only satellites never acquire inferred mass. Duplicate recognized
fields, nonfinite/nonpositive values, invalid uncertainties and malformed values
reject. Unsupported fields remain raw; no default mass/radius is supplied.

Raw pack requests use `object_kind: star|planet|moon|minor_body`, explicit `system`,
`parent`, and optional expected `target`. The older `body_kind` spelling remains
an alias for compatibility. This selection supplies `target` for every response;
a mismatched numeric source target rejects before publication. A star's parent
must be its system. Canonical schema/save formats are unchanged.

## Reproduction and verification

```sh
cargo test -p salimon-universe --locked
cargo clippy -p salimon-universe --all-targets --locked -- -D warnings
cargo fmt --all -- --check
python -m unittest discover -s scripts -p test_observed_sources.py -v
cargo run -p salimon-universe --example pack_build --locked -- catalogs/sources/observed-526-v1/build.json artifacts/observed-526-pack > artifacts/observed-526-manifest.json
python scripts/verify_observed_sources.py --snapshot catalogs/sources/observed-526-v1 --pack artifacts/observed-526-pack --output artifacts/observed-526-coverage.json
python scripts/check_docs.py
```

The independent Python verifier checks raw hashes, every pack blob/index/CRC,
every PS raw column, identity, parent, SI value, signed error, missing value,
limit and canonical promotion, plus all JPL vector components, split epochs,
original headers and parent/classification. Relative decoder comparison tolerance
is 2e-15 with absolute 1e-12 rounding tolerance. These tolerances are arithmetic
checks, not source accuracy or uncertainty approval. The retained report includes
null/limit counts and min/median/max source uncertainty distributions; no known
scientific uncertainty is replaced by a numerical zero-error claim. Rust tests
independently transcribe physical headers, retain Earth reference goldens and
exercise failed identity publication, malformed metadata and repeat builds.
CI builds this selection and runs the independent verifier offline.

The pack includes source manifest, canonical compressed partition/index, PS
observation/summary and JPL state/raw-response sidecars. The single source-inventory
cell `[0,0,0]/0` is an explicit staging key, not a claimed galactic location for
hosts lacking astrometry. It is not installed as a gameplay spatial index or
player navigation knowledge. Its measured storage/memory is for these inputs;
there is no full-Gaia density extrapolation or whole-galaxy budget claim.

To acquire a **new** selection, explicitly run:

```sh
python scripts/acquire_observed_sources.py --output artifacts/new-observed-snapshot
```

Acquisition uses inherited HTTP proxy/CA trust and bounded requests. Existing
paths are never replaced. Failed acquisition retains original partial bytes but
publishes no completed manifest/request. Live source updates may change counts,
measurements or identities; review the new bytes and scientific results before
using a new revision. Tests and normal pack builds never download data.
