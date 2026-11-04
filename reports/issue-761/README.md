# Issue #761 — unified overhead runtime

Implemented a development-only `/?scene=top-down` entry with one illustrated SVG
scene for the Scout, player and planetary fixtures. The roof has no renderable
cover. WASD walks a bounded interior corridor while the camera follows. The
ordinary development and shipped production entries retain the independent radar.

Source revision: `e2ebe3d016725463dc014d749a4a07f0f6e21229`.
Base: `main`, `c13d14b29e69973df9f7df63ca4bde157b1cac19`; no active parent PR.
Branch: `issue-761-top-down-runtime`. The PR/comment binds final head/base, CI and
independent review. Report-only commits reuse the unchanged client inputs recorded
in [source-hashes.json](evidence/source-hashes.json); they are not old-head CI passes.

## Criteria and verification matrix

| Criterion / category | Mandatory procedure and expected result | Local result / evidence |
| --- | --- | --- |
| AC-1 / integration + E2E | `npm run test:browser -- --project=top-down`: single scene, anchored player/terrain/Scout, ordinary radar and built radar entry preserved | PASS, 3 tests in [results](evidence/top-down-results.json) |
| AC-2 / unit | `npm test`: fixed-clock cadence replay, normalized movement, camera centered, stable unique IDs, input/time failure and corridor boundaries | PASS, all 44 domain tests, [log](evidence/unit.log) |
| AC-3 / unit | Same command: rotated/translated pose round trips; unchanged passenger anchor and planet objects; huge split cells and near-cell-boundary sub-mm differences; astronomical planet pose cancels from camera output | PASS, six new domain tests; [no-isolation confirmation](evidence/unit-no-isolation.log) |
| AC-4 / unit + visual | Pure deterministic draw list; actual SVG layer order and floor/player/rock pixel counts; exact byte equality of two fresh-run original PNGs, no masks or tolerated differences | PASS, [observations](evidence/observations.json), [first baseline capture](evidence/top-down-baseline-0.png), [second](evidence/top-down-baseline-1.png), [whole view](evidence/test-finished-1.png) |
| AC-5 / build + regression E2E | `npm run build`; existing domain and all navigation/browser routes; current-head Navigation client CI including negative/repeat/approval jobs | Build PASS [log](evidence/build.log); 24 unchanged browser cases PASS; affected top-down 3 PASS after raster fix. Remote current-head CI/review pending at report creation; see PR binding comment |
| Manual / hardware | N/A: no physical-input, device-specific, performance or mobile behavior is required by this foundation | Playwright input is synthetic; no hardware claim |

The complete initial browser run was **26 PASS / 1 FAIL** (exact new SVG
comparison). After the top-down-only raster setting changed, the focused final
project was **3 PASS / 0 skipped**. All other browser inputs/settings were unchanged.
The full clean current-head suite is delegated to the existing deterministic CI,
including its expensive navigation negative and repeat comparisons. No check was
removed or weakened. Local source/type build was rerun after the final config edit.

## Environment and reproduction

Debian 13 Linux x64; Node 24.19.0, npm 11.9.0; locked Playwright 1.64.1 / Chromium
156.0.8078.4; viewport 1280×800, DPR 1, en-US / UTC, dark / reduced motion.
Authored fixture has no RNG; fake epoch 2026-11-04T00:00:00Z, fixed 60 Hz runtime.
Existing radar preflight used ANGLE Vulkan SwiftShader. SVG captures use Chromium
CPU rasterization (`--disable-gpu`), with the same pinned vendored Fontconfig.
This is the actual production SVG adapter, not a mocked image or graphics fixture.

```sh
cd client
npm ci --cache /tmp/salimon-npm-cache --no-audit --no-fund
PLAYWRIGHT_BROWSERS_PATH=/tmp/salimon-browsers npx playwright install chromium
npm test
node --experimental-strip-types --test --test-isolation=none src/*.test.ts scripts/*.test.ts
npm run build
PLAYWRIGHT_BROWSERS_PATH=/tmp/salimon-browsers npm run test:browser
SALIMON_EVIDENCE_DIR=test-results/top-down-verified PLAYWRIGHT_BROWSERS_PATH=/tmp/salimon-browsers npm run test:browser -- --project=top-down
```

Additional local no-isolation execution confirmed all 44 individual cases after
an initial restricted executor invocation reported only file-level subtests.
Networked commands use the authorized executor network grant and inherited proxy.
Initial npm/browser installation failed because the default home cache directory
was unavailable; explicit writable `/tmp` caches resolved it without lock changes.

## Failures retained and resolved

- Paused fake clock initially withheld React's delayed lazy Suspense commit.
  Boot now advances the clock while waiting for the actual scene. [Initial log](failed-capture/suspense-boot.log).
- SwiftShader SVG captures drifted at 52 antialiased edge pixels despite unchanged
  scene state. [Original A](failed-capture/top-down-baseline-0.png), [B](failed-capture/top-down-baseline-1.png), [raw diff](failed-capture/raw-diff.png), [full run](evidence/initial-browser.log).
  The top-down-only CPU raster path makes repeated fresh runs exact; no tolerance,
  mask, geometry or navigation reference was changed.
- An overlapping repeat reused a preview server that the completed full suite
  shut down, causing `ERR_CONNECTION_REFUSED` in its production-entry case.
  [Failure](evidence/overlap-preview-failure.log). The isolated final project passed
  all three cases; that failed overlap is not counted as a pass.
- Branch protection inspection returned GitHub 403 `Resource not accessible by
  integration`; visible repository rulesets are empty. Existing workflow gates
  remain mandatory; no assertion of unknown protection settings is made.

## Ownership, scope and independent review

`topDownFrames.ts` owns explicit anchor conversions using existing split positions;
`topDownRuntime.ts` owns immutable state, IDs, fixed steps, follow camera and layer
ordering; `TopDownView.tsx` / `topDown.css` own illustrated SVG and DOM input.
Client usage, technical architecture, maintenance routing and approved-design
implementation notes were updated together. No dependency, lockfile, radar
simulation, navigation baseline or mandatory CI script changed. New source/guide local
link paths were checked ([results](evidence/docs-links.json)); broad historical-guide
scanning also found pre-existing missing native paths after the browser migration,
outside this issue. New section anchors were inspected against actual headings.

The planetary universe pose is abstract metadata and cancels from local rendering;
ship origin/rotation changes carry ship-local passengers without changing IDs.
Automatic cross-frame transfer, planet traversal, collision/interaction systems,
seating, cockpit transitions, persistence and multiplayer remain later work.
These new captures are deterministic baseline candidates, not promoted approved
navigation references. [Unchanged radar sample](evidence/radar-visible-contacts.png)
and [zero navigation pixel diff](evidence/radar-visible-contacts-diff.png) retain
representative regression evidence; full eight-view originals remain in CI artifacts.

Independent reviewer: distinct Codex subagent `/root/review_761`, cycle 1.
Preliminary independent source/domain/original review found no blocking code issue;
final verdict must bind the published head/base and completed current CI. Status at
report creation: **QA pending**. CI validates the session review record; it does
not run an AI model. The owner retains merge authority; no merge/deployment occurred.
