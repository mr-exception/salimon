# Issue #764 — fixed planetary exploration area

The shared overhead development world now starts outside the Scout in a fixed
fictional living-world landing area. Flat geological strata, boulders/stones and
sparse alien plate colonies provide a coherent illustrated sample. Physical
props and both mineral/biological source archetypes have stable planet-fixed IDs,
anchors and footprints. Shared nearby inspection exposes each source; it never
extracts, depletes, transfers, carries or spawns resources.

Parent: PR [#772](https://github.com/mr-exception/salimon/pull/772), branch
`issue-763-scout-cutaway` at `8a3e27367c608e3e2ad30436a09c403e909e494f`.
Child: `issue-764-planetary-site`; tested source `9b89116b5f908d9aafcf180af0e82cf72222c80f`.
#761/#762 are completed in this ancestry; #772's current-head CI and same-session
review passed. The child targets the parent's head branch; merge #772 first.
The unique child diff excludes the delivered Scout implementation/report.
Final head/base and CI/review are bound in the PR comment. Report-only changes
preserve [recorded client input hashes](evidence/source-hashes.json).

| Criterion / category | Checks and durable evidence |
| --- | --- |
| AC-1 outdoor landing area, movement and collision | Unit tests assert a clear default outdoor spawn, physical rock/source blocking and unchanged fixed entities under movement. PLANET-1 walks from outdoor spawn to mineral, biological colony and near Scout via actual keyboard input, verifies collision stops and retains the same world DOM node. [Spawn](evidence/planet-spawn-0.png), [near mineral](evidence/planet-near-rock-0.png), [near colony](evidence/planet-near-colony-0.png), [near Scout](evidence/planet-near-scout-0.png). |
| AC-2 repeatable fixed region and anchors | Two fresh browser contexts repeat the journey with exact observed fixed-tick input durations; IDs, source kinds/site IDs/frames/positions remain identical before/after the route. Domain tests prove placement does not depend on ship/camera pose. [Source observations and comparisons](evidence/browser-results.json). |
| AC-3 stable resource interaction hooks, no extraction | `planetarySite.ts` owns `site:illustrated-landing:v1`, invariant planet-relative spawn/Scout pose and source kind/archetype metadata. Both source types are reachable via existing selection/range/inspection. Repeated and distant requests are tested; source entities remain unchanged and no inventory/depletion/loot implementation is introduced. |
| AC-4 E2E and visual comparisons | **6 local browser cases PASS**, zero skipped. [Comparison summary](evidence/comparison-summary.json). Spawn, near-mineral, near-colony and near-Scout originals/replays use existing `compareImages` and unchanged `POLICY` (0.1 perceptual threshold, zero pixels over threshold, antialiasing included, no masks). Raw changed pixels and difference images retained. A deliberately wrong checkpoint must be rejected. The existing static TD-1/4 byte-equality assertion remains. |
| Unit / integration / build | `npm test`: 52 PASS, zero skipped; `npm run build`: PASS, existing large-chunk warning. [Unit](evidence/unit.log), [build](evidence/build.log). Existing Scout/control browser routes remain part of the focused suite; [browser log](evidence/browser.log). Broad Navigation/WebGL, negative/repeat and baseline-approval checks run in current-child CI. |
| Manual/hardware | Same-session implementing Codex `/root` reviews source, unique diff, actual results and original captures; no independent execution claimed. Physical input, native/mobile/hardware FPS and final subjective art qualification NOT RUN, outside this cloud issue's scope. |

## Reproduce and environment

```sh
cd client
npm ci --cache /tmp/salimon-npm-cache --no-audit --no-fund
PLAYWRIGHT_BROWSERS_PATH=/tmp/salimon-browsers npx playwright install chromium
npm test
npm run build
PLAYWRIGHT_BROWSERS_PATH=/tmp/salimon-browsers SALIMON_EVIDENCE_DIR=test-results/764-final-policy npm run test:browser -- --project=top-down
```

Debian 13 x64, Node 24.19.0/npm 11.9.0, locked Playwright 1.64.1 /
Chromium 156.0.8078.4, 1280×800/DPR 1, en-US/UTC, dark/reduced motion,
vendored fonts/Fontconfig, fixed fake clock epoch 2026-11-04. Headless SVG CPU
raster is the actual 2D adapter. Built radar isolation uses ANGLE Vulkan
SwiftShader, with the same production bundle as the parent. Browser keys are
synthetic physical-adapter input; no player/world/camera setters are exposed.

## Failures and visual policy

Initial typechecking found mutable inferred arrays in two test variables; explicit
state annotations fixed them. Early visual-driver experiments required PNG-byte
identity after camera movement and found differences. Exact fixed-tick input
removed approximate endpoint variation. Subsequent recorded observations show
identical player and camera transforms while SVG edge antialiasing still differed
(e.g. 2,144 raw pixels with maximum channel difference 8). Extra settling, viewport
recovery, isolated contexts and partial-raster disabling did not establish byte
identity. Manual tracing setup also conflicted with Playwright's existing automatic
tracing; the duplicate setup was removed.

The final visual check reuses the repository's established comparator without
changing its policy, masks, thresholds, approved references or negative checks.
It reports raw differences honestly and rejects every pixel above the prescribed
threshold, including antialiased pixels. It also rejects a different real camera
checkpoint. These are repeat-comparison candidates, not newly approved reference
baselines. Temporary rendering-coordinate rounding, viewport recovery and raster
flag experiments were removed; gameplay coordinates retain their full precision.
[Failure logs, original pair, diff, observations and trace](failures/) remain.
The initial huge Buffer mismatch formatting was terminated after retaining its
originals; later checks use compact structured comparisons.

## Documentation, review and remaining scope

Updated the client README, maintenance ownership map, overhead art implementation
status and planetary environment sample contract. `planetarySite.ts` owns authored
site/source identity; the existing runtime owns walking/interactions and SVG draws
that same world. Report-link/source-hash checks and `git diff --check` are required
before publication. The normal production `/` still opens radar until #766;
`/?scene=top-down` is the development world and `view=interior` selects the Scout
fixture. No global generation, scientific biosphere, actual resource catalog,
mining, persistence, inventory, frame handoff (#765) or cockpit navigation (#766)
is claimed. Gathering #757 still requires #767 acceptance.

Same-session local source/result/original-image review is complete with all local
criteria passing. Current-child mandatory CI remains QA pending until the final
revision-bound ACCEPT comment. Human retains
merge authority; no merge, release, deployment or issue closure is performed.
