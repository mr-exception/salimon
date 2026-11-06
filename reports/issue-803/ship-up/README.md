# Universal ship-up 2D camera — PR #807 follow-up

The active ship now stays vertical, bow pointing to the top of the 2D screen,
indoors and outdoors, for every physical heading. `worldToView` projects relative
world positions into the ship-aligned view; entity facings and selection cues
subtract the same ship heading. `viewToWorld` maps screen-relative W/A/S/D back
to the unchanged physical movement/collision frame. The camera basis remains
stable across doorway ownership changes. Authored site pose, saved anchors,
collision/reach geometry and 3D radar orientation controls are unchanged.

- User-requested addition to [PR #807](https://github.com/mr-exception/salimon/pull/807), closing #803.
- Production source: `963e359ec00f2f6b49629bfd978fc95897f5b2d5`; final assertion revision `da89cb9fcd81312c0335291b5533072d3b0110f4`, parent delivery `b8e0fb2c343bd1c7cbecb6fe9b7e70b9b711cada`.
- PR base remains `main` / `4b27aa62366c3ca57a97f7a2965f6d1882d5de1e`; final report-only commit does not change executable inputs.
- Current validation supersedes the initial report's source-specific results.
- Status: **QA pending — required CI awaiting user report**. No merge or baseline approval.

## Verification and same-session review

| Criterion / category | Result and evidence |
| --- | --- |
| Universal orientation, domain/unit/integration | PASS: `npm test`, 22 test files, including arbitrary headings (0, −0.12, π/2, −π, 2.8, 7), indoor/outdoor positions, all inverse input directions, unchanged anchors and correct rendered player facing; [log](unit.log) |
| Build/typecheck | PASS: `npm run build`; [log](build.log); existing bundle-size advisory only |
| Current browser / screenshot matrix | PASS: all 22 distinct affected scenarios across initial run (18 pass) and corrected focused run (5 pass, including one unchanged storage check); [initial raw results](browser-initial-results.json), [final raw results](browser-final-results.json), [final log](browser-final.log) |
| Documentation | PASS: universal rule in art direction, playable foundation and client guide; browser steering helpers now interpret waypoint directions in the new screen basis |
| Physical-device/manual performance | N/A to this deterministic camera rule; no physical-input, mobile or hardware GPU claim |

Browser command (from `client/`):
`CHROMIUM_PATH=/usr/bin/chromium SALIMON_EVIDENCE_DIR=test-results/803-ship-up npx playwright test --project=cockpit --project=top-down`.

Final focused command: same environment variables, `SALIMON_EVIDENCE_DIR=test-results/803-ship-up-verified npx playwright test --project=cockpit --project=top-down --grep 'VIEWPORT|TRAVERSE-1'` (5/5 PASS; grep also selects the existing storage test whose title includes viewport). Final `npx tsc -b` also passes after assertion edits.

[Source hashes](source-hashes.json) bind the final inputs. The 18 passing scenarios
use the same production code, domain tests, dependencies and their unchanged
browser assertions; only the four failing tests' measurements changed afterward.

Fresh originals: [1280×720](1280x720/fresh-0.png) / [repeat](1280x720/fresh-1.png),
[960×600](960x600/fresh-0.png) / [repeat](960x600/fresh-1.png). Fresh and paused
repeats at both sizes have **0 policy-changed and 0 raw-different pixels**.
Intentional inset negatives still fail: 128,472 changed pixels at 1280×720 and
89,785 at 960×600. [Inside doorway](door-inside-0.png),
[outside doorway](door-outside-0.png), [standing after cockpit](standing-return.png),
[storage](storage.png) and [terminal](terminal.png) retain representative originals.
Across both doorway replays the maximum one-tick projected anchor displacement
is **0.9318400701 px**, below the unchanged 3 px continuity bound. Exact movement,
single ownership transitions, scene identity, repeat captures and negative
wrong-checkpoint comparison all pass. Raw measurements/comparisons are embedded
as base64 JSON attachments in the two canonical Playwright result files.

Initial failures: three new bow assertions distinguished IEEE signed `-0` from
`0`; now `abs(dx)` must equal exactly zero. The dev traversal test measured the
rotating sprite box corner (9.3714 px when turning), not the world anchor. It now
measures the origin of the player's actual screen CTM, retains old/new sprite
bounds as raw context, and preserves the 3 px threshold. Production code did not
change. [Initial log](browser-initial.log) and [original traversal capture](initial-traversal-assertion.png)
preserve these failures. A diagnostic rerun started before the edit applied due
to a corrected working-directory mistake and repeated the same failures;
[its log](browser-stale-assertions.log) is retained. The final run follows the
completed edits/typecheck and passes. No screenshot threshold was changed.

The viewport tests explicitly require zero rotation for hull, floor, desk and
entry, and a screen-space bow vector with zero horizontal component and upward
vertical component. Existing viewport/HUD/DPR/resize, focus and repeat/negative
checks remain enabled. Approved navigation baselines and comparator policy are
unchanged. Software SVG and cockpit ANGLE/SwiftShader; synthetic keyboard/mouse,
controlled UTC clock from 2026-11-04, system Chromium 154.0.8037.57, DPR 1/2.

Same-session review by implementing Codex agent: inspected the final diff,
unit/domain results, original fresh/smaller-window/standing captures, actual
repeat/negative results and doorway anchor measurements. Confirmed the universal
orientation/input mapping, world alignment, frame continuity and preserved HUD
contract; no remaining local blocking finding. No independent execution claim.
Final ACCEPT remains pending user-reported current-head CI. All broad functional,
negative/repeat visual and baseline-approval CI gates remain intact and unqueried.

**Environment additions: None.** Existing project packages, Node 24.19.0,
npm 11.9.0 and system Chromium reused; no installations or template edits.
