# Repair CI dependencies on removed reports

Source: main `0a7715f93ad7c2da5b3e7621cb3b7eca9871bf66`.
The failing [source build](https://github.com/mr-exception/salimon/actions/runs/41174551654)
reported 111 missing documentation targets. The failing
[periodic run](https://github.com/mr-exception/salimon/actions/runs/41175606583)
could not import `reports/issue-288/verify.py`. Its weekly benchmark also referenced
a removed `reports/issue-196/baseline.json`.

## Changes

- Replace 111 broken report links in 49 owned guides with immutable GitHub blob
  links. Every target was verified with `git cat-file -e` before replacement.
  Existing reports use `d003bd58d506e04e3df97744f29f68449b6e2b3e`; reports absent
  there use original PR #713 head `f1cd0b3e878fac487ce898547a0c8c35aa40519b`.
- Move the unchanged pure numerical `evaluate` function into `scripts/active_budget.py`.
  Current tests no longer dynamically import historical verifier dependency chains.
  Raw historical replay stays at its archived revision.
- Retain the original proposal and flight baseline under `scripts/fixtures/` and
  update the weekly workflow path. No thresholds, masks or numerical limits change.
  A regression validates all twelve baseline workload/count rows.
- Explain historical command checkout requirements in report, runner and validation guides.

## Validation

Cloud Debian 13 x86_64, Python 3.12.12, Rust/Cargo 1.99.1; locked dependencies.

- `python3 scripts/check_docs.py`: PASS, zero failures, no added baseline exemptions.
- `python3 -m unittest discover -s scripts -p 'test_*.py'`: PASS, 142 tests.
- `cargo fmt --all -- --check`: PASS.
- `cargo build --workspace --locked`: PASS after supplying missing local ALSA development files.
- Fixture bytes match their original Git blobs exactly. SHA-256:
  proposal `6c8101c6b4ab414f7458c5eb0402faf8a7cf60cba7858233a96b80a81411f0d1`;
  baseline `fc9f01e03c199b5081169669603f2257a1177f40c4257da06e9c1c8cf3d8bfcc`.
- Initial unmodified script discovery reproduced the missing-verifier import error.
  Initial build failed because the executor lacked ALSA headers/pkg-config metadata.
  A new baseline-row assertion initially expected six rows; corrected to the actual
  two populations × six workloads and reran the complete suite successfully.

Rust workspace tests/Clippy and current-head remote checks are recorded in the PR
completion update. This tooling/link repair changes no runtime physics, GPU code,
assets or save format; no gameplay capture is required to establish its behavior.
This does not certify all scheduled graphical/platform/benchmark suites, historical
M1/Metal budgets, source rights or subjective presentation. No merge or deployment.
