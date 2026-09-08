# Documentation maintenance gate — Universe U35 (#440)

Verification claims and issue closure use the
[cloud/human workflow](verification-workflow.md). Match recorded evidence to its
actual context and transfer human criteria explicitly; routine cloud PRs do not
wait for M1-only acceptance. This changes no same-PR documentation requirement.

This is a required author/reviewer gate for **every Universe U01–U34 implementation
PR**, including later fixes to those systems. It applies alongside the existing
[coding conventions](coding-conventions.md#tests-and-documentation) and
[completion workflow](coding-conventions.md#issue-completion-and-follow-up).
Documentation ships in the **same PR** as its implementation. The final audit
[#441](https://github.com/mr-exception/salimon/issues/441) cannot substitute for it.

## Before implementation

1. Read the active issue/accepted decisions and the affected source/tests. Use
   [technical architecture](technical-architecture.md) and the
   [maintenance map](maintenance-map.md) to identify canonical owners.
2. Identify each affected README, `README.ai.md`, `architecture.md`, `invariants.md`,
   domain contract and scenario/build/validation guide. Read only relevant owners.
3. Record documentation impact in the PR plan. Distinguish requirements approved
   for future delivery, behavior implemented now, historical fixtures, deliberate
   deferrals and native evidence. A closed portable issue is not native approval.

## Same-PR author gate

| Change | Required documentation review |
| --- | --- |
| New crate, module ownership, dependency or source path | Owning README/AI guide/architecture/invariants; root technical architecture and maintenance map |
| Schema, identity, units, frames, epoch, provenance or compatibility | Canonical domain/save/catalog contracts, producers/consumers, version/migration notes and fixtures |
| Physics, flight, resources, scanner or timed/offline behavior | Owning invariants, cross-system gameplay contracts and affected scenarios; preserve approved rules |
| New command, input, build/test/config parameter or asset interface | User/contributor usage, validation/build commands and asset regeneration guide where applicable |
| Implemented/deferred status changes | Project phases, relevant feature documentation and implemented-status notes in approved design; retain the approved requirement record |
| Evidence, limitations or unresolved acceptance | Per-issue report with actual checks, failures/unrun gates and linked follow-up ownership |

For every PR, include a **Documentation impact** section with:

- `Updated` and actual changed guide paths/links, or an explicit `No impact`
  exception with the source paths, exact reason and how unchanged behavior was
  established. Listing a report alone does not document changed contracts.
- Links to the affected **owning architecture and invariants**, and a short
  mapping from changed behavior to source/test and its canonical guide. Include
  root routing documents when ownership changes.
- Superseded statements corrected, remaining TODO/deferred/native boundaries,
  the link/path checks performed and the per-issue report.

A small mapping is sufficient; avoid restating whole contracts in a PR:

| Changed behavior | Source/test evidence | Canonical guide |
| --- | --- | --- |
| Describe the resulting behavior | Exact module/test paths | Owning architecture/invariants or contract links |

Remove contradictory current descriptions in affected guides. Keep useful old
reports/engineering fixtures explicitly historical; do not rewrite evidence.
Update approved-design implementation notes without weakening approved future
requirements. Do not claim a full universe, scanner, offline model, scientific
pack or reference-machine performance from one foundation PR.

## Precise no-impact exceptions

An exception is allowed only when a guide's behavior/API/ownership/usage remains
accurate. For example, a new regression test can identify its unchanged contract
and prove that no public behavior, source ownership or command changed. A private
refactor must explain why documented paths/interfaces and invariants remain
accurate. A changed rule, schema, module owner or command requires actual guide
changes; “docs later,” “covered by #441,” “internal” or an unexplained “N/A” is
not an exception. The reviewer can reject an insufficient exception.

## Readiness and reviewer gate

Do not mark a PR ready until its documentation-impact section is complete and
paths/local links/anchors resolve at the final revision. Read changed source and
tests against the guide: a resolving link does not prove semantic accuracy.
Confirm supported commands/configuration and preserve failures/native limits.
Missing or contradictory documentation, unresolved local links or an unjustified
exception requires changes before approval/merge.

[PR template](../.github/PULL_REQUEST_TEMPLATE.md) and
[Universe issue form](../.github/ISSUE_TEMPLATE/universe-implementation.yml)
carry this checklist. Existing U01–U34 issues already include the U35 acceptance
requirement. Authors and reviewers apply this gate now. This PR establishes a
**review-enforced policy**, not an automatic branch-protection check, approval or
repository-permission change.

## Automation and final integration ownership

- [#442](https://github.com/mr-exception/salimon/issues/442) implements offline CI
  checks for links/paths, owned guides, scoped stale terminology and verifiable
  examples with positive/negative fixtures and documented exclusions. It depends
  on this policy; it must not claim semantic correctness from lint alone.
- [#441](https://github.com/mr-exception/salimon/issues/441) performs the final
  implemented-code-to-doc audit after U01–U33, with evidence and explicit blockers.
  It remains independent of U34 to avoid a dependency cycle.
- U01–U34 authors remain responsible for the same-PR updates and report for their
  own implementation. Completing #440 installs this ongoing contributor rule;
  it does not certify future PRs or bypass unfinished native validation.

## Current foundation example

[U01 schema architecture](../client/universe/architecture.md) and
[invariants](../client/universe/invariants.md) document the delivered isolated
schema, bounded fixture JSON, exact IDs and provenance validation. The
[technical architecture](technical-architecture.md#canonical-universe-schema-foundation-402)
and [maintenance map](maintenance-map.md#canonical-universe-schema-402) route its
owners. [U01 report](https://github.com/mr-exception/salimon/blob/d003bd58d506e04e3df97744f29f68449b6e2b3e/reports/issue-402/README.md) records portable validation;
production ingestion/packs, playable migration and native checks remain deferred;
U02 frames and the bounded U04 prototype now have their own implementation notes. A versioned definition reference is not a full
published planet or verified scientific dataset.

## Offline CI validation (#442)

`python scripts/check_docs.py` checks maintained root/docs/crate/tooling Markdown
local inline links, reference definitions and ATX heading anchors (including
Unicode and duplicate headings), required owned paths, scoped stale-contract
patterns and the frozen schema-1 JSON envelope. Code fences and remote URLs are
not resolved. Historical reports, vendored docs, model authoring and asset READMEs
are outside this maintained-guide scope; they remain the author's responsibility.
Raw code snippets, dynamic paths/globs, Setext headings and arbitrary Markdown/HTML
are not interpreted. This is a bounded linter, not a full Markdown renderer.

PR CI also passes `--base <base-sha>`: changed Rust/Cargo crate boundaries require
all three owning guides and an owning Markdown update. A precise source-pattern
exception with nonempty reason may be reviewed in `scripts/docs-policy.json`;
this mechanical test does not decide whether an update/reason is sufficient.
`--write-baseline` is an explicit maintainer debt-review command, never run by CI;
there is currently no accepted baseline debt. Never automatically accept findings.
Run `python -m unittest discover -s scripts -p test_check_docs.py -v` for positive
and deliberately broken fixtures. CI emits file/line errors and fails on findings.
The source job now includes documentation validation; branch protections and
semantic review remain maintainer-owned.
