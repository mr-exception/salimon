# #766 — Cockpit integration

Tested final production source: `b33122f14a1edfc1a93a49a9c1c3b582cbb35719`.
Parent: `main` at `232bc3082cbfdf57254685cf29080f37ff1604f8`.
PR: [#776](https://github.com/mr-exception/salimon/pull/776).
Final published head/base, required CI and final review are bound in its comment.

Default development and built `/` start in the persistent 2D landing site. Nearby
cockpit interaction reserves the owner's one chair, seats the avatar at its
ship-local anchor and mounts the existing radar with the same navigation runtime.
Voluntary exit stands beside the chair in one action. A development power fixture
returns seated, retains occupancy and never reopens on power restoration.
Standing without power releases the chair. The SVG world stays mounted and ship
simulation continues in either view, preserving throttle, velocity and position.

`cockpit.ts` owns local admission/reservation/power; `TopDownView.tsx` composes
presentation and clocks; `App.tsx` restores throttle/scanner state on mount.
Isolated foundation inspections remain development fixtures; `/?scene=radar`
retains legacy navigation. Client, ownership and cockpit guides are updated.
No approved navigation references were changed or promoted.

## Verification matrix and commands

All commands run in `client/`.

| Criterion / category | Mandatory check | Actual result / evidence |
| --- | --- | --- |
| C1 default 2D → real radar → standing, same world; integration/E2E | `npm run test:browser -- --project=cockpit` | Original COCKPIT-1 passed twice at a8c0a1c; final-source diagnostic walked the same default outdoor approach and radar/standing transition twice, PASS. Full final-source CI tracked on PR. |
| C2 seated blackout, explicit restoration, unpowered stand; E2E/domain | COCKPIT-1 and `npm test` | PASS before rendering-only changes; seating/power code unchanged. Originals/measurements in `evidence/`; final full coverage tracked in CI. |
| C3 one occupant and owner-only control; unit | `npm test` | 59 PASS, including exact camera settling. `evidence/render-repair/unit.log`; cockpit authority tests unchanged by subsequent illustration-only change. CI reruns all tests at final head. |
| C4 continuous position/velocity/throttle and legacy radar; integration/E2E | COCKPIT-1 and full Navigation client CI | Local measurements PASS at 37% throttle, including exact SI trajectory over observed tick delta. Initial CI passed cockpit and all legacy radar cases. Final head checked separately by CI. |
| C5 deterministic originals/repeats; visual | COCKPIT-1, PLANET-1, TRAVERSE-1 and existing visual/negative checks | Initial CI/reproductions FAIL, retained below. Final-source focused viewport/full-page repeats PASS with zero pixels above unchanged policy. Required complete current-head CI remains pending until PR review comment records it. |
| Build/typecheck | `npm run build` | PASS at b33122f; `evidence/render-repair/build.log`. |
| Physical hardware / network multiplayer | Separately owned future qualification | NOT RUN; no server, remote-player or physical-input claim. Routine cloud delivery uses synthetic input/software WebGL. |

Original local command: `SALIMON_EVIDENCE_DIR=/tmp/766-browser-pass npx playwright test --project=cockpit`.
Post-failure focused command: `SALIMON_EVIDENCE_DIR=/tmp/766-fixed-browser npx playwright test --project=top-down --project=cockpit --grep 'PLANET-1|TRAVERSE-1|COCKPIT-1'`.
The camera-only version of that run was interrupted after the avatar mismatch
recurred; uncompleted traversal is not counted as passing.
Final renderer diagnostic: `SALIMON_EVIDENCE_DIR=/tmp/766-raster-polygon npx playwright test --config=playwright.raster.config.ts`.
Its archived config/spec are under `evidence/render-repair/` as `.txt`; they are
not an added gate. The ordinary required CI retains all complete functional,
visual, negative, repeat and baseline-approval checks.

Local environment: Node 24.19, Linux x64, locked npm/Chromium 156.0.8078.4,
1280×800 DPR1 en-US/UTC. SVG uses CPU rasterization; real radar uses ANGLE Vulkan
SwiftShader WebGL 2. Browser clock starts 2026-11-04T00:00:00Z, inputs are actual
Playwright keyboard/mouse events and exact observed fixed ticks, with no gameplay
setters or forced rendering. Final diagnostic capture variants share originals
where hashes match; `capture-hashes.json` records that mapping. Final source/test/
asset/config hashes are in `evidence/render-repair/source-hashes.json`.

## Failures and rendering repair

`failures/ci-41982777624/` retains initial CI originals, observations and diffs for
head 8493105 / merge 92ee124: PLANET-1 failed by 3 pixels at (798–800,231),
TRAVERSE-1 by 1 at (677,385), with identical recorded transforms. Those points
land on the tiny storage-capacity lettering. It now uses explicit vector ink;
the capacity is also retained in the HUD and accessible label.

`failures/vector-label-only/` and `failures/settled-camera-only/` retain further
local failures, including four avatar-edge pixels at (638–641,376). A nanometre
camera settle cutoff now finishes at the exact player anchor, with a regression
test for zero residual offset; that alone did not remove the mismatch. The small
avatar now uses authored polygon contours instead of browser-tessellated curves.
Final focused fresh-context captures at default zoom pass the unchanged policy
for viewport/full-page first and second captures. This does not establish that
capture mode alone was a cause or fix. Final broad CI is still required.

Earlier HMR reload, frozen-clock canvas initialization and live-physics observation
failures remain under `failures/`. The ship check now verifies the exact expected
motion over measured tick delta rather than freezing flight between observations.
Large redundant traces were omitted; essential original PNGs, logs, numerical
measurements and diffs are retained. No threshold, mask or approved reference was
relaxed. The final source is not claimed to pass a suite that was run only earlier.

## Review and limits

Implementing Codex agent, same-session review: source/criteria and actual original
radar, standing, seated and renderer diagnostic evidence inspected. Functional
C1–C4 evidence is valid for unchanged code; C5 final diagnostic passes. **Overall
QA pending required current-head CI**; final ACCEPT/BLOCKED is recorded on the PR.
No independent reviewer or external AI execution is claimed. Issue stays open
until merge. The owner retains merge authority.

Only local chair/control state is implemented. Multiplayer authority/sync,
reconnect/disk persistence, Core energy, ship damage and animations remain future
work. The illustrative radar ship simulation continues but does not relocate the
authored 2D landing pose. Power fixtures are development-only. Synthetic input
and software WebGL do not qualify physical GPUs, hardware input or M1/Metal.
