# Issue #440 — same-PR documentation maintenance gate

Delivered 2026-10-21, stacked after U01 #402. The canonical
[U35 policy](../../docs/documentation-maintenance.md) now defines mandatory
same-PR author/readiness/reviewer checks for every U01–U34 implementation/fix:
actual owning architecture/invariants links, changed behavior/source/test/guide
mapping, affected README/root/domain/validation updates, precise reviewed
no-impact exceptions and implemented/future/historical/native distinctions.
Missing or contradictory docs require changes before review approval/merge.
The final #441 audit cannot substitute for individual updates.

A new PR template and required-field Universe issue form carry the checklist.
AGENTS, coding conventions, source routing, technical architecture, validation,
project phases and approved-design implementation status link the canonical rule.
The touched approved design now correctly records the implemented #493 engine-off
switch while keeping the Jump Drive panel future. No approved gameplay formula
or requirement, runtime code, asset, save format or branch protection is changed.

This delivers **review-enforced contributor policy**, not automatic CI enforcement
or semantic correctness. Continued author/reviewer application is ongoing policy,
not a blanket certification of future PRs. Automated links/paths/stale-contract
checks remain [#442](https://github.com/mr-exception/salimon/issues/442), now able
to follow the installed policy once this PR merges. Final consistency audit remains
[#441](https://github.com/mr-exception/salimon/issues/441), blocked on relevant
U01–U33 implementation and independent of U34. Those existing scoped issues own
remaining automation/audit delivery; no duplicate workflow was introduced.

## Validation

- `python reports/issue-440/check_added_links.py --base b6bc22dcef3405badab29625780d637b7ea2b65d`:
  added local Markdown targets/anchors checked; zero errors. [Output](links.log).
  The bounded evidence helper is **not** U37's repo-wide lint; external URLs,
  unchanged links and semantic accuracy are outside its checks.
- Parsed `.github/ISSUE_TEMPLATE/universe-implementation.yml` with PyYAML;
  checked supported top-level fields, unique field IDs, required documentation/
  acceptance/scope/blocker text areas and required completion checkboxes: passed.
- Confirmed the PR template includes the Documentation impact section and mapped
  changed contract paths back to actual source/guides: passed. Root contributor
  links, U01 owning guides and current engine switch were inspected.
- `git diff --check`: passed.
- Rust/native/GPU/Blender checks: not rerun for this documentation/template-only
  layer. The preceding #402 layer passed workspace build, 775 tests, formatting
  and all-target Clippy; no code/asset/dependency changed above it. No screenshot
  or native approval is claimed.

The issue/PR body must link this report and the delivered policy. Closing #440 on
merge records policy installation; future PRs still owe their own evidence.
