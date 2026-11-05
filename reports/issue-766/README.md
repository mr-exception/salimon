# #766 — Persistent world / cockpit / radar handoff

PR: [#776](https://github.com/mr-exception/salimon/pull/776).
Tested production source: `ed84b1559a66cd9eacebc7f087e7ffb2c5ac3dbf`.
Base: `main` at `232bc3082cbfdf57254685cf29080f37ff1604f8`.
The final report-only head/base, current-head CI and review verdict are bound in
the PR comment. The source matches the verified working tree apart from guide
edits and a trailing-newline cleanup; final-head CI checks the exact commit.

The default development/built `/` starts outside the Scout in the persistent 2D
world. Nearby cockpit interaction reserves the owner's one chair and physically
seats the avatar at its ship-local anchor before mounting the existing 3D radar.
Voluntary exit stands beside the chair in one action. A development outage fixture
returns seated and retains occupancy; restoration waits for explicit interaction.
Standing releases the chair even without power. The world stays mounted and the
same ship runtime preserves position, velocity, orientation, throttle and scanning.

`cockpit.ts` owns local seat/control/power state. `TopDownView.tsx` composes the
world and navigation clock ownership; `App.tsx` restores runtime controls/scanner
state on mount. `TopDownIllustration.tsx` retains the original SVG shapes/colors.
`topDownSprites.tsx` initializes and reuses fixed-resolution PNG blob assets
(64 texels/metre, terrain 32). It does not own gameplay or collision state.
Sprite readiness is observable; ticks live outside the painted SVG. Exact camera
settling below a nanometre prevents residual observation offsets. Client/root,
architecture, ownership and cockpit guides describe the final boundaries.

## Required verification matrix

All commands run in `client/`.

| Criterion / category | Check | Actual result |
| --- | --- | --- |
| C1 default world → real radar → standing; integration/E2E | COCKPIT-1, connected world and ship-local chair assertions | PASS locally, twice; `evidence/final/cockpit-measurements.json` and originals |
| C2 seated blackout, restoration without reopening, unpowered stand; E2E/domain | COCKPIT-1 and `cockpit.test.ts` | PASS locally, twice; blackout/restored PNGs and domain log |
| C3 single occupant and owner-only control; unit | `npm test` | 59 PASS; `evidence/final/unit.log` |
| C4 preserved nonzero simulation and existing navigation; integration/E2E | COCKPIT-1 verifies 37% throttle and exact SI trajectory over observed ticks | PASS locally; previous CI passed all legacy radar cases; full final-head CI tracked on PR |
| C5 original/repeat visuals, source/frame/camera stability; visual/E2E | TD-1/4, PLANET-1, TRAVERSE-1, COCKPIT-1 | All four PASS locally after final renderer change; unchanged policy, no masks/reference promotion |
| Build/typecheck | `npm run build` | PASS; `evidence/final/build.log` |
| Complete cloud gates | Navigation client CI: domain/build, all browser projects, negative mutations, repeat visuals, baseline approval | Pending until final PR review records current-head results |
| Physical input / GPU / M1/Metal / network multiplayer | Separately owned qualification; not required for routine cloud path | NOT RUN; no hardware/server claim |

Final local commands:

```sh
npm run build
CI=1 SALIMON_EVIDENCE_DIR=/tmp/766-sprites-final npx playwright test --project=top-down --project=cockpit --grep 'TD-1|PLANET-1|TRAVERSE-1|COCKPIT-1'
npm test
```

Four browser cases pass in 3.9 minutes, including complete fresh-context repeats,
actual keyboard/mouse input, WebGL radar, collider stops, both doorway handoffs,
original PNG comparisons and wrong-checkpoint negatives. Browser originals,
per-checkpoint transforms, raw handoff samples, ship states, comparator outputs,
build/unit/browser logs and source/test/asset/config hashes are under
`evidence/final/`. PNG sprite initialization retains logical IDs and footprints;
physical frame/location tests remain in the unchanged domain suite.

Local: Linux x64, Node 24.19, lockfile Chromium 156.0.8078.4, viewport 1280×800,
DPR1 en-US/UTC, controlled browser clock from 2026-11-04T00:00:00Z. World uses the
SVG image adapter and CPU rasterization; cockpit WebGL is ANGLE Vulkan SwiftShader.
No gameplay setters or forced rendering observers are exposed. World failure
traces keep actions, filmstrips and sources; explicit JSON keeps relevant state
without repeatedly serializing immutable image resources into DOM snapshots.
The approved navigation capture configuration and references are unchanged.

## Failed evidence and repair

Initial local HMR, canvas initialization under a paused clock and live-motion
observation failures are retained under `failures/`; they are not passes.
The motion check verifies the actual elapsed tick trajectory rather than freezing
simulation between observations. CI runs 41982777624 and 41985715193 retained
required repeat-image failures despite matching scene transforms; the second
passed 31 of 32 browser cases. Their original pairs, state and pixel diffs are
in the corresponding failure directories.

The early localization to particular tiny labels was too narrow: subsequent
failures also involved other edges. Art-specific changes, fresh browser processes,
layout/readback variants, GPU-enabled and filtered-surface experiments did not
establish complete acceptance. Art/readback/filter experiments were reverted;
the original authored illustration is preserved. Logs and useful originals remain
under `failures/`; large redundant/broken traces are omitted.

The final repair caches the authored art at a fixed backing resolution, then
moves/rotates the same PNG sprites with the existing SVG/world adapter. Repeated
movement no longer rerasterizes vector/font edges at changing transforms. The
first cache prototype overloaded trace recording; bounded blob references and
action/screenshot traces with explicit state JSON restored verification cost.
All four affected cases then passed, including both previously failing journeys.
No screenshot threshold, mask, approved reference, mandatory route or failure was
waived. Historical successful captures outside `evidence/final/` describe their
recorded source, not the final renderer.

## Same-session review and limits

Implementing Codex agent, same-session review: final source, criteria, actual
measurements and original radar/standing/blackout/planet/doorway captures reviewed.
C1–C5 PASS locally. Overall QA awaits required current-head CI; its final
ACCEPT/BLOCKED verdict is recorded in the PR comment. No independent reviewer or
external AI execution is claimed. Issue stays open before merge.

No multiplayer authority/sync, reconnect/durable persistence, actual Core energy,
ship damage, animations or flight-driven 2D relocation is implemented. The radar
sample's ship motion continues while its authored landed 2D pose stays fixed.
Outage controls are development-only. Asset cache/initialization adds browser
presentation work and fixed-resolution art; it is not a hardware FPS or arbitrary
zoom qualification. No merge, deployment or release is performed.
