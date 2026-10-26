# Issue #645 — warning-only architecture and PR review

Delivered against merged #638–#642 (`2f222bb`). The read-only CI inventory includes
open/draft PRs with paginated file lists and head SHAs. The review compares the
actual PR base to the working tree, emits path-specific crate direction, new
cross-domain/import, new cycle, broad-file and shared-path candidates, and retains
JSON/text artifacts. Explicit unavailable inventory never means zero competitors.

The policy freezes observed crate edges, carries review thresholds and exact
owner/reason exceptions. The maintained [architecture policy](../../docs/architecture-baseline.md#warning-only-boundary-and-active-pr-review-645)
defines ownership, false positives, escalation, warning-to-enforcement review,
stack choices and how to measure future overlap. Warnings and inventory failures
do not reject existing valid builds; tooling contracts still fail on regressions.

Validation (Linux x86_64, Python 3.12):
- `python3 -m unittest discover -s scripts -p 'test_architecture_*.py' -v`: 9 passed.
- `python3 -m unittest discover -s scripts -p 'test_*.py'`: 129 passed.
- `python3 scripts/architecture_review.py --base origin/main --output artifacts/architecture/review.json`: passed, zero source candidates for this tooling-only diff; live inventory unavailable locally. Fixture tests cover permitted/forbidden edges, new/existing cycles, comments, broad files, exact exceptions, draft overlap, self exclusion and unavailable data/base.
- `python3 scripts/check_docs.py --base origin/main`: zero failures.
- `git diff --check`: passed.

No gameplay/GPU changes. Rust runtime checks are not required for this tooling
change. Live GitHub Actions/API execution will run after publication; local
fixtures do not establish future CI/API success or reduced real merge conflicts.
The scanner remains lexical (cfg/macros/aliases can yield false positives or
misses); same-path overlap does not prove conflicting hunks. No failing architecture
policy gate is introduced. No PR was merged.
