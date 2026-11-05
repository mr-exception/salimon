# #766 — Cockpit integration

Tested source: `a8c0a1c6de621cd5317ece9fb6290ea40c45fa08` (the final helper whitespace cleanup changes no execution).
Parent: `main` at `232bc3082cbfdf57254685cf29080f37ff1604f8`.

Default development and built `/` start in the persistent 2D landing site. The
owner's nearby cockpit interaction reserves one chair, seats the avatar at the
ship-local chair anchor and mounts the existing radar with the same navigation
runtime. Voluntary exit stands beside the chair in one action. A development
power fixture returns seated, retains the reservation and never reopens on power
restoration. Standing without power releases the chair. The SVG world stays
mounted and ship simulation continues in either view.

`cockpit.ts` owns local admission/reservation/power; `TopDownView.tsx` composes
presentation and clocks; `App.tsx` restores throttle/scanner state on mount.
Isolated foundation inspections remain development fixtures; `/?scene=radar`
retains the legacy standalone navigation route. No approved reference changes.

## Verification matrix

| Criterion | Required path/command | Result / evidence |
| --- | --- | --- |
| C1 default 2D → radar → standing, same world | `npm run test:browser -- --project=cockpit`, built entry/legacy CI | PASS locally; `evidence/browser.log`, measurements and PNGs; broad current-head CI tracked on PR |
| C2 power loss seated, restoration explicit, unpowered standing | same browser test plus `npm test` | PASS; `evidence/unit.log`, `cockpit-measurements.json`, blackout/restored PNGs |
| C3 single occupant and owner-only control | `npm test`, `src/cockpit.test.ts` | PASS; `evidence/unit.log` |
| C4 simulation position/velocity/throttle continuity and legacy radar | cockpit measurements, Navigation client CI | PASS locally; `evidence/cockpit-measurements.json`; broad current-head CI tracked on PR |
| C5 original/repeat visuals | cockpit repeated PNGs, unchanged comparator | PASS; all four repeats have zero changed pixels under policy; `evidence/*-repeat.json` |
| Physical hardware / network multiplayer | Not required for routine cloud delivery | NOT RUN; no server or physical-input claim |

Commands use Node 24.19 locally, locked npm dependencies, Linux x64 and desktop
Chromium, 1280×800 DPR1 en-US/UTC. Cockpit captures use the locked Chromium,
SVG CPU raster and actual SwiftShader WebGL. Browser clock starts at
2026-11-04T00:00:00Z; movements use actual Playwright keyboard events and observed
fixed ticks, with no state setters.

Failures are retained under `failures/`: an initial HMR reload invalidated a
walking run; the next frozen-clock run reached seating but needed explicit clock
advancement to initialize the newly mounted R3F canvas. A subsequent replay exposed a normal one-tick live-physics observation gap; the corrected check verifies the exact SI trajectory over observed ticks instead of requiring motion to stop. Original failure screenshots/log/results are retained; large redundant traces were omitted. Typecheck initially
caught missing extracted helper exports; corrected before browser verification.

## Limits

Only local chair/control state is implemented. Multiplayer authority/sync,
reconnect/disk persistence, Core energy, ship damage and animations are future
work. The illustrative radar's ship simulation continues, but does not relocate
the authored 2D landing pose. Power fixtures are development-only. Synthetic
browser input/software WebGL do not qualify physical GPUs or hardware input.

## Active-session review

Implementing Codex agent, same-session review: C1–C5 PASS locally; inspected the unique source diff, actual measured ship motion and original standing/blackout/radar PNGs. Local evidence ACCEPT; overall QA pending required current-head Navigation client CI. No independent reviewer or external AI execution is claimed. No navigation references were changed or promoted. Final published head/base, CI and final verdict are bound in the PR comment. Issue remains open pending merge.
