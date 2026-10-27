# Private public-data authorization — 2026-10-27

The owner explicitly authorized public astronomy data for private noncommercial
development and deferred legal/ethical and redistribution review until a future
public or commercial release. Real acquisition, ingestion, scientific validation
and locally used offline packs are authorized beyond existing synthetic/small
fixtures. No additional owner source-use approval is required for that scope.

Recorded this decision in the source policy, Project Q&A and universe contributor
guide. #659 remains open as a future release review. Updated #413/#524/#526 and
PRs #663–#667 to distinguish that future gate from current development work.
Actual source completeness, model accuracy, indexing/performance and unfinished
implementation remain incomplete where previously reported.

Existing non-production Rust/Python validators already admit observed snapshots;
the production review sidecar belongs to release/distribution mode. No runtime
code, data bytes, manifest hashes, provider-status metadata or release approval
was changed. No data acquisition, publication or PR merge was performed.

Validation: documentation/base-impact checks and existing source-importer
contracts are run for this clarification; exact results are recorded below.

- `python3 scripts/check_docs.py --base codex/issue-417-gravity-flight`: passed,
  zero failures and zero retained historical findings.
- `python3 -m unittest discover -s scripts -p test_import_universe_sources.py`:
  six passed, zero failures.
- `git diff --check`: passed. No code changed, so no gameplay, hardware or new
  scientific-data results are claimed.
