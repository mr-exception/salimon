# Issue #308 — reviewed visual corpora and periodic appearance coverage

2026-10-18. Incremental on #307 / PR #320. Portable tooling delivered;
**no production aesthetic approval or native signoff is claimed**.

## Delivered

- Version-1 candidate/review metadata records actual runtime backend, adapter,
  driver, physical viewport/DPI, OS, source revision and binary SHA-256.
  A fixture-contract hash binds actions, assertions, seeds, repeats, tolerances
  and masks; PNG hashes and exact checkpoint completeness bind image evidence.
- Independently versioned software-Vulkan and native Metal corpora start
  explicitly pending. Approval requires image review plus reviewer/date/evidence
  and explicit tolerance/mask review. Capture refuses an approved destination;
  there is no promotion command or automatic baseline reset.
- `compare --reviewed` validates provenance/compatibility, then compares the
  existing production fixtures. Exit 0 means approved appearance passed; 1 means
  failure; 3 means missing/pending/unapproved coverage. Temporal defects or
  incomplete evidence fail even when baseline approval is pending.
- Nightly verification preserves temporal checks and adds reviewed appearance.
  History, summaries and dashboard distinguish pending coverage from success;
  real failures override pending. Source-only PR workflow is unchanged.
- Persistent synthetic silhouette, material and layout defects all fail the
  appearance comparator while their repeat stability passes.

## Validation

Linux x86_64, Python 3.12, Rust 1.99.1:

- **Passed:** `python -m unittest discover -s scripts -p 'test_*.py'`
  — 92 tests, including CLI exit codes, overwrite refusal, strict fixture/image
  hashes, metadata compatibility, review requirements and CI pending semantics.
  The printed simulated timeout/failure JSON belongs to a passing failure test.
- **Passed:** `cargo test -p salimon-client --locked commands_use_live_controller_paths_and_inspection`.
- **Passed:** `cargo clippy --workspace --all-targets --locked -- -D warnings`,
  `cargo build --workspace --locked`, `cargo fmt --all -- --check`.
- **Passed:** periodic workflow YAML parsing/appearance-step presence,
  `node --check scripts/ci_dashboard/dashboard.js`, `git diff --check`.
- **Not run:** production GPU captures, live approved production comparisons,
  Metal capture, human aesthetic review, native FPS. CPU synthetic fixtures test
  failure detection and workflow semantics, not production appearance.

## Pending work transferred

[Software-Vulkan review #321](https://github.com/mr-exception/salimon/issues/321)
owns real software captures, review and the separate approved baseline PR.
[Native #314](https://github.com/mr-exception/salimon/issues/314) owns Apple
M1/Metal captures, human review and signoff; it remains pending. Neither corpus
certifies the other. The maintained [review workflow](../../scenarios/visual/README.md)
describes the exact commands, review metadata and tolerance/mask policy.

Portable delivery closes #308 and leaves those approval requirements pending.
