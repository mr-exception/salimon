# PR #567 merge conflict resolution — 2026-10-26

[PR #567](https://github.com/mr-exception/salimon/pull/567) originally delivered
Scout M1 lighting evidence and timing preparation. Its evidence and runtime
logging were subsequently delivered through merged PR #571; matched cloud
measurements followed through #594. Issues #483 and #566 are already closed.
This update reconciles the original branch with current `main` at
`24710fe0376cc197b35f9ec5fcdf176e43310187`, preserving both parents without
rewriting the original branch history.

## Resolution and preservation

All eight conflicted paths retain the current `main` versions:
`docs/scout-presentation.md`, `reports/issue-463/README.md`,
`reports/issue-483/{README.md,SHA256SUMS,frame_data.py,frame_reference.py,test_frame_data.py}`
and `scripts/README.md`. This preserves cloud admission profiles, unchanged-source
and context checks, repeated image hashes, fuller distributions and failure
reporting, plus newer canonical guidance and delivered cloud evidence.
The automatically merged runtime source and architecture also exactly match main.

Before the manifest repair below, the resolved repository tree was byte-identical
to main (`git diff --quiet origin/main` and `git diff --cached --quiet origin/main`).
All 286 original PR paths remain present: 276 are byte-identical to the original
head `45fac8573a180cd7fc262da180eab9ef107d352b`; ten guides/source/tool/manifest
files have the newer main revisions. Every original capture, offline mood image,
raw log and rejected timing artifact remains byte-identical.

The inherited 280-entry evidence manifest had four stale entries after subsequent
main updates: `README.md`, `frame_data.py`, `frame_reference.py` and
`test_frame_data.py`. Refresh only these four SHA-256 digests to the retained
current files. The remaining 276 entries already matched. Historical evidence
bytes, failures and recorded validation outcomes are unchanged.

## Validation

Linux x86_64, Python 3.12.14. Commands run from repository root:

- `python3 scripts/check_docs.py --base origin/main`: passed, zero failures.
- `python3 -m unittest discover -s scripts -p 'test_*.py'`: 120 passed.
- `python3 -m unittest discover -s reports/issue-483 -p 'test_*.py' -v`:
  six passed, including strict native and explicit cloud admission rejection cases.
- `cd reports/issue-483 && sha256sum --check --quiet SHA256SUMS`: passed,
  all 280 entries after the four-entry manifest repair.
- `git diff --cached origin/main --check`: passed for the final change.
- Local relative links in this report: checked against existing files.

Rust build/test/Clippy, graphical capture, Blender rerender and hardware timing
were not rerun locally: the final change relative to main is this report and
four manifest digests; all Rust source, assets, scenarios and Python tooling are
identical to main. GitHub PR CI validates the pushed merge revision separately.

## Documentation impact and limits

Current [Scout presentation](../../docs/scout-presentation.md),
[runtime architecture](../../client/runtime/architecture.md) and
[invariants](../../client/runtime/invariants.md), and
[renderer architecture](../../client/renderer/architecture.md) and
[invariants](../../client/renderer/invariants.md) are preserved from main.
No runtime behavior, interface, ownership, shader, physics or asset change is
introduced. Existing frame-log behavior maps to `client/runtime/src/app.rs`,
its canonical runtime architecture and `scripts/README.md`; admission behavior
maps to the retained `frame_data.py` and its six tests above.

[Original and cloud evidence](../issue-483/README.md) and
[matched cloud measurements](../issue-566/README.md) retain their original limits.
Two rejected M1 foreground attempts remain rejected, with zero accepted M1 timing
windows. Human appearance/baseline approval remains
[#561](https://github.com/mr-exception/salimon/issues/561), hardware budgets
[#563](https://github.com/mr-exception/salimon/issues/563), and physical platform
checks [#557](https://github.com/mr-exception/salimon/issues/557). Readiness for
merging this reconciliation grants none of those approvals. The PR is left open
for the user's merge.
