# Issue #524 — real Gaia DR3 preparation and measured pack blocker

**Incomplete full-release acceptance.** This PR delivers reproducible real-source
acquisition, independent exact-ID evidence and actual offline U07/U12 measurements.
It does **not** close #524, approve a smaller scientific population, approve a size
exception, or claim that the full-release 2–5 GB aim / 10 GB gate passed.

## Frozen scope and results

On 2026-10-31, the AIP Gaia DR3 TAP mirror returned the complete
`random_index < 8192` interval and two separately queried golden stars. Original
VOTable responses, field units/metadata, string-preserving CSV, exact queries,
SHA256s and credits are frozen under
[`catalogs/sources/gaia-524-v1`](../../catalogs/sources/gaia-524-v1/).
The owner-authorized private-development use does not rewrite the source license
status or constitute a public/commercial release decision (#659).

| Production adapter / storage result | Measured value |
| --- | ---: |
| Real sample input rows | 8,192 |
| Selected / quality-rejected / astrometry-rejected | 7,826 / 366 / 0 |
| Selected without measured RUWE | 1,517 |
| Missing or nonpositive parallax | 2,993 |
| Incomplete velocity | 7,681 |
| Canonical records (star + system per selected row) | 15,652 |
| Installed sample pack bytes | 11,894,669 |
| Catalog partition bytes | 794,293 |
| Astrometry + selection sidecar bytes | 10,557,591 |
| Literal-ID root manifest bytes | 539,829 |
| Installed bytes / selected star | 1,519.8913 |
| Peak process RSS, Linux dev build | 151,576 KiB |

These are **real bounded-sample measurements**, not an installed full-Gaia forecast
or hardware performance qualification. All 29 selected original source columns,
including errors, correlations, quality and missing values, survive in the actual
pack sidecar. Original five decode timings, build time, error/coverage
distributions and process context are retained in
[measurement JSON](gaia-measurement-rss.json).

The frozen NASA PS snapshot independently associates exact DR3 IDs
`5853498713190525696` with Proxima Cen and `4472832130942575872` with Barnard's star.
The checks also verify J2016.0, positive published parallax uncertainty and broad
independent parallax ranges. No nearest-neighbor/name-only association or
averaged replacement observation is introduced. The golden pack's explicit
`stellar-golden` aliases are evaluation identities, not new production star names.
See [cross-match evidence](gaia-source-check.json).

Two offline builds produced exactly the same manifests and installed bytes:

- Sample: `53cc1aa9235b67920cea4d710a1c1ec2bc321ca3c60349144c6923dae0d86a26`.
- Golden: `4b3c0f9eaaa081f42ba7c629fae1fa668dfbc288233fcc23e2b9f84a63768234`.

## Full-release blocker and remaining work

The current U12 format stores every selected star **and** generated system ID as
literal strings in one root manifest capped at 16 MiB. Even ignoring almost all
numeric digits, delimiters and other metadata, these two quoted IDs require at
least 31 bytes per source. Thus this layout cannot represent a billion-source
population in the allowed manifest, independently of astrometric compression.
The measured 7,826-star manifest already occupies 539,829 bytes. Raising the cap
would not solve the installed-data budget or the need for a scalable index.

The source issue leaves the final science population undefined. The requested
clarification has not selected a distance/brightness-limited population; this
report retains the broader usable-DR3 scope. No arbitrary truncation is approved
here. Remaining #524 work is a scalable compact index/astrometry representation,
an explicit complete science selection and release-specific association/ambiguity
coverage, full-selection acquisition/count/error distributions, and actual full
installed bytes/decoder/RSS evidence against 2–5/10 GB. An owner-approved size
exception would need its own explicit decision if the measured gate fails.

A full-table TAP `count(*)` request returned HTTP 400; the exact
[request](full-gaia-count-request.json) is retained. Full-table counts, full-release
snapshots, scientific association coverage beyond these goldens and full-pack
budget acceptance are **failed/unrun**, not inferred from the random sample.
The primary ESA TAP endpoint initially timed out; successful acquisition used the
recorded AIP mirror. The sample decoder is a dev-profile cloud measurement;
release-profile throughput and physical hardware budgets remain unverified.

## Commands and provenance

Parent/source base: `3bcf3bf977108408e4a3ba9d855aef0051370515` (`main`).
Environment: Linux x86_64, glibc 2.41, 3 logical CPUs, Rust 1.99.1, locked workspace.

```sh
python scripts/acquire_gaia_sources.py --output catalogs/sources/gaia-524-v1
python scripts/verify_gaia_sources.py --sources catalogs/sources/gaia-524-v1 --ps catalogs/sources/observed-526-v1/ps.csv
cargo run -p salimon-universe --example gaia_verify --locked -- catalogs/sources/gaia-524-v1 artifacts/gaia-524-measurement
# Reused executable, second fresh output; Python resource.RUSAGE_CHILDREN recorded RSS:
target/debug/examples/gaia_verify catalogs/sources/gaia-524-v1 artifacts/gaia-524-measurement-rss
```

The acquisition, independent verification, two offline builds and exact hashes
passed. Existing U07 tests retain ambiguous-match, duplicate conflict, missing
measurement and malformed-source failures. New tooling tests reject TAP
ERROR/OVERFLOW, duplicate fields and row-width errors without losing large IDs,
nulls or units. Shared Rust/script/lint/docs outcomes are recorded with the
[combined validation](../issue-470/README.md).

Documentation impact: [Gaia contract](../../docs/universe-gaia-ingestion.md),
universe README/architecture/invariants, source encoding and root routing are
updated in this PR. VOTable is an explicit retained-source encoding, not a new
runtime network path. Current scientific/full-budget ownership stays on #524;
#659 and #720–#723 retain separate release decisions/qualification.
