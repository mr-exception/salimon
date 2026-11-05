# Issue #765 — seamless Scout doorway traversal

The player now walks from the fixed outdoor spawn through the open aft doorway,
through cargo/cabin to the cockpit and back in one continuous overhead world.
Crossing the clear entry plane transfers the collision-resolved anchor between
planet-fixed and ship-local ownership without changing its world position.
The camera preserves its smoothing lag and zoom while adopting the new frame.
Hull/jamb/partition collisions and all indoor/outdoor entities remain present.

`topDownTraversal.ts` owns crossing and detached in-memory location snapshots;
`topDownRuntime.ts` composes them after collision and during camera follow.
`capturePlayerLocation` retains frame ID, exact bigint cells/metre offsets and
facing for a future authoritative reconnect adapter. It provides no serialization,
server reconnect, spawn validation or persistence. Ship relocation is simulated
only in tests; flight and cockpit seating/navigation remain later work.

Base: `main` at `3e1e1242d7a304e9c53985633bdb55270cb5793f`.
Source: `issue-765-seamless-traversal` at
`dd5b5c2f2f8f12c1e48c83efb7523b375aa20607`.
All four prerequisites (#761–#764) are merged in this ancestry. There was no active
PR stack. Report-only commits retain the [source/input hashes](evidence/source-hashes.json).
Final published head/base, CI and review verdict are bound in the PR comment.
The six existing browser cases passed at `addb47f` before the text-only rendering
change ([log](evidence/pre-text-browser.log)); the final TRAVERSE case passes at
`dd5b5c2`. Current-head CI repeats the complete suite, including text rendering.

| Criterion / verification category | Mandatory check / expected result | Actual evidence |
| --- | --- | --- |
| AC-1 seamless outdoor/cockpit round trip; E2E | TRAVERSE-1 uses keyboard input from default outdoor spawn to cockpit chair and back; same scene node/IDs, exactly one ownership change per crossing, 0.04 m per step and under 3 px screen displacement across each handoff | PASS: [journeys](evidence/traversal-journeys.json), [browser log](evidence/browser.log), [originals/metrics](evidence/) |
| AC-2 hull and non-door walls; unit/integration/E2E negatives | TR-3 sweeps into aft jambs, both sides and nose; existing room/crate tests and browser blocked aft approach retain original frame | 56 domain tests PASS; blocked-approach browser PASS |
| AC-3 frame/camera anchors and relocation; unit/integration/visual | TR-1/2 check four rotated poses, exact-plane/tangent/missed-entry cases and camera interpolation; TR-4 checks boarding then relocation/rotation/exit, fixed surface and exact snapshot cells. Two fresh-context browser journeys compare four checkpoints with unchanged POLICY and reject the wrong checkpoint | PASS: [summary](evidence/summary.json), [domain log](evidence/unit.log), [originals/diffs](evidence/) |
| AC-4 identifiable indoor/outdoor objects; E2E/visual | Original entity IDs persist and floor/desk/Core/planetary colony occupy the shared viewport at each checkpoint | PASS: [journeys](evidence/traversal-journeys.json), [browser log](evidence/browser.log), [originals/metrics](evidence/) |
| Build/typecheck | `npm run build` | PASS; existing bundle-size advisory. [Build log](evidence/build.log) |
| Broad functional/visual CI | Current-head Navigation client domain/build/browser, navigation negative, visual repeat/negative and unchanged-baseline approval | Pending publication; final verdict recorded in PR |
| Manual/hardware | Same-session source/evidence/original-image review; physical input, device FPS/native rendering are outside this browser cloud scope | Reviewer is implementing Codex `/root`, no independent execution claimed. Physical/hardware NOT RUN; no hardware claim |

Reproduction (from `client/`):

```sh
npm ci --no-audit --no-fund
npm test
npm run build
CHROMIUM_PATH=/usr/bin/chromium SALIMON_EVIDENCE_DIR=/tmp/issue-765-browser-verified npm run test:browser -- --project=top-down -g TRAVERSE
```

[Domain log](evidence/unit.log): 56 PASS, zero skipped at source
`76ae4b87960e08fa6c8ef002ae9470bd791d465a`; subsequent changes affect only the
browser clock driver and SVG text rendering, leaving tested domain inputs unchanged. Debian 13 x86_64,
Node 24.19.0/npm 11.9.0, Playwright 1.64.1, system Chromium 154.0.8037.57;
1280×800/DPR 1, en-US/UTC dark/reduced motion, vendored font configuration.
2D rendering is headless Chromium SVG with CPU rasterization. Preflight built
radar capture used WebGL2 ANGLE Vulkan SwiftShader (software adapter), not hardware
GPU performance. The fake browser epoch is 2026-11-04T00:00:01Z; input is synthetic
Playwright keyboard through the production adapter, without state setters.

## Failures, review and limitations

Initial unit iteration caught exact-plane round-trip noise; a documented 1 nm
plane tolerance fixes it. Another unit expectation incorrectly ignored retained
camera lag on exit; it now verifies the production exponential-follow equation.
Initial test syntax/type errors were corrected. [Initial unit](failures/unit-initial.log)
and [typecheck](failures/build-initial.log) outcomes are retained.

The first browser campaign completed the traversal but failed the repeat image
comparison (8,058 pixels above unchanged policy) using approximate-duration
steering. The route now holds keys for exact observed fixed steps and captures
the full viewport using the established planetary-test approach. This produced
identical player/camera observations, but the next run failed on one storage-label
edge pixel at (681,362): RGB (145,143,121) vs (182,172,140).
[Exact-step originals, diff and results](failures/exact-step/) retain that failure.
The SVG now requests `textRendering="geometricPrecision"` in addition to shape
precision, so small labels retain subpixel geometry during camera movement. No visual
threshold, mask, reference or gameplay-coordinate rounding was changed.
The same campaign's TD-2 lost its player node while source was edited during the
run; final verification uses committed source without edits. This is a possible
HMR interference explanation, not a claimed proven cause. [Initial log/results](failures/)
and [failed original pair](failures/traversal-initial/) are retained. A subsequent
exact-step campaign was intentionally interrupted after six passing cases because
1 ms clock polling made the traversal driver excessively slow. The final driver
advances in 16 ms RAF intervals and still asserts exact tick counts;
[interrupted log](failures/browser-interrupted.log).

Client usage/ownership guides, maintenance routing and cockpit/reconnect
implementation status now describe this development-world behavior. No approved
navigation reference changes are included. Original captures and repeat comparisons
are evidence candidates, not promoted visual baselines. Same-session implementing reviewer `/root` inspected the unique source diff,
unit/typecheck results, raw transition samples, original door/cockpit/return captures
and diff images. AC-1–AC-4 local checks PASS: largest measured world-step error
9.23e-16 m; maximum observed player screen displacement 0.733 px; all four repeat
comparisons have zero pixels above policy. Raw subthreshold differences are retained.
The wrong-checkpoint negative rejects 86,148 pixels. Local review has no remaining
blocking findings; verdict remains QA pending until current-head CI resolves.
Final same-session review binds exact head/base and all check results in the PR.
Repository ruleset listing returned no rulesets; the branch-protection endpoint
returned HTTP 403, so additional hidden protection requirements could not be
inspected. The existing workflow and actual published checks are followed.
Human retains merge authority; issue stays open before merge.
