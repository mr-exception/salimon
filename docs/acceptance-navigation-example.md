# Navigation acceptance example

This is a worked planning/review example for [the acceptance policy](acceptance-policy.md),
not a new QA certification of historical issues. It exercises the development
issue and PR templates against [#744](https://github.com/mr-exception/salimon/issues/744)
and the known [#742](https://github.com/mr-exception/salimon/issues/742) defect.

## Issue-template example: #744 ship-vector guides

Expected result: heading is yellow and velocity light green, independently
projected from the ship as forward rays clipped to the viewport. No celestial
off-screen guides or arrowheads remain. Stationary velocity is suppressed.
The current issue explicitly supersedes #738/#743's old object-guide design;
those historical screenshots cannot authorize restoring it.

Prerequisites: navigation prototype and desktop camera behavior present in the
parent; verify actual code and ancestry before using provisional PR evidence.
Production owners: `client/src/vectorGuides.ts`, `ShipVectorGuides.tsx`,
`ContinuousContacts.tsx` and `App.tsx`; cloud Chromium uses actual WebGL with
synthetic keyboard/pointer input. Fix viewport/DPR, seed and clock where the
fixture supports them and record actual values, including any uncontrolled timing.

| ID | Observable acceptance |
| --- | --- |
| NAV-1 | No off-screen celestial guides, heading/velocity arrowheads or fallback arrows. |
| NAV-2 | Yellow heading and light-green velocity reach the correct viewport edge; velocity is absent at <=1e-6 m/s; differing/opposite/overlapping vectors remain accurate and distinguishable. |
| NAV-3 | Rays follow pan/zoom/turn continuously and clip finite forward geometry with an off-screen origin, including all edges/corners and misses. |
| NAV-4 | Scanned contacts remain selectable and expire by scanner rules, independently of camera interaction. |

| Category | IDs / production coverage | Command or procedure and expected evidence | Requirement |
| --- | --- | --- | --- |
| Unit | NAV-2/3, `vectorGuides.ts` | `npm test --prefix client`; finite ray/rectangle and degenerate/zero projection assertions, exact log and source SHA. | Mandatory |
| Integration | NAV-4, scanner/runtime | Same domain suite; detection/selection/expiry contracts, not just React markup. | Mandatory |
| E2E | NAV-1–4, running App | `npm run build --prefix client`, then `npm run test:browser --prefix client`; inspect `client/tests/edges.spec.ts` and production-bundle route. Preserve JSON results, traces and backend metadata at current head. | Mandatory |
| Visual | NAV-1–3, actual canvas/SVG overlay | Inspect originals plus numerical endpoints from the edge/ray browser routes at fixed viewport/DPR; verify colors/dashes, no arrowheads/object guides, edge clipping, zero/opposite/overlap and off-screen origin. Retain original images and state, not a contact sheet alone. | Mandatory |
| Manual/hardware | Physical input, hardware FPS, mobile/native | N/A to this desktop browser issue: synthetic input/software WebGL establishes cloud behavior only; no mobile/native/performance claim is made. | Not a routine delivery gate |

The full browser suite also checks body continuity. A screenshot-baseline suite
with approved comparisons is future #751 work; existing originals and geometric
assertions do not imply that suite already exists. No images are promoted here.

## PR-template example and known-defect review

An implementation PR would use `Closes #744`, state its actual head/base/parent,
link `reports/issue-744/README.md`, fill each matrix row with actual results and
link current-head CI. For a hypothetical current review all results start
UNVERIFIED until inspected. A build pass alone leaves NAV-1–4 unverified and the
review BLOCKED, not ACCEPT. The documentation-impact section names the client
and radar contract guides as well as the report.

The historical [#742 report](../reports/issue-742/README.md) provides a concrete
counterexample to build-only or still-image acceptance: on its recorded parent,
keyboard panning had **3 missing meshes and 3 missing body-pixel observations in
65 sampled frames** (143 in-view body observations). A final still image or the
matching mouse route without misses would not prove continuity.

Use this additional defect criterion when reviewing #742 or affected navigation
changes: PAN-1 requires every in-view scanned body at its current projected
position/pixels during both keyboard and pointer pan, without pointer-down/up
flicker, selection reset or scanner reset. Run the two routes in
`client/tests/panning.spec.ts`, retain per-frame state and originals, and check
selection/scanner and ray invariants alongside visibility. Reproduce the frozen
historical evidence without rerendering using:

```sh
python3 reports/issue-742/verify_frames.py reports/issue-742/before
python3 reports/issue-742/verify_frames.py reports/issue-742/camera-idle-recheck
```

The first command must FAIL on that known defect; the second must PASS for the
retained repaired frames. These commands check archived evidence only, not the
current application. New implementation acceptance requires new compatible
production evidence and current-head CI, with an independent reviewer.

| Review input | Criterion | Verdict and action |
| --- | --- | --- |
| Historical defective keyboard frames, even with passing build/mouse route | PAN-1 FAIL | REQUEST_CHANGES: retain bodies/update projection each movement frame; add keyboard and pointer temporal regression coverage. |
| Corrected frames but old head CI or missing current originals | PAN-1 UNVERIFIED for current delivery | BLOCKED: collect/rebind current evidence and resolve CI before acceptance. |
| Current compatible production captures, all mandatory rows PASS, no blockers, independent review complete | All applicable criteria PASS | ACCEPT bound to exact head/base, cycle and reviewer identity; issue stays open until owner merge. |

The last row is a conditional example, not an approval of this or a historical PR.
The #749 report records the actual template walkthrough and frozen-evidence checks.
