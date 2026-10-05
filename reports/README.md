# Task reports

Task-result evidence lives here; maintained specifications and guides live in
`docs/`. Follow the mandatory [completion-report convention](../docs/coding-conventions.md#task-completion-reports).

- GitHub tasks: `issue-<number>/README.md`.
- Tasks without an issue: `task-<id>/README.md`.
- Place applicable screenshots/images and useful state/logs beside each report;
  link them with relative paths. Record outcome, changes, exact validation results,
  skipped checks, environment and limitations. Link the report from the issue update.

The migrated `issue-*` and `task-*` directories preserve historical validation
from earlier revisions. Older `task-*` reports retain their original
`validation.md` filenames so external evidence links and chronology remain
traceable; new reports use `README.md`. Historical geometry, task ordering and
performance results do not define the current implementation. Do not rewrite
past passing/failing evidence to match a newer revision. Disposable generated
runs remain under ignored `artifacts/`; promote useful evidence deliberately.

## Duplicate evidence cleanup

[evidence-aliases.json](evidence-aliases.json) maps two removed byte-identical
PNG copies to their retained files with SHA-256 digests. The issue #29 failure
image and cabin image had identical bytes; issue #34's initial/return cockpit
images also matched. Report links now use the retained image. Original structured
results/logs remain unchanged and may contain historical capture filenames; use
this alias index to locate those bytes. Distinct failed-run or older-layout
evidence is retained because it records useful validation limits.
