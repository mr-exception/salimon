# Issue #762 — player movement, collision and interactions

The existing strict-overhead development world now has collision-aware WASD
walking, persistent facing and a smoothed follow camera. Visible walls, a closed
aft door, furniture and a crate replace the invisible corridor clamp. Entity-local
metre footprints feed frame-aware swept queries; conservative 0.46 m player
clearance prevents tunnelling and local-axis motion allows wall sliding.

The Core and cockpit fixtures expose stable-ID nearby selection, explicit reach,
action labels, cyan outlines and no-op inspection request/cancel. Standing,
walking and interacting state includes the active target. Invalid requests give
feedback; repeat requests do not create effects. Movement, target loss, Escape,
blur, hidden-page state and resize cancel inspection. Focus changes clear held
movement; editable fields retain their keys. Inspect buttons return keyboard
control to the world.

Base: `main` at `ecae30f8a59ede6f5b15e597579b615145490e74` (#761 via #768).
No active parent PR existed; branch `issue-762-player-interactions`.
The final PR binding comment records exact head/base and current CI/review.
[Input hashes](evidence/source-hashes.json) bind local source/test bytes; these
are not claims of a future-head CI pass.

| Acceptance | Local verification / evidence |
| --- | --- |
| AC-1 movement, collision, selection, valid/invalid interaction | 10 focused top-down domain cases cover sweeps, high-speed blocking, rotated frames, tangency/away motion, detour, walls, closed door, disabled/missing/distant target, stable selection, request/repeat/cancel/target loss. Browser TD-3 walks the detour using keyboard input, reaches both consoles, inspects/cancels via keys and buttons. [Collision](evidence/crate-collision.png), [Core inspection](evidence/core-inspection.png), [cockpit reached](evidence/cockpit-reached.png). |
| AC-2 smooth follow, stable facing, determinism | Fixed-step cadence equivalence, normalized movement, retained facing and bounded camera movement/settling in domain tests. Browser TD-2 observes movement and facing at rest. TD-1/4 compares two fresh-run SVG screenshots byte-for-byte, with explicit floor/player/rock pixels and layer ordering. [Original baseline candidate](evidence/top-down-baseline-0.png). |
| AC-3 focus-loss release | Browser TD-2 verifies release, blur, resize, visibility and focus cleanup plus editable-control isolation; TD-3 verifies blur cancels active inspection. |
| AC-4 unit + browser + screenshot evidence | `npm test`: **48 PASS**, zero skipped; `npm run build`: **PASS**. Focused Playwright: **4 PASS**, zero skipped. [Unit log](evidence/unit.log), [build log](evidence/build.log), [browser log](evidence/browser.log), [raw JSON](evidence/browser-results.json). Full navigation, negative/repeat comparisons and baseline approval run in existing current-head CI. |

## Reproduction and environment

```sh
cd client
npm ci --cache /tmp/salimon-npm-cache --no-audit --no-fund
PLAYWRIGHT_BROWSERS_PATH=/tmp/salimon-browsers npx playwright install chromium
node --experimental-strip-types --test --test-isolation=none src/topDown.test.ts
npm test
npm run build
PLAYWRIGHT_BROWSERS_PATH=/tmp/salimon-browsers SALIMON_EVIDENCE_DIR=test-results/issue-762-final npm run test:browser -- --project=top-down
```

Debian 13 Linux x64, Node 24.19.0, npm 11.9.0; locked Playwright 1.64.1 /
Chromium 156.0.8078.4. 1280×800, DPR 1, en-US/UTC, dark/reduced motion;
fixed fake clock epoch 2026-11-04. Existing top-down CPU SVG raster settings and
vendored Fontconfig are unchanged. Radar preflight used actual ANGLE Vulkan
SwiftShader, [capture](evidence/radar-preflight.png). Browser keys/mouse are
synthetic input through the actual DOM adapter; there are no state setters.
Position/facing DOM attributes expose observations only. This does not qualify
physical input, hardware performance, native or mobile platforms.

## Failures and scope

The first focused browser run passed 3 cases but the long keyboard journey
exceeded the default 30-second test timeout. [Log](failures/initial-browser.log)
and [original](failures/journey-timeout.png) are retained. The steering helper now
batches up to 65 ms of actual keyboard input per step and the complete journey
has a 90-second budget; the final journey passed in 19.6 seconds. Earlier focused
domain iteration caught a fixture detour clearance/waypoint issue; final layout
and local-frame route now pass. No mandatory check, assertion or pixel tolerance
was removed. Button focus/cancel handling was also corrected before final checks.

This remains the single development fixture under `/?scene=top-down`. Closed aft
door and indoor start are intentional until #763–#765 implement the full Scout,
planet and crossing. Cockpit inspection does not launch radar; #766 owns seating
and navigation handoff. No mining, Core effects, persistence, multiplayer or
full world content is claimed. Production radar entry and approved navigation
baselines are preserved; these new SVG captures are review candidates only.

Independent reviewer: distinct Codex subagent `/root/review_762`, cycle 1.
Preliminary independent source/test/original-image review found no blocking issue;
final verdict is pending published-head CI and the evidence packet. CI validates
the session-produced review record and does not run an AI model. Human retains
merge authority. No merge, deployment or release is performed.
