# Issue #156 native continuation handoff

## Scope revision — 2026-10-13

The user approved the verified dense **500-fragment** route and settled timing
windows as sufficient for #156 now, and requested a review PR. The revised
acceptance is satisfied by the retained evidence; keep #156 open until merge.
The complete 1,000 route and remaining matrix are deferred to
[#165](https://github.com/mr-exception/salimon/issues/165); active collision
optimization remains in [#157](https://github.com/mr-exception/salimon/issues/157).
No large-load jobs need restarting for this PR. See the current decision at the
top of [the report](reports/issue-156/README.md).

The continuation instructions below are retained as historical handoff for
#165. Their “unfinished” and “remain open” statements describe the former full
matrix scope. The later user instruction authorizes the review PR and closure
of #156 on merge under its revised acceptance.

The user subsequently resumed a **focused 50-fragment check only**. That check
is complete: natural sleep, all 50 IDs/materials and 64 kg retained, zero drift
through fast rotating flight/landing/takeoff, and three small support/impact
regressions passed. The 50 fixture has no loaded bottom/dependent pair, so its
stack-removal/carrying precondition could not pass. Details and exact commands
are at the top of [the report](reports/issue-156/README.md). No large-load jobs
were restarted; original #156 acceptance remains incomplete. No source changes
were necessary for this focused continuation.

Updated 2026-10-13 (Asia/Nicosia). **Stopped at the user's request; issue #156 is
unfinished and must remain open.** All task-owned native jobs and waiting matrix
orchestrators were stopped. Resume only when the user requests continuation.

- Issue: <https://github.com/mr-exception/salimon/issues/156>
- Branch: `issue-156-native-completion`; push this branch, never main.
- Detailed results and retained evidence: [reports/issue-156/README.md](reports/issue-156/README.md).
- PR #164 is already merged. This branch is the subsequent native completion
  work; no new PR or completion comment was created in this session.
- #155 is closed; its reference hardware/matrix still governs #156 validation.

## Changes implemented

Production remains the custom Rust/winit/wgpu client. Native work repaired
support-aware contact-island sleep, loaded contact/friction behavior, exact SAT
projection reuse, and supported material-point rest observation. Stable IDs,
mass, visible/pickable sleeping fragments and ship-local precision remain intact.
The current physics implementation is commit `047105c`; later commits change
measurement guards, automation deadlines, the correctness harness and evidence.

The canonical sleep policy is in
[client/physics/architecture.md](client/physics/architecture.md). It observes
actual proxy vertices and COM through a supported 0.5 s dwell window with a
15 mm enclosing-radius budget, rejects excessive speed/spin/deep overlap and
does not give the first observation prior-time credit. It is not a blanket
freeze of low-speed airborne bodies.

Native measurement fixes include the Metal first-frame visibility backport,
optional minimum retained samples, gameplay input isolation during benchmarks,
startup occlusion warmup reset, and rejection of measured occlusion or drawable/
DPI changes. Original normal-game focus/cursor behavior is preserved.

Valid protocol step requests (1–600 frames) have a 120 s server deadline;
ordinary/malformed requests retain the shorter bound. The acceptance harness
uses matching bounded deadlines. Its floor-navigation changes use ordinary
look/walk/F inputs, route around the Core, try both strafe directions and face
an open floor gap before every release. They do not install poses or sleep flags.

## Verified results

- 353 workspace Rust tests passed; one existing GPU test ignored. Clippy with
  warnings denied, formatting and release staging passed.
- 55 Python tooling tests passed, including five ship-local look/axis/precision
  regressions. No source changed after those checks.
- All 17 standard native scenarios / 1,712 steps ultimately passed. The first
  sweep failed two assertions (carrying/resource-loop); both passed unchanged-
  binary retries. Their original cause was not isolated; failures remain recorded.
- Complete native dense **500** correctness passed: all sleeping after 1,056
  production updates; fast rotating 25,000 m/s flight/landing/takeoff preserved
  all IDs, 625 kg and zero local drift; real F removal of support 162 made
  dependent 238 fall; all 34 fragments outside that component stayed asleep;
  three pickup/drop cycles preserved identity/accounting.
- Full corrected native 100 route also passed. Corrected 1,000 carrying remains
  unverified: an older harness passed sleep/flight/support removal (515 → 743,
  11 unrelated sleeping fragments) but failed repeat-pickup identity. Its fresh
  floor-navigation retry ended with SIGTERM before readiness, not a physics or
  route assertion. Do not claim it passed.

Dense settled 500/1,000 performance both passed six native windows (three per
camera), with complete persistent sleep and conserved accounting. Identical-
snapshot solver comparisons exclude clone cost:

| Load | Stateless / sleeping median solver time | Sleeping integration / pair visits | Native median-interval FPS |
| --- | --- | --- | --- |
| 500 | 2,518.251 / 0.198813 ms | 0 / 0 | Facing 59.67–59.74; away 59.72–59.77 |
| 1,000 | 7,069.035 / 0.460667 ms | 0 / 0 | Facing 74.98–75.25; away 75.44–75.48 |

These are different recorded display cadences, not a comparable load/FPS curve.
500 is approximately the proposed 60 FPS target; neither dataset proves a
strict per-frame 60 FPS floor. Read the report for p95/p99/spikes and CPU/GPU
splits. Dense moving contact work remains expensive: active 250 facing repeats
included roughly 4.9–5.2-second p95 frame intervals dominated by physics.
Near-60-FPS medians after settling hide that initial-motion cost.

## Exact remaining matrix

Required matrix: counts 0/10/25/50/100/250/500/1,000 × scattered/dense/surface ×
settled/active × facing/away, three repeats with at least 100 retained native
frame intervals each: 96 distinct cases / 288 windows.

Retained: **221 valid windows**, comprising 73 complete cases / 219 windows,
plus two completed repeats of active dense 250 away. Archives beside the report
are committed and portable; original ignored `artifacts/` files are not pushed.

| Remaining selection | Missing windows |
| --- | ---: |
| Settled 500/1,000, scattered/surface, both views, three repeats | 24 |
| Active 250, dense away: third repeat | 1 |
| Active 250, surface, both views, three repeats | 6 |
| Active 500/1,000, all layouts, both views, three repeats | 36 |
| Total | 67 |

The active-250 partial archive contains the two usable dense-away repeats; keep
their provenance when adding the third independent fresh-process window.
All resumed windows after League of Legends restarted are excluded, including
one window that exited successfully while the other game consumed CPU/GPU.
Mixed moving/resting windows must not be reported as sustained active-only
tails; inspect moving sample counts rather than requested state labels.

## Continuing elsewhere

1. Read AGENTS.md, the issue/comments, affected owner guides and the detailed
   report. Do not repeat passed checks for unchanged source or replay old failed
   investigations as new acceptance.
2. Build a release executable for the continuation host. Current iMac executable
   SHA-256 was `95e662d5aeea7cec6a8913ba59adbfe51f84d70eaf5f3cfeb6315bf106cc2d68`;
   the executable and local app bundle live only in ignored artifacts. A rebuild
   may have a different hash: record actual revision, hash and hardware.
3. Run the current full 1,000 route. The harness is already committed and passed
   500; no additional speculative physics/navigation change is justified before
   observing the actual result.

   ```sh
   python3 scripts/build_game.py --profile release --output artifacts/build/release
   python3 scripts/fragment_sleep_acceptance.py --binary artifacts/build/release/salimon-client --counts 1000 --layout dense --settle-steps 2400 --timeout 14400 --request-timeout 120 --carrying --output artifacts/issue156-native-floor-navigation-1000-retry
   ```

4. Collect the missing native matrix on the reference Apple M1 iMac at 1920×1080,
   Metal, AutoVsync, diagnostics off, with Salimon unobscured and other games/
   video/heavy jobs stopped. Other hardware can establish correctness but cannot
   certify this reference performance. Record actual DPI and display cadence.
   Follow [scripts/FRAGMENT_BENCHMARKS.md](scripts/FRAGMENT_BENCHMARKS.md).
   For settled cases use `--settle-steps 2400 --warmup-seconds 1`; active cases use
   fresh processes and `--warmup-seconds 0`. Both use `--sample-seconds 5
   --minimum-samples 100 --repeats 3`. Primary/stress settled dense data already
   uses longer 10 s warmup / 30 s sampling and need not be repeated unchanged.
5. Keep preparation separate from timing. Prefer the checked-in standard runner
   with no competing heavy jobs; its preparation batches measure sequentially
   after all batch preparations finish. Preparation can be very slow and window
   durations extend until the minimum sample count is met. Do not substitute CPU
   replay/fixed-time correctness for native frame timing.
6. Promote exact completed results/hashes/raw archives into the report, document
   failures and fixture limitations, and only then publish a completion PR/update.
   Leave #156 open until acceptance passes; use closure on merge for a PR awaiting
   review. No main push or merge is authorized by this handoff.

## Avoid repeating these mistakes

- A local, uncommitted manual-gate wrapper lost naturally prepared fixtures when
  non-PTY stdin returned EOF. No measurement resulted. Interactive `MATRIX_START`
  input was subsequently verified using a PTY, but the final retry's native
  children were terminated before collection. Neither is a passed matrix group.
- Do not revive old waiting orchestrators or restore checkpoint sleep/poses.
  Recreate fixtures through production updates and respect user stop requests.
- Large correctness preparations were SIGSTOP/SIGCONT-paused during isolated
  measurement. Pauses count in wall deadlines but grant no simulation rest time.
  Excessive pauses can exhaust correctness deadlines. Avoid scheduling timing
  work in front of a nearly expired preparation.
- A matrix scheduler may be paused between windows, but never pause a measured
  native child and then claim its intervals are an uninterrupted benchmark.
- Keep speculative analysis short and act on actual failures. Repeated harness
  retries, poor scheduling and the EOF gate mistake materially extended this task.
- Do not quit unrelated games, expose their command-line credentials, or certify
  performance while they are consuming CPU/GPU. Inspect process names/CPU only.

All useful passing and failed evidence is linked from the report. Native
correctness remains fixed-step evidence; disk/backend persistence, networking,
orbital simulation, production energy management and a full-engine/broad-phase
refactor remain outside this issue.
