# Issue completion and follow-up policy

## Outcome

On 2026-10-17, the owner requested that the current PR close its source issue
and move unresolved points to new follow-up issues instead of leaving the
entire source issue open. This documentation change establishes that workflow
for future issue delivery.

The canonical rule lives in
[the coding conventions](../../docs/coding-conventions.md#issue-completion-and-follow-up).
[AGENTS.md](../../AGENTS.md) replaces its former acceptance-only closure rule
with that workflow. The [document index](../../docs/README.md),
[validation guide](../../docs/validation.md), [report guide](../README.md) and
[Project Q&A](../../docs/project-qa.md) link to it. The Q&A's active instruction
to keep #255 open was replaced with follow-up tracking at delivery.

The delivering PR uses a closing reference for the source issue. New follow-ups
carry the unresolved criteria/decisions, evidence and blockers, with links in
both directions. Dependency references move to the follow-up when needed.
Failed, skipped and unrun checks remain visible; issue closure does not grant
technical acceptance or approve unresolved decisions.

## Validation

Documentation-only checks on the changed Markdown files:

- New and modified relative link/path checks: passed, including the new canonical
  section anchor. A full-file scan found one pre-existing broken reference,
  `reports/evidence-aliases.json`; its repair is tracked separately in
  [follow-up #297](https://github.com/mr-exception/salimon/issues/297). All other
  scanned links resolve. See [link-check output](links.log).
- Workflow consistency search: passed. The former rule is removed from
  AGENTS.md and active Q&A guidance. Historical reports and the explicitly
  historical `ISSUE-156-HANDOFF.md` retain their original dated dispositions.
- `git diff --check`: passed.

Rust/Python tests, builds, native runs and screenshots were not run: there are
no runtime, tooling, asset or visual changes. No new tests mirror the prose.

## Scope and limitations

This PR updates project instructions. It does not migrate existing GitHub
issues, change their acceptance results or rewrite historical reports.
The existing missing evidence-alias manifest is transferred to
[follow-up #297](https://github.com/mr-exception/salimon/issues/297), with evidence
and acceptance criteria; it does not block delivery of the policy change.
