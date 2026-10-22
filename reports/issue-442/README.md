# Issue 442 — offline documentation CI

Delivered standard-library validator, reviewed policy, failing/positive fixtures
and PR source-impact comparison in the Ubuntu source workflow. Fixed two stale
anchors and replaced five nonexistent historical report targets with their issue
threads; no historical evidence is invented. Current maintained scope has zero
baseline debt. Guides: [policy](../../docs/documentation-maintenance.md#offline-ci-validation-442).

Validation (Linux, Python 3.11): five validator contracts pass;
`python scripts/check_docs.py` passes with zero findings.
No Rust behavior changes; Rust/native checks are unnecessary for this tooling PR.
The offline linter's Markdown subset, exclusions and source-impact exceptions are
explicit. It cannot prove semantic correctness or enforce branch protection.
