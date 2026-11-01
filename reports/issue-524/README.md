# Issue #524 — nearby celestial catalog within 1 GB

## Outcome and area

Delivered the owner's 2026-11-01 replacement scope: a documented nearby celestial
snapshot, under **1,000,000,000 bytes**, with a reproducible expansion path.
The complete frozen sources and production offline pack are retained privately
in this workspace; the public repository tracks coverage, hashes and tooling.
The earlier all-usable-DR3 scope is preserved in the
[historical full-release preparation report](full-release-preparation.md), not
claimed passed by this bounded delivery. #659 still owns future public/commercial
source distribution; no source status or release eligibility is promoted.

[Canonical coverage and expansion guide](../../docs/nearby-celestial-catalog.md)
and [machine-readable coverage record](../../catalogs/nearby-524-v1.json).

| Selection / result | Actual value |
| --- | ---: |
| Gaia region | All sky, observed parallax >=10 mas |
| Nominal boundary | 100 pc / about 326 light-years around the Sun |
| Complete-query raw Gaia rows / pages | 574,531 / 71 |
| Accepted Gaia stars | 331,735 |
| Quality-rejected Gaia rows, retained in sources | 242,796 |
| Astrometry rejections / duplicate rows | 0 / 0 |
| Accepted Gaia sources with incomplete velocity | 202,476 |
| NASA PS planets with nominal host distance 0–100 pc | 1,590 |
| Existing selected JPL Solar System targets | 18 |
| Published stars / systems / individual bodies | 331,913 / 331,913 / 1,607 |
| Frozen source bytes | 184,061,247 |
| Installed pack bytes | 146,408,616 |
| **Frozen sources + one installed pack** | **330,469,863** |
| Portable bundle bytes / files | 330,936,320 / 572 |
| Root manifest bytes (v2 limit 64 MiB) | 22,925,711 |
| Pack cells | 72 |
| Peak child RSS across both release executions | 268,052 KiB |

This is an observed-parallax/nominal-PS-distance selection, **not a complete or
error-free physical sphere**. Source errors, missing data, Gaia quality cuts and
JPL frames/epochs remain explicit. The published star count also includes 177
PS-backed stars beyond the accepted Gaia observations plus the Sun; it does not
reclassify rejected Gaia astrometry as passing. Systems retain the existing
simple star grouping, not a solved multiple-star hierarchy. The 18 JPL targets
are selected coverage, not every Solar System body. Frozen original PS rows are
retained for provenance; only the distance-selected rows enter this pack.

## Data location and hashes

- `artifacts/nearby-524-v1/sources/`: complete original TAP responses and exact CSV
  transcription, count/tail responses, units/metadata, URLs/queries, hashes,
  selected PS/JPL inputs, selection record and offline build request.
- `artifacts/nearby-524-v1/pack/`: production content-addressed pack.
- `artifacts/nearby-524-v1.tar`: private portable bundle, including local coverage
  and README. It was created once, its exact member set/sizes and both embedded
  manifests were checked, and its SHA256 was recorded. Bulk data is ignored by
  Git because this repository is public; the bundle is delivered via the workspace.

Bundle SHA256:
`73d27250c77799dac8d1c189f7333aabe3218a8da1c1fc6c13916777791bc0e6`.
The coverage record binds the exact source, selection and pack manifest hashes.
The bundle is reproducible offline from its frozen input bytes. A fresh later
TAP capture can have different server-generated XML metadata/hashes and must be
recorded as a new capture rather than silently replacing this one.

## Implementation and verification

Acquisition uses `(parallax DESC, source_id ASC)` keyset pages and exact source
parallax text. Initial/final counts and an empty tail pass. IDs are unique and
strictly advance by that key. TAP ERROR/OVERFLOW, changed columns, units or
counts, duplicate/nonadvancing keys, source tampering, changed resume parameters
and budget overruns fail closed. Source-ID-only pagination timed out after
98,304 rows; the [actual TAP error](nearby/nearby-page-failure.xml) is retained.
Parallax-indexed paging completed the same full query; no rows were dropped to
avoid that failure. The final retained Gaia TAP payload totals 332,545,344 bytes.

Original source XML/CSV use lossless gzip archives. Stored hashes bind the gzip
files; original-response hashes bind inflated bytes. The production observed
adapter verifies stored hashes before bounded 8 MiB CSV inflation and rejects
truncated/trailing members. All selected original columns/uncertainties remain
intact. PS joins use exact published DR3 IDs, with separate batches for other
hosts; there is no name/proximity merge. Independent Proxima/Barnard host,
parallax/error and J2016.0 checks pass.

Pack v2 losslessly compresses sidecars, bounds exact decoded lengths and rejects
corrupt, truncated or concatenated streams. Raw v1 packs still read and keep
their original 16 MiB root limit; v2 admits the measured nearby manifest within
64 MiB. The builder checks global IDs/sources incrementally rather than cloning
and revalidating every preceding cell on each batch. Finish still validates the
whole publication and atomically replaces the active manifest. Collision,
migration, corruption and failed-update regressions preserve those contracts.

