# Browser planetary exploration acceptance (#854)

The React/TypeScript production `/` composes the published surface (#847),
geology/collision (#848), bounded regional walking (#849), overhead illustration
(#850), geological mineral deposits (#851), local observations/wayfinding (#852)
and atomic sparse expedition save (#853). The production cockpit is panoramic
(#815/#844); orthographic `/?scene=radar` is an isolated engineering fixture.
Historical Rust-native planetary systems are revision-specific references.

## Reproducible acceptance selection

From `client`: `npm ci --no-audit --no-fund`, `npm test`, `npm run build`, then
`npm run test:planetary`. Use Node >=22.18 and Playwright Chromium (or explicitly
set `CHROMIUM_PATH` to a compatible existing browser). The existing Playwright
config serves the actual built client on port 4175. Development and hazard
fixture servers are runner infrastructure; the journey does not use their state
injection APIs. Save fixtures describe admitted prior states, not live mutation
of movement/mining/landing results. Clock is fixed to 2026-11-04, DPR 1; SVG/Canvas
is CPU-rasterized, cockpit WebGL uses ANGLE SwiftShader. Inputs are Playwright
keyboard/mouse events, not physical OS/device certification. On slow software
renderers, use the existing `SALIMON_SLOW_BROWSER=1` option for the landing
journey: it batches keyboard-driver calls and extends its wall-clock deadline
without changing physics ticks, assertions or tolerances.

| Criteria | Check owner | Observable result |
| --- | --- | --- |
| AC-1/2 | `tests/planetary-journey.spec.ts` | Restore Vesper; interrupt/resume mining; inspect landmark; walk 320 m across 4+ regions, turn/reload/backtrack; one fixed Scout; deposit 1,000 g through its panel; reload in ship frame with identical marks/depletion and 60,000 g conserved. |
| AC-1/2 | `tests/deposits.spec.ts` | Actual generated common/dense/rare/barren sources, interrupted mining, worked-source eviction/reload/revisit with durable mark and exact grams; no automatic replenishment. |
| AC-1/2/3 | `tests/landing.spec.ts` | Real production Land request from launch-derived Vesper/synthetic moon approach; actual descent and correct physical touchdown, door walk, reload, relaunch/re-land with cargo and a deliberately recorded destination mark. Moon is an illustrative sample, not generated astronomy. |
| AC-3 | `tests/planetary-journey.spec.ts`, `tests/exploration.spec.ts` | Malformed/version/publication/inventory saves remain byte-identical with recovery feedback; quota failure preserves prior note and blocks replacement. |
| AC-3 | `src/planetSurface.test.ts`, `planetGeology.test.ts`, `regionStream.test.ts`, `expeditionJournal.test.ts`, `planetDeposits.test.ts` | Pole/wrap/face transforms, swept unsafe slope/footprint, stale publication/prefetch, invalid coordinates, unknown definitions, concurrent writer, failed gram write and bounded histories reject safely. |
| AC-1/3 | `tests/streaming.spec.ts`, `terrain.spec.ts`, `terrain-art.spec.ts` | Four-direction physical walking/camera and backtracking; real steep collision; canonical material/geometry after reload and eviction. |
| AC-4 | `tests/terrain-art.spec.ts`, `exploration.spec.ts`, `landing.spec.ts` | Original/repeat and intentional missing/wrong-material/mark/HUD comparisons; distinct geological states, 960/1280/1920 desktop HUD layouts plus portrait/wide checks. Gameplay references and tolerances remain unchanged; three Help text references are explicitly proposed. |
| AC-5 | Active status, phases, foundation, art directions, migration register and client guide | Delivered browser V1 versus archived native and deferred systems is explicit. |

Launch/landing maneuver safety remains owned by #845's `flight-loop.spec.ts` and
negative-navigation script. This selection reuses landing integration assertions;
it does not retune or duplicate flight physics. Full regression continues in the
existing daily/manual workflow. The new journey belongs to its world group through
the existing cockpit project; no new CI process or reviewer job is needed.

## Bounds and evidence limits

Region cache retains at most 32 regions × 2,048 samples, with at most 2,048 newly
generated samples per render patch, 1,681 illustrated vertices and a 320×320
albedo raster. Tests report actual counters over long traversal, eviction and
rapid alternating destination/publication requests. These are bounded retained
objects/work, not measured process heap bytes or real GPU/native FPS. The save
retains 128 recent physical visits and 256 deliberate marks; rendering a region
does not discover it. An exhausted mark journal rejects new marks rather than
evicting prior notes. Changed sources/partial work remain sparse durable deltas.

Save fixtures may deliberately place a player at a generated outcrop or a ship
in a moon approach. Such coverage proves the ensuing production interaction,
not a played interplanetary transfer. Landed ship pose and local cargo follow the
same shared owner in 2D and cockpit; no second ship or copied Vesper loot is valid.

Keep screenshots, traces, raw counter/save JSON, pixel diffs and failures in ignored
`client/test-results/`. Successful combined journeys retain an action trace;
other suites follow their existing trace policy. The PR records exact revisions,
commands and PASS/FAIL/NOT RUN. CI artifacts use existing uploads with 14-day
retention; local paths are not public artifact links. New geological captures are
review candidates. Three registered Help images
(960/1280/1920) intentionally propose the corrected durable-notes wording; their
originals, old references and diffs remain in ignored output, their manifest hashes
change explicitly, and appearance review remains pending with the owner. No
comparison threshold, mask or environment binding changes. User tracks CI and
reports results
in the same chat; local review does not imply CI acceptance.

Climate, weather, ecology, underground, water, procedural resource types, full
planetary chart, Core, Fabricator and multiplayer remain deferred. The authored
Vesper biological sample is not procedural ecology. Physical input, M1/Metal,
real GPU FPS, audio and subjective art signoff remain unverified separately.
