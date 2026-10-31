# Issue #434 — asynchronous verified resume

Implemented after #432/#433 on `codex/offline-universe-five-tasks` (parent
`8753724`), for the requested single combined PR. Shared final acceptance and
publication follow #441/#435.

Normal startup creates its native window/menu while one worker reads and
reconstructs the saved journey, catches up ship/timed systems and verifies the
candidate. The worker sends **verified source bytes**, not thread-local domain
caches, to the main thread. Gameplay remains locked until durable atomic
checkpoint replacement and actual current-region CPU/GPU/stream readiness.

The menu shows real completed/total admitted elapsed time and a catch-up percent
only when calculable. Reading, verification, publication and current-region
preparation have explicit stage text, without invented completion percentages.
Pending/error stages expose no Continue action. Errors keep the last recoverable
checkpoint and show Retry/Quit. Local clock rollback or unsupported dates cannot
rewind a journey or grant an approximate return. Saved admitted tick backlog
survives menu wall time and current-region loading.

Cancellation is checked before publication and during bounded physical work.
An interrupted attempt never exposes its private candidate. Restart uses the
existing verified two-slot store; it creates no public staging/orphan save slot.
Clock errors are reported from the worker so the menu can remain responsive.
Already-started atomic IO owns its receipt; no false success is inferred from a
thread finishing or a progress value reaching 100%.

## Verification

Linux cloud, Rust/Cargo 1.99.1, locked debug profile. Actual native renderer uses
Xvfb/Mesa llvmpipe Vulkan, separate from physical hardware qualification.

- `cargo test -p salimon-client --locked launch`: **26 passed**, including
  actual durable bytes/slot counts, cancel/restart/no replay, incompatible
  world/motion publications, invalid/backward clocks, setup failure/retry,
  pre-publication rejection, denied early Resume, current-region admission and
  preserved backlog. A measured actual local-clock absence (>1 second) is
  distinct from controlled elapsed fixtures.
- Runtime all-target Clippy with `-D warnings` and locked debug build: **passed**.
- Native progress: **22/22 steps passed** (80.229 seconds), actual takeoff,
  held partial propagation at 1.7%, denied early Resume, durable publication,
  current-region readiness and continuation. Two held-state native PNGs have
  identical SHA256 hashes.
- Native backward-clock/retry: **16/16 steps passed** (69.209 seconds), including
  readable error, denied Resume, actual-clock retry and continuation.
- [Capture provenance and original trace index](native-evidence.json) records
  actual binary/source hashes per capture; compressed JSON traces are lossless.
  Original PNGs are retained here, including [partial progress](offline-progress.png)
  and [clock error](offline-clock-failed.png). These are evidence, not promoted
  visual baselines. Shared final-head acceptance is indexed by #435.

The first async compile correctly rejected sending GameplaySession's Rc query
cache between threads. The implementation now transfers validated serialized
source and reconstructs caches on the native thread; no unsafe Send or broad
cache refactor was added. A historical starter-supply fixture reopened a real
current-clock terminal restart at Unix second 100; it now uses the actual restart
time, retaining strict clock admission. An initial screenshot probe used an
invalid checkpoint name and a later run found a dead Xvfb socket; corrected the
fixture name and started a dedicated live Xvfb display. These setup failures do
not count as passing native evidence.

## Reproduction and ownership

[Native menu/worker contract](../../docs/universe-offline-propagation.md#native-progress-and-verified-resume-434)
maps launch, checkpoint, timing and region readiness owners. Maintained routes:
[progress](../../scenarios/evidence/offline-resume-progress.json) and
[clock recovery](../../scenarios/evidence/offline-clock-recovery.json).

The `offline_launch`/`launch_release` E2E operations seed only a new isolated
fixture directory. Their optional bounded hold exposes real private progress for
GPU capture and early-Resume rejection; controlled-clock status is explicit.
`SALIMON_E2E_LAUNCH_DIR` is an explicit canonical-journey probe for real-clock
restart of that isolated store. Normal play never enables automation or a fixture
gate from these environment settings alone.

No native workstation, Metal, physical input, OS suspend/wake or M1 FPS is claimed
(#557/#563); subjective presentation remains #561. No merge/deployment is done.


## Shared batch acceptance

Subsequent implementation and final cloud evidence are complete in the same
combined PR: [integrated results, original media and limits](../issue-435/README.md).
That final index supersedes earlier pending-batch statements while preserving
historical focused-check revisions/counts above. No merge or hardware acceptance
is implied. The audit precedes integrated verification; this link is additional
evidence, not a dependency from #441 to #435.
