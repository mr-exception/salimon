# #767 — Playable Scout / planet acceptance and mining readiness

Base: `main` at `3eb7968125f17196a093b74e81c389623ea89a30`, after the owner merged
#776 with #778's Docker delivery. No merge was performed by this chat. There are
no other active PRs at preparation. The second incremental PR therefore targets
main; all Docker files/workflow are preserved byte-for-byte.

Tested built journey source: `286565efcff450ce974499e58c7fe49ab2577371`. Follow-up renderer source was reapplied as `94b9ea1`; its
runtime/config inputs match the earlier four-check source `ed84b15` by hash.
Guide/report-only edits do not change those executed paths. The final published
head/base, CI and exact review are bound in the PR comment.

`SLICE-1` starts at built `/`, walks outside, inspects both resource archetypes,
walks through the aft Scout entry and visible cabin to the cockpit, opens the
actual existing 3D radar, sets 15% throttle, exits standing in one action and walks
back to the starting area. The entire uninterrupted journey repeats in fresh
contexts through real keyboard/mouse adapters, with no gameplay setters.

This PR also completes visual qualification left by #776: original authored SVG
art is cached into fixed-resolution PNG sprites and moved by the same SVG/world
adapter. No second simulation/runtime, changed collider or source identity is
introduced. Camera precision/readiness and bounded trace capture support repeat
verification. Root/client/architecture/ownership and the playable guide are aligned.

## Criterion matrix

| Criterion / category | Required verification | Actual outcome / evidence |
| --- | --- | --- |
| S1 complete built production journey; E2E/integration | `SLICE-1` | PASS twice; `evidence/slice-journeys.json`, checkpoint originals |
| S2 continuous camera, entry/exit, interactions and frames; domain/E2E | Exact 0.04 m crossing samples, <3 pixel motion, one transfer each way; same connected world and source IDs | PASS; raw before/after points/boxes and source records in journey JSON |
| S3 coherent illustration, simultaneous cutaway/exterior, reviewable originals; visual | Five repeat captures under unchanged policy; wrong-checkpoint negative | PASS; all comparisons and diff PNGs retained; negative rejects the wrong view |
| S4 mining reuse and explicit future boundaries; documentation | Review actual owners and #757 against canonical guide | PASS; `docs/playable-2d-foundation.md` |
| Existing cockpit power/control/ship continuity and planet/collision/frame regressions | Four affected browser cases + domain suite | PASS locally; #766 `evidence/final/` has 59 units and four cases; current runtime/config hashes match |
| Legacy radar, negative regressions, repeat visuals, baseline approval | Full current-head Navigation client CI | Pending final CI; no old-head pass relabelled |
| Docker production smoke | Inherited current-head Docker client smoke workflow | Pending final CI; Docker source remains unchanged |
| Build/typecheck | `npm run build` | PASS; `evidence/build.log` |
| Physical input / GPU / native / multiplayer | Separate future qualification, not required for routine cloud path | NOT RUN; no hardware/server claim |

Commands run in `client/`:

```sh
npm run build
CI=1 SALIMON_EVIDENCE_DIR=/tmp/767-merged-base npx playwright test --project=cockpit --grep SLICE-1
```

The final full journey passes in 1.7 minutes, with two complete replays. The local
Node 24.19/Linux x64 run uses lockfile Chromium 156.0.8078.4, 1280×800 DPR1,
en-US/UTC, fixed browser clock from 2026-11-04T00:00:00Z, SVG image rendering and
ANGLE Vulkan SwiftShader WebGL. Five simulated seconds settle each capture.
Full-page readback retains the fixed viewer grid and whole UI. There are no masks,
tolerance changes or promoted navigation references. Traces retain actions,
filmstrips and sources; explicit state JSON avoids repeatedly snapshotting cached
image resources.

`evidence/` retains original pairs/diffs for spawn, mineral inspection, exterior
with simultaneously exposed interior, standing return and return outdoors. It
also retains the exact checkpoint/frame records, both raw handoff sample arrays,
final continuous ship motion, stable source IDs/kinds/frames/points/site,
browser/backend/input metadata, command logs and source hashes. The original
captures were reviewed in this active chat by the implementing agent; coherent
palette, readable cutaway/objects and physical seated/standing states pass.
These are reviewed candidates, not human-promoted navigation references.

## Failures and original evidence

`failures/viewport-capture/` retains the initial two-pixel standing-return failure.
`shared-browser-core-edge/` and `fresh-browser-process/` retain subsequent one-pixel
exterior failures and identical transforms; process isolation did not fix them.
A prior full-page run passed before the renderer changed and is not reused as
current-source proof. Related CI/local experiments and raw originals remain in
the #766 report. Art-specific, layout/readback, GPU and filter experiments did not
establish acceptance; the original illustration was restored. The final fixed
asset cache then passed all four affected cases and this complete journey.
No mandatory failure was waived or called flaky.

## Mining reuse and unsupported future features

#757 can reuse `TopDownClock`/`stepTopDown`, swept collision and range,
`requestInteraction`/`cancelInteraction`, both stable source IDs and site v1,
`capturePlayerLocation`/`copyPlayerLocation`, `convertAnchor` and `crossScoutEntry`.
Location snapshots retain exact split cells/offsets/facing in planet-fixed or
ship-local coordinates. They persist **in memory only**; #757 must implement
serialization/validation/restoration and durable transaction guarantees.

Not implemented: mining/harvesting extraction, depletion, loot/transfer or durable
inventory; actual Core energy; Fabricator production; multiplayer/backend and
reconnect; ship damage/destruction; full planetary generation. The pure inventory
domain still needs #757's whole-litre/integer-grams-per-litre migration and
independent 10 kg / 10 L capacity enforcement. Radar sample flight does not
relocate the authored landed 2D Scout. No other game runtime is needed for mining.

## Review

Implementing Codex agent, same-session review; no independent execution/reviewer
claim. Final unique diff, criteria, actual journey/ship/frame measurements and
original captures inspected. S1–S4 PASS locally. Overall QA pending required
current-head Navigation/Docker CI; final ACCEPT/BLOCKED is recorded on the PR.
Issue #767 stays open before merge. No merge/deployment is performed here.