Both fresh release builds exercised every production filesystem cell/sidecar and
five exact-ID goldens. They produced **identical manifests, installed byte counts,
record/selection counts and every decoded-sidecar hash**. Original per-cell
read/decode samples and actual build times are retained in
[run A](nearby/nearby-pack-a.json) and [run B](nearby/nearby-pack-b.json), with
[repeat/RSS evidence](nearby/nearby-repeat-rss.json). The first build took 37.648 s;
its 72 original catalog-read/decode samples sum to 2.227 s. The filesystem is warm
after writes; these are Linux release CPU/IO observations, not cold-disk,
Metal/GPU, frame-rate or reference-hardware claims. The 64 KiB catalog cache never
exceeded its bound; manifest/index, decode scratch and returned catalogs are separate.

The independent verifier checks original query/boundary/continuation semantics,
source units/datatypes and every original CSV string. With `--pack`, it compares
**every published Gaia and planet source field** with its original row, including
errors/correlations/missing values, verifies exact selection membership/counts,
and enforces the combined 1 GB budget. Both the
[source verification](nearby/nearby-source-verification.json) and
[complete pack verification](nearby/nearby-final-verification.json) pass.

## Commands and checks

Base/source: `75a1e112be2e5435d9671c41edd345d27d0163cb` (merged #725), branch
`issue-524-scalable-gaia`. Linux x86_64, Rust/Cargo 1.99.1, locked dependencies,
3 online logical CPUs / 2-CPU cgroup quota, 10,452,459,520 bytes RAM.
Rust commands used `source /workspace/.salimon-tools/env.sh`.

| Command / coverage | Result |
| --- | --- |
| `cargo test -p salimon-universe --locked` | 124 passed, no ignored tests; [log](nearby/nearby-universe-tests.log) |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | Passed; [log](nearby/nearby-workspace-clippy.log) |
| `cargo fmt --all -- --check` | Passed |
| `python -m unittest discover -s scripts -p 'test_*.py'` | 166 passed; [log](nearby/nearby-all-script-tests.log). Printed failure/timeout text belongs to deliberate negative fixtures. |
| `python scripts/check_docs.py --base origin/main` | Passed, zero findings; final committed-base check recorded with the PR |
| `python scripts/acquire_nearby_sources.py --output catalogs/sources/nearby-524-v1` | Complete region; moved to the private `artifacts/nearby-524-v1/sources` location afterward |
| `prepare(Path('artifacts/nearby-524-v1/sources'))` from `acquire_nearby_sources` | Explicit `nearby-524-v1` publication and complete exact-host PS/JPL routing |
| `python scripts/verify_nearby_sources.py --sources artifacts/nearby-524-v1/sources` | Passed full frozen-source verification |
| `cargo build -p salimon-universe --example nearby_verify --release --locked` | Passed |
| `target/release/examples/nearby_verify artifacts/nearby-524-v1/sources artifacts/issue-524-preflight/nearby-pack-a` | Passed all cells/sidecars/ID goldens; repeated with `nearby-pack-b` |
| `python scripts/verify_nearby_sources.py --sources artifacts/nearby-524-v1/sources --pack artifacts/nearby-524-v1/pack` | Passed every published original field, coverage and combined size gate |

Initial workspace build/check attempts found missing ALSA development files.
Installed Debian `libasound2-dev`, `libasound2t64` and `libasound2-data` into the
executor's local sysroot; no repository workaround or dependency relaxation was
introduced. The final workspace Clippy and focused source tests pass. The existing #526
full observed-source verifier was also exercised against a new v2 pack: all
6,451 PS rows and 18 JPL targets retain their original numeric/error/limit
[checks](nearby/observed-526-v2-coverage.json). Both Python verifiers share the bounded raw/v2 storage decoder, protected
by two additional malformed-stream/compatibility regressions. Broad
final-head workspace build/tests are tracked by the PR's required CI; CI uses
hermetic fixtures, while the complete private regional corpus is checked locally.
No graphics, gameplay, physical-device or hardware performance claim is made by
this source-download task, so no GPU captures or human trials were run.

## Documentation impact and future expansion

Updated all three universe owner guides, source/pack/reader/Gaia contracts,
architecture and maintenance routing, validation and tooling commands. The
[nearby guide](../../docs/nearby-celestial-catalog.md) defines area, gaps, private
location, data limits, replay and a versioned expansion procedure. It distinguishes
the owner's nearby/1 GB goal from Codex's chosen first 100 pc observation boundary.

Mapping: `pack.rs`/`pack/observed.rs`/`sources.rs` → compression, manifest-bound,
global-identity and gzip-source regressions → pack/source contracts;
`acquire_nearby_sources.py`/`verify_nearby_sources.py` → eight offline resume,
transcription, unit/query and false-coverage boundary tests → nearby guide;
`nearby_verify.rs` → actual full regional source/pack/reader evidence above.
Local guide paths/links and source-impact coverage are checked before readiness.

Neutral `[0,0,0]` storage shards are not physical galactic locations. Source
astrometry stays available for an explicit later coordinate/inference adapter;
this download does not replace the ordinary playable world or grant player
knowledge. Full Milky Way coverage/scalable indexing and the old full-release
budget remain future expansion, not passes inferred from this regional result.
Keep this publication stable, choose an explicit next boundary/revision/budget,
reacquire and reverify complete coverage, and extend the paged index if current
bounds are reached rather than silently truncating or rekeying the data.
