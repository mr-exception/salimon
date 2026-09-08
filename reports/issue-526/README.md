# #526 — selected observed-source coverage

Delivered full frozen PS default-row coverage and the explicitly selected Solar
System input hierarchy in the real private offline pack pipeline. Inputs are
original public service bytes acquired on 2026-10-31 under the owner's private,
noncommercial authorization. No release terms/status or playable save is changed.

Base/parent: `main`, `f3dc39435e5be639cce0ed906ed3017e384c37ea`.
Branch: `issue-526-observed-source-coverage`. No active PR stack existed.
[Canonical contract](../../docs/observed-source-coverage.md) maps source, input
formats, identities, commands and limitations. No merge performed.

## Outcome and evidence

- 6,451 PS default rows, 4,847 hosts (4,477 exact Gaia DR3 IDs; 370 literal Archive
  host IDs), 1,069 multi-planet hosts; 577 rows retain multi-star metadata.
  The separately frozen count query agrees; no rows were filtered to pass import.
- 18 selected JPL centers, 54 geometric ICRF/SSB TDB samples, exact 12-hour epochs
  from 2000-01-01 through 2000-01-02. Sun is a star; planet/moon/Pluto parents are
  explicit. All 18 mean radii import; six GM-only/blank masses stay absent.
- Real physical-header variants, spaced uncertainty and approximate Sun mass are
  deliberately supported. Duplicate/malformed values, wrong expected target and
  invalid star parent reject. The old `body_kind` request spelling remains valid;
  canonical catalog and gameplay save schemas do not change.
- The independent verifier compares every raw PS column, ID/parent, SI value,
  signed error, missing value, limit and canonical promotion; every JPL vector
  component/split epoch and original model/reference text; every source/blob
  hash and catalog CRC. Reported component differences and PS conversion
  differences are zero for these bytes. Decoder tolerances are 2e-15 relative /
  1e-12 absolute; this is not zero scientific/model uncertainty.
- 16,164 canonical records; 1,091,615-byte compressed partition/index;
  26,398,189-byte PS sidecar; 136,267-byte JPL sidecar; 582,216-byte pack manifest;
  total unique blobs 27,657,468 bytes (manifest is additional). Build: 2.861 s,
  maximum child RSS 321,600 KiB; independent verification: 0.523 s. Linux x86_64,
  Rust 1.99.1, dev profile with debug info; no hardware performance claim.

Original bytes, exact query/selection, retrieval context, credits and hashes are
in [the frozen selection](../../catalogs/sources/observed-526-v1/selection.json)
and [source manifest](../../catalogs/sources/observed-526-v1/manifest.json).
[Coverage JSON](coverage.json) retains per-field missing/limit counts, raw-source
value/relative-error min/median/max distributions, real named goldens and source
models. [Measurements and source hashes](measurement.json) bind the implementation
used. [Pack manifest](pack-manifest.json.gz) preserves the exact output identities
and all blob hashes/sizes. Large observation blobs are reproducible from retained
originals using the documented builder; they are not duplicated in Git.

## Verification

All commands ran from the repository root with
`/workspace/.salimon-tools/env.sh` sourced for Rust.

| Command | Result |
| --- | --- |
| `cargo test -p salimon-universe --locked` | 116 passed, zero failed/ignored; includes whole-selection repeat build, real header values, independent Earth goldens, expected-ID refusal, source/alias/migration/atomic-publication regressions |
| `cargo clippy -p salimon-universe --all-targets --locked -- -D warnings` | Passed |
| `cargo build -p salimon-universe --example pack_build --locked` | Passed |
| `cargo fmt --all -- --check` | Passed |
| `python -m unittest discover -s scripts -p 'test_*.py'` | 147 passed; includes four acquisition/verifier tests; mocked negative-case output in log is expected |
| `python scripts/import_universe_sources.py --manifest catalogs/sources/observed-526-v1/manifest.json --output artifacts/issue-526/staged-sources` | 20 original artifacts verified/staged without network |
| `./target/debug/examples/pack_build catalogs/sources/observed-526-v1/build.json artifacts/issue-526/final-pack > artifacts/issue-526/final-manifest.json` | Passed; measured with Python monotonic clock / Linux child resource usage |
| `python scripts/verify_observed_sources.py --snapshot catalogs/sources/observed-526-v1 --pack artifacts/issue-526/final-pack --output artifacts/issue-526/coverage.json` | Passed, complete frozen selection |
| `python scripts/check_docs.py` | Zero failures |
| `git diff --check` | Passed |

Compressed original logs: [universe](universe-tests.log.gz),
[scripts](python-tests.log.gz), [lint](clippy.log.gz), [build](build.log.gz),
[format](fmt.log.gz), [docs](docs.log.gz), [staging](staging.log.gz).
CI now builds the frozen selection and runs its independent verifier offline;
current-head remote checks are reported on the PR/issue at publication.
Workspace build/tests and cross-platform publication goldens are covered by that
CI, rather than duplicating the whole game build locally for this leaf change.

## Corrected attempts and limits

Initial full input build exposed Charon's blank mass field followed by the next
physical column (`unsupported Horizons mass value`); the parser now keeps it
missing and has direct regressions. One Clippy run rejected redundant
`trim_start` before `split_whitespace`; corrected without suppressing the lint.
During verifier development the first CRC check incorrectly used decoded bytes
instead of the format's compressed payload, then the record wrapper needed
unpacking; both verifier errors were corrected before any pass was recorded.
`/usr/bin/time` was unavailable; actual build measurements use standard Python
`resource` instead. No tolerance or source value was loosened to obtain a pass.

This inventory does not assert coverage of every Solar System moon/small body,
current-date or interpolated ephemerides, complete orbital phase/frame/astrometry,
reconstructed multi-star hierarchy or independent full Gaia cross-matches. The
source-inventory cell is an explicit staging key, not a located gameplay galaxy.
Numerical decoder agreement does not establish physical/model accuracy beyond
provider metadata. Existing U14 owns arbitrary-date model evaluation; #524 owns
full Gaia science and whole-galaxy storage budgets. Runtime/end-to-end adoption
retains its existing owners (#435/#439). No unimplemented gameplay is transferred
from this issue to claim an input-coverage pass.

Public/commercial redistribution is unreviewed (#659). Physical/native/M1/Metal,
GPU, presentation and 60 FPS qualification were not run and are not relevant to
this source/portable-pack change; #557/#561/#563 retain their existing ownership.
No source approval, world/asset baseline or release was promoted.
