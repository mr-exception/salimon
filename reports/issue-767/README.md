# #767 — Playable Scout / planet acceptance and mining readiness

Parent: #776, `issue-766-cockpit-handoff` at
`a3a36ec899e5608d65cda7e848d137a6dd5bd2b5`.
Tested source: `46cfa00904cae09986ab513cde441fb2b513ef7a`.
The unique child change adds one production browser journey and guides; it does
not introduce another game runtime. Final head/base and required CI are bound
in the published PR review comment.

`SLICE-1` starts at built `/`, walks outside, inspects both resource archetypes,
walks through the aft Scout entry and visible cabin to the cockpit, opens the
existing real 3D radar, sets nonzero throttle, exits standing in one action and
walks back to the outdoor starting area. The complete uninterrupted journey is
repeated in fresh browser contexts with actual input and read-only observations.

## Criterion matrix

| Criterion / category | Required verification | Actual status / evidence |
| --- | --- | --- |
| S1 built complete journey; E2E/integration | `npm run test:browser -- --project=cockpit --grep SLICE-1` | Pending final local run and current-head CI |
| S2 stable camera, entry/exit, frame and object association; domain/E2E | Same journey measures both handoffs, connected world, source identities; existing frame/collision/location tests | Pending final journey; parent domain suite has 59 PASS and final-source CI is tracked separately |
| S3 coherent presentation, exposed interior and reviewable originals; visual | Five fresh-context repeat comparisons under unchanged policy; wrong-checkpoint negative | Pending final local run; earlier failure retained below |
| S4 mining reuse and explicit future boundaries; documentation | Review canonical guide against actual owners and #757 | PASS; `docs/playable-2d-foundation.md`, client guide and architecture/index/root routing updated |
| Legacy 3D navigation, source reproducibility, collision/frame regressions | Existing full Navigation client CI, negative and repeat campaigns, baseline approval | Pending current-head CI |
| Build/typecheck | `npm run build` | PASS; `evidence/build.log` |
| Physical input / GPU / native / multiplayer | Separate future qualification, not required for routine cloud path | NOT RUN; no hardware/server claim |

All commands run in `client/`. Exact final local command:
`SALIMON_EVIDENCE_DIR=/tmp/767-final-browser npx playwright test --project=cockpit --grep SLICE-1`.
The production/test inputs and inherited rendered assets are indexed by hashes.
Node 24.19, Linux x64, lockfile Chromium 156.0.8078.4, 1280×800 DPR1, en-US/UTC,
controlled browser clock from 2026-11-04T00:00:00Z; CPU SVG and actual ANGLE Vulkan
SwiftShader WebGL. Five simulated seconds settle each capture. Full-page readback
retains the fixed viewer grid and whole UI; comparisons have no masks or tolerance
changes. Captures are review candidates, not promoted navigation references.

The test records the planet/world point and frame at each checkpoint, all raw
before/after point/box samples at the two doorway handoffs, final continuous ship
motion, source IDs/kinds/frames/positions/site, browser/backend and original PNGs.
Source interactions inspect inert placeholders and cannot extract or transfer.

## Failed runs and inherited repairs

`failures/viewport-capture/` retains the initial failed comparison: two pixels at
(697–698,576) on the storage label in the standing-return shot, despite identical
recorded transforms. A subsequent full-page run passed locally before the parent
changed; it is not reused as current-source proof or called a causal fix. #776
retains its related CI/local failures and repairs: vector capacity lettering,
exact terminal camera settling and an authored polygon avatar. This child was
rebased onto that completed source and its full journey is revalidated afterward.
No mandatory failure is waived, labelled flaky or hidden by adjusting policy.

## Reuse and unsupported future features

Mining #757 can reuse `TopDownClock`/`stepTopDown`, swept collision and range,
`requestInteraction`/`cancelInteraction`, both stable source IDs and site v1,
`capturePlayerLocation`/`copyPlayerLocation`, `convertAnchor` and `crossScoutEntry`.
Location snapshots keep exact split cells/offsets/facing in planet-fixed or
ship-local coordinates. They currently persist **in memory only**; #757 must
serialize/validate/restore durable state and implement transaction guarantees.

Not implemented: mining/harvesting extraction, depletion, loot/transfer or durable
inventory; actual Core energy; Fabricator production; multiplayer/backend and
reconnect; ship damage/destruction; full planetary generation. The pure inventory
domain still needs #757's whole-litre/integer-grams-per-litre migration and
independent 10 kg / 10 L capacity enforcement. The illustrative radar simulation
does not relocate the authored 2D landed Scout. These are explicit future features,
not a claim of a complete mining game or server.

## Active-session review

Implementing Codex agent, same-session review; no independent reviewer/execution
claim. Canonical guides and unique test diff reviewed against #767 and #757.
Final originals, measured journey and current-head CI still pending; overall
QA pending until the published review records ACCEPT or BLOCKED. Issue stays
open before merge; no merge/deployment is authorized by this report.
