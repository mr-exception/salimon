# U10 canonical fictional-system builder (#411)

Portable implementation on 2026-10-22, based on merged U09 (#529). Delivery PR
will close #411 on owner review/merge; no merge is performed by this task.

## Outcome and ownership

`salimon-universe::synthesis` now publishes immutable, bounded offline catalogs
and full generation sidecars from explicitly reviewed residual-density, age and
metallicity regions. Original observed records/datasets are preserved verbatim;
reserved fictional IDs are stable across ordering, installs and unrelated regions.
Stars, planets, physically admissible major moons and selected large small bodies
are materialized individually. Asteroid/comet populations retain compact count,
size/radial/density/seed recipes instead of member allocations. SHA-256 covers
actual generation recipes, seed-domain inputs, catalog and full publication bytes.

The local file adapter bounds input and stages output replacement; the synthetic
cohort workload measures distributions and raw/compressed storage. CI repeats
the workload and checks the same publication golden on macOS/Windows portable
jobs alongside Ubuntu workspace tests. See the [builder contract](../../docs/universe-fiction-builder.md),
[owning architecture](../../client/universe/architecture.md#canonical-fiction-synthesis--u10-411)
and [invariants](../../client/universe/invariants.md).

## Measured evidence

Linux x86_64 Debian 13; Rust/Cargo 1.99.1; Python 3.12.14. [Validation summary](validation.txt) and [release cohort/storage evidence](sanity.json) retain the actual results. No graphical capture applies
to this portable offline change.

- `cargo test --workspace --locked`: **839 passed**, 0 failed; 28 intentionally
  ignored opt-in checks.
- `cargo test -p salimon-universe --locked`: **59 passed**, including all final
  golden/domain-reference assertions; release synthesis tests: **6 passed**.
- `cargo build --workspace --locked`: passed.
- `cargo clippy --workspace --all-targets --locked -- -D warnings`: passed;
  final universe all-target check also passed.
- `cargo fmt --all -- --check`: passed.
- `python scripts/check_docs.py` and `--base origin/main`: 0 findings on the maintained-guide check;
  base-impact check: 0 findings. Validator contracts: 5 passed.
- `fiction_build` pinned request: 8 systems, 152966 output bytes, full SHA-256
  `97d2c46654d7afc5e4aac03fdf94358e9182f0b8ff38251990a802684ea632f1`.
- `fiction_sanity`: 256 systems, 2580 records, 512 descriptors representing
  281599744 unmaterialized minor members. Catalog JSON 3477943 bytes; zlib-6
  424279 bytes. Catalog plus request/sidecars 5484124 bytes; zlib-6 794203 bytes.
  Full repeated/reordered publications and repeated JSON summaries matched;
  debug/release summaries were also byte-identical.

The fixed-seed tests cover 512 systems at low/high metallicity and young/old ages,
physical surface/Roche/Hill constraints, Kepler consistency, density/count scaling,
stable identity and per-domain input/reference hashes. Four distinct workload
cohorts retain five-bin spatial coverage and low-mass-dominated stellar draws.
These are broad sanity checks, not empirical Galactic population goodness-of-fit.

The file adapter produced identical pinned output twice; unknown-version,
malformed and oversized requests preserved prior output. Input/output aliases
were refused without changing the source and no scratch files remained.

## Documentation impact

Updated root/crate README, README.ai, architecture/invariants, technical
architecture, maintenance map, validation, approved-design implementation status,
canonical generation policy implementation notes and the new builder contract.
Source/tests, adapter commands, units, recipe formulas, provenance, namespace,
digest/version semantics, bounds and downstream dependencies are mapped there.
Superseded schema-only/no-hashing claims are corrected without weakening future
surface/migration/discovery rules. Local link/path/anchor and base-impact results
are recorded in the validation summary. Remote macOS/Windows portable CI goldens
are pending PR creation; they are not claimed passed locally.

## Limits and downstream ownership

The model is deliberately restricted to single main-sequence systems with broad
heuristics, circular planets, one admissible major moon and selected belt bodies.
Residual-density/age/metallicity maps are reviewed caller inputs, not a fitted
full-Galaxy source-completeness model. The coarse stellar proxy/lifetime truncation
is not calibrated astrophysical population synthesis or long-term stability.
No real observation is replaced or misrepresented as fiction.

U11 [#412](https://github.com/mr-exception/salimon/issues/412) owns full canonical
physical/terrain/geology definitions: U10 hashes actual generation input recipes,
not a complete landable surface. U12 [#413](https://github.com/mr-exception/salimon/issues/413)
owns reviewed production region/data authoring, large-catalog batching and complete
sidecar packaging, hash manifests and actual install-size/throughput acceptance;
small synthetic zlib results do not establish a 2–5 GB Galactic pack. Source gates
[#513](https://github.com/mr-exception/salimon/issues/513),
[#524](https://github.com/mr-exception/salimon/issues/524) and
[#526](https://github.com/mr-exception/salimon/issues/526) remain required. U14
[#415](https://github.com/mr-exception/salimon/issues/415) owns missing epoch,
phase, orientation, velocities and complete evolving state evaluation.

Native runtime/scanner/save/terrain integration and Apple M1/Metal graphical and
performance signoff remain [#436](https://github.com/mr-exception/salimon/issues/436),
[#437](https://github.com/mr-exception/salimon/issues/437),
[#438](https://github.com/mr-exception/salimon/issues/438),
[#439](https://github.com/mr-exception/salimon/issues/439). They were not run here;
portable/CI results must not be interpreted as native signoff. Existing playable
world, terrain/resource-v1 IDs, save schema and player-known chart are unaffected.
