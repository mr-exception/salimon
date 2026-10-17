# Issue 268 — persistence verification scope

## Outcome

Prepared the session-only scope clarification for owner review. The existing
Phase 0 persistence boundary is unchanged: streaming restoration in a running
world session is supported; disk save/quit/relaunch restoration is unimplemented.
The owner decision remains pending in [Project Q&A](../../docs/project-qa.md#what-does-236-require-by-savereload).

## Changes and review decision

- Added the canonical [reload verification scope](../../client/world/resource-contracts.md#persistence-and-reload-verification-scope-268), linking the two native evidence scenarios and the historical #236 report.
- Linked the architecture overview to that single scope definition.
- Defined future disk-persistence prerequisites without introducing a save format or backend.

Approving this PR selects in-session streaming unload/reload for #236's
“save/reload” requirement, excluding cross-process restoration from this wave.
The proposed GitHub #236 scope amendment is: replace “save/reload” with
“in-session streaming unload/reload (same running world session; disk saves and
save/quit/relaunch restoration are outside this wave)”. Keep the rest of #236
and its native-only execution unchanged. Apply that issue amendment and remove
the pending Q&A question once the owner approves; do not retroactively rewrite
the dated native report or claim new native evidence.

## Validation

Documentation-only change; no runtime, scenarios or historical evidence modified.
Inspected `MiningSession` ownership/restoration in `client/world/src/mining.rs`,
the two evidence scenarios, and the #236 report against the proposed wording.

- `git diff --check`: passed.
- Changed Markdown relative link/anchor checks: passed (Python 3.12.14).
- Rust builds/tests and native Apple M1 runs: not run; no executable behavior
  changed. Existing #236 evidence is referenced, not reproduced or recertified.

## Limitations

Owner approval is required to resolve #268's product decision. No new disk-save
implementation or native persistence certification is delivered by this PR.
