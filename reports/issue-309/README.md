# Issue #309 — portable gameplay-session composition

Implemented on 2026-10-18, based on `4e5c1d7` (main). Prerequisites #301,
#302, #306 and #307 were completed and included in this base.

## Delivered

- `GameplaySession` owns composed controllers, equipment, resource/cargo state,
  held controls, typed commands, typed snapshots and portable fixed-tick stepping.
  The existing update order, backlog retention, assist/power rules, immediate
  mining releases, streaming identities and interaction gates are preserved.
- `ClientApplication` owns native admission, lifecycle, renderer/window lifetime,
  bounded catch-up scheduling, action-bar presentation/expiry and evidence IO.
  Portable ship/surface frames and interaction gates moved under `session/`.
- `ResourceGameplay` clearly owns fragment motion/sleep/history/scratch, ship
  anchors, cached queries and cargo-ledger synchronization. `MiningTool` now
  contains only held mouse/F input; authoritative extraction remains in world.
- E2E and benchmark fixtures construct the production session directly.
  `execute_session`/`inspect_session` support headless protocol regressions;
  native automation adds window/capture/GPU observations to the same route.
- Headless benchmark collection now advances the full production session, with
  separate physics measurements, rather than a fragment-only simulation. Reports
  mark `update_scope: gameplay_session_v1`; historical total CPU timings are not
  directly comparable. Isolated solver baselines, fixture settling and component
  microprofiles retain their deliberate narrower measurement scopes.
- Runtime architecture, invariants, maintenance routing and benchmark guidance
  were updated. No new engine, ECS, event bus or domain/renderer dependency added.

## Checks

Environment: Linux x86_64; rustc 1.99.1, cargo 1.99.1; debug profile.

| Command/check | Final result |
| --- | --- |
| `cargo fmt --all -- --check` | Passed |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | Passed |
| `cargo test --workspace --locked` | Passed: 506 tests, zero failures; 22 existing opt-in tests/doc-tests ignored |
| `cargo test -p salimon-client --locked native_protocol_adds_only_shell_observations_and_expires_session_feedback` | Passed: one additional shell/session integration test added after the full suite |
| `cargo build --workspace --locked` | Passed |
| `target/debug/salimon-client --benchmark --mode physics --count 10 --warmup-seconds 0 --sample-seconds 1` | Passed; ten fragment identities/mass conserved, full-session scope observed; [summary](headless-smoke.json) |
| `git diff --check` | Passed |

The full suite includes new typed-command versus protocol equivalence through
blackout, mechanical Core intake, mass removal, conversion/power restoration and
11 ms fractional backlog; steering/reset/view/throttle gates; and actual ship
advancement through benchmark session setup. Existing mining, carrying, streaming,
500-object cargo, takeoff/landing, lifecycle and deterministic scenario regressions
now exercise the session. The additional shell regression checks portable snapshot
parity, native observation fields and transient expiry through explicit steps.
Initial migration compile/test failures were corrected; no final failed check remains.

## Limits / native follow-up

Native window/GPU smoke, OS event delivery, screenshots, Apple M1 visuals and FPS
were not run in the cloud. #312 and #313 remain pending native follow-ups; this
PR does not close either. Ignored CPU experiments were not promoted to native
verification. The debug executable smoke is a path/conservation check, not a
performance acceptance result. In-memory session state is unchanged; disk/network
persistence remains outside this issue.
