# Recover merged Scout lighting and acoustics onto main

The three PRs merged successfully, but only #608 targeted `main`. #610 merged
into `codex/580-room-lighting`; #609 subsequently merged that combined work into
`codex/543-construction-materials` after #608 had already reached `main`.
Consequently #543 is closed, while #580/#583 remain open and their delivery is
absent from `main`.

## Recovery

This branch starts at `b8ecf52` on `main` and applies the exact missing delta
from `b805743` (the merged #609 result). The recovered branch tree equals the
original final three-PR stack for all gameplay source, scenarios, tooling and
original evidence. The eight newer planetary-environment documentation files
from #611 are preserved byte-for-byte. No new gameplay changes or reconstruction
of the already delivered features are needed.

[Branch routing, source hashes and preservation audit](branch-audit.json)
records 123 recovered source/scenario/tool/evidence files with matching SHA-256
hashes. The canonical lighting/acoustics guides are restored with their original
reports and native/audio evidence. Historical stacked instructions in those
reports remain historical; this recovery PR targets `main` directly and closes
both #580 and #583 on merge.

Original delivery evidence:

- [Lighting report, native screenshots and measurements](../issue-580/README.md).
- [Tactile audio report, native observations and WAVs](../issue-583/README.md).
- [Construction report and approved economy](../issue-543/README.md), already on main.

Human visual/listening acceptance, actual physical devices and hardware budgets
remain #561/#557/#563. Construction progress/geometry remain #544/#545 as in the
original delivery. Recovery of the reviewed implementation grants no additional
baseline, platform or hardware approval.

## Validation

- `cargo test --workspace --locked`: passed (400 runtime, 102 ship, 81 renderer
  and the remaining domain suites; existing ignored lanes retained).
- `cargo clippy --workspace --all-targets --locked -- -D warnings`: passed.
- `cargo fmt --all -- --check`, `git diff --cached --check`: passed.
- `python3 scripts/check_docs.py`: passed, zero failures.
- `python3 -m unittest discover -s scripts -p test_salimon_test.py`: 21 passed.

Exact [test](tests.log.gz), [Clippy](clippy.log.gz), [format](fmt.log.gz),
[documentation](docs.log.gz) and [Python runner](python.log.gz) logs are retained.
Original native evidence is retained rather than rerendered, since the recovered
implementation is byte-identical to the delivered stack. No new physical device
or hardware measurements are claimed.
