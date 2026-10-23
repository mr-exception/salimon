# Task reports

Task-result evidence lives here; maintained specifications and guides live in
`docs/`. Follow the mandatory [completion-report convention](../docs/coding-conventions.md#task-completion-reports).

The [verification workflow](../docs/verification-workflow.md) defines one narrow
exception: a fully evidenced human review requiring no repository changes can
close with a durable issue update instead of a report file/PR. Repository changes
and all automated implementation/verification deliveries retain reports and PRs.

- GitHub tasks: `issue-<number>/README.md`.
- Tasks without an issue: `task-<id>/README.md`.
- Place applicable screenshots/images and useful state/logs beside each report;
  link them with relative paths. Record outcome, changes, exact validation results,
  skipped checks, environment and limitations. Link the report from the issue update.

When unresolved points remain, link their new follow-up issues from the report
and close the source issue through the delivering PR, following the canonical
[issue completion workflow](../docs/coding-conventions.md#issue-completion-and-follow-up).
Keep failed or incomplete validation visible when transferring the remaining scope.

The migrated `issue-*` and `task-*` directories preserve historical validation
from earlier revisions. Older `task-*` reports retain their original
`validation.md` filenames so external evidence links and chronology remain
traceable; new reports use `README.md`. Historical geometry, task ordering and
performance results do not define the current implementation. Do not rewrite
past passing/failing evidence to match a newer revision. Disposable generated
runs remain under ignored `artifacts/`; promote useful evidence deliberately.

## Duplicate evidence cleanup

The historical [alias manifest](https://github.com/mr-exception/salimon/blob/7e8946ee75d7f9be1158118109b26d47f96445ec/reports/evidence-aliases.json)
recorded two byte-identical PNG mappings for issues #29 and #34. The manifest
and those report folders were intentionally removed by the
[2026-10-06 old-report cleanup](https://github.com/mr-exception/salimon/commit/9b0a24526e2da337efdac2beb31c525e7fbb27d1).
There is no live alias index or retained target for these mappings in the current
tree. Use the immutable historical manifest/image references and
[verified provenance report](issue-297/README.md) to inspect those bytes; all
recorded digests were checked against the original and retained Git blobs.
Historical capture filenames in archived logs describe their recorded revision,
not files guaranteed to exist in the current checkout. Keep useful current
evidence and its validation limitations unchanged.
