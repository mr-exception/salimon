# Issue 409 — JPL and observed exoplanet ingestion

Delivered bounded portable Horizons API-1.2 vector-table-2 ingestion, explicit
ICRF/SSB geometric frame and TDB epoch, SI positions/velocities and physical
header mass/radius/errors. Exact-state lookup refuses missing/out-of-range dates
and cross-scale inputs; imported coverage does not claim full model validity.
The caller supplies reviewed system/parent/kind when exporting a canonical body.

PS default-row ingestion preserves original keys/columns/references, independently
optional signed errors, missing values and explicit/unspecified limit flags.
Typed period/distance/physical/orbital quantities use documented conversions.
Gaia IDs reconcile with U07/base stars without overwriting them. True measured
planet mass, minimum mass and limits remain distinct; no unmeasured orbit, host
physical composite, phase, time scale or velocity is invented. Duplicate/match
conflicts fail and canonical/observation order is reproducible.

[Canonical contract](../../docs/universe-observed-ingestion.md),
[architecture](../../client/universe/architecture.md#observed-solar-system-and-exoplanets--u08-409)
and [invariants](../../client/universe/invariants.md) map sources/tests/limits.
Crate README/AI guide, root architecture/maintenance routing, source/reference/
validation guides and approved-design status change in this PR. Local link/path
lint and manual source/guide review cover units, missing data, identity and validity.

## Validation

Linux x86_64 cloud CPU, Rust 1.99.1, Python 3.12.14; no extra dependencies:

- `cargo test -p salimon-universe --locked`: **44 passed**, including eight new
  U08 contracts. Actual importer states match the independent U03 Earth SI/epoch
  goldens (0.00025 m / 1e-10 m/s numeric allowances, not physical covariance).
- Physical header checks: Earth radius 6,371,010 m +/-20 m; mass 5.97219e24 kg
  +/-6e20 kg, unknown physical validity/confidence.
- Actual pinned Proxima checks: host `gaia_dr3:5853498713190525696`, period
  966,353.76 seconds +/-45.792 seconds; distance 1.30119 pc with distinct
  +0.00034/-0.00035 pc errors and original literature reference.
- Synthetic checks: Gaia/base reconciliation, no-Gaia source keys, multi-planet
  ordering, duplicates, missing orbits, upper/lower/unknown-limit semantics,
  minimum vs true mass, partial errors and malformed/ambiguous input.
- `python3 scripts/import_universe_sources.py --output artifacts/universe-sources-issue409`:
  pass; all three local source artifacts verified/staged with no network.
- `cargo run -p salimon-universe --example observed_import --locked` twice and
  `cmp` of redirected files: pass; summary/digest recorded below. CI runs these
  commands after U06 verification. Output is explicitly a development fixture,
  not a production artifact. Three datasets/nine canonical records preserve
  complete sidecars; schema-1 catalog and U04 wire are unchanged.
- `cargo test --workspace --locked`: **824 passed, 28 intentionally ignored**, no failures.
- `cargo build --workspace --locked`: pass.
- `cargo clippy --workspace --all-targets --locked -- -D warnings`: pass.
- `cargo fmt --all -- --check`: pass.
- `python3 -m unittest discover -s scripts -p test_import_universe_sources.py -v`:
  four passed (including no-network reproducibility and production refusal).
- `python3 scripts/check_docs.py`: pass, zero findings.
- `python3 scripts/check_docs.py --base issue-408-gaia-ingestion`: pass
  after committing, including changed-source owning-guide coverage.

[Validation log](validation.log) preserves actual tests/build/Clippy output.
The final two fixture outputs contain **25,192 identical bytes**, SHA256
`eee67fc1e69b021134f4f1c2d0cdca865bde89a1b024c87390a1ae1a4c2eef6a`.
[Fixture summary](fixture-summary.json) records source counts and the observed
norm errors versus independent Earth goldens: position 0 m, velocity <=3.64e-12 m/s.
These are numerical conversion checks, not an ephemeris accuracy claim.

## Delivery scope and remaining gates

This PR follows [U07 PR #525](https://github.com/mr-exception/salimon/pull/525);
merge #525 first. Earth/Proxima reference-fixture acceptance, missing-data and
portable failure/CI contracts pass. Actual frozen source fixtures remain exactly
the U06 inputs; no production snapshots/manifest permissions were promoted.

Full reviewed Solar System/PS coverage and physical/source-format extensions
transfer to [#526](https://github.com/mr-exception/salimon/issues/526), after
[#513](https://github.com/mr-exception/salimon/issues/513) source-use/full-snapshot
approval; U12 #413 now preserves that dependency and must include observation
sidecars. Current physical parsing recognizes the documented radius and scaled
mass forms; unrecognized columns remain raw. This is not a full SPK ephemeris pack.
[U14 #415](https://github.com/mr-exception/salimon/issues/415) already owns
arbitrary-date evaluation and explicit lower-accuracy outside-validity models;
U08 does not silently extrapolate or manufacture those models. Native/playable
integration/reference M1 checks were **not run** and remain #435–#439/#521.
Closing #409 on PR merge records portable delivery and explicit remaining scope,
not scientific production coverage, source permission or native approval.
