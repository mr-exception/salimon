# Issue #763 — always-visible Scout cutaway

The shared illustrated 2D world now has a recognizable Scout outline with exposed
cockpit, warm main cabin and partitioned aft cargo room. A wide open cargo passage
and aft threshold connect the rooms. Desk, separate chair, Core, Fabricator and
one 50 kg / 50 L storage placeholder have stable interaction IDs. Bunk, crate,
furniture, hull walls and jambs share physical footprints with the visible layout.
Inspections are no-ops; no machine, storage or cockpit navigation behavior is claimed.

Base: `main` at `5d5b5f816951a52d25e940bedf97adc3d387c74c`.
Source tested: `c481d3f23a36abace6459ba48643cba92965b1c0` on
`issue-763-scout-cutaway`. No active PR existed at preflight. #761/#762 are
completed and their runtime/controls are in this ancestry. Formal blocked-by
lists for #763/#764 were empty; their explicit prerequisites were checked.
Report-only changes preserve the [source hashes](evidence/source-hashes.json).
Final published head/base and current CI verdict are bound in the PR comment.

| Criterion / category | Verification and evidence |
| --- | --- |
| AC-1 exterior and interior expose the same floors/modules/player | TD-1/4 checks ordered layers, visible floor/player/rock pixels and byte-identical fresh-run captures. SCOUT-1 starts outside and verifies every module plus floor pixels. [Interior](evidence/interior.png), [exterior](evidence/exterior-cutaway.png). No roof layer or scene replacement exists. |
| AC-2 walkable rooms and addressable objects | Domain sweeps cover wall/bunk/partition blocking and open central cargo/entry clearance. Browser TD-3 walks around the cabin crate, inspects Core, chair and desk. SCOUT-1 walks to Fabricator/storage, inspects both, and reaches the entry while retaining the same ship DOM node. [Cockpit](evidence/cockpit-reached.png), [cargo](evidence/cargo-modules.png), [entry](evidence/entry-cargo.png). |
| AC-3 stable camera/cutaway | Fixed-step follow/frame tests, camera-at-rest exterior test, insertion-independent draw order, original image inspection and identical repeated browser captures. No disappearing roof, stacking artifact or player occlusion observed. |
| AC-4 automated tests/screenshots | `npm test`: 49 PASS, zero skipped; `npm run build`: PASS; focused top-down Playwright: 5 PASS, zero skipped. [Unit log](evidence/unit.log), [build log](evidence/build.log), [browser log](evidence/browser.log), [raw results](evidence/browser-results.json). Broad navigation/negative/repeat/baseline checks use the existing current-head CI. |
| Integration/E2E/visual | Real keyboard/mouse through production DOM/SVG adapters, collisions, target selection and cancellation; built radar isolation. [Radar preflight](evidence/radar-preflight.png), [backend](evidence/backend.json). New Scout captures are candidates, not approved navigation baselines. |
| Manual/hardware qualification | Same-session source and original-image review by implementing Codex `/root`; no distinct reviewer claimed. Physical input, hardware FPS, native/mobile and subjective final art approval NOT RUN, outside this cloud issue's qualification. |

## Reproduce

```sh
cd client
npm ci --cache /tmp/salimon-npm-cache --no-audit --no-fund
PLAYWRIGHT_BROWSERS_PATH=/tmp/salimon-browsers npx playwright install chromium
npm test
npm run build
PLAYWRIGHT_BROWSERS_PATH=/tmp/salimon-browsers SALIMON_EVIDENCE_DIR=test-results/763-final npm run test:browser -- --project=top-down
```

Debian 13 Linux x64, Node 24.19.0, npm 11.9.0, locked Playwright 1.64.1 /
Chromium 156.0.8078.4; 1280×800, DPR 1, en-US/UTC, dark/reduced motion,
vendored fonts/Fontconfig and fixed fake clock epoch 2026-11-04. Headless SVG CPU
rasterization is the 2D adapter; the built radar uses actual ANGLE Vulkan
SwiftShader WebGL (recorded backend). Synthetic browser input does not certify
physical input or hardware rendering. Build retains its existing large-chunk warning.

## Failures, review and boundaries

The initial wall-bound test assumed a closed aft door and diagonal movement could
now exit the open passage; it now tests an intact side wall directly. Browser
iteration caught the separate chair blocking the former center route, a too-small
chair selection range, and insufficient chair/partition clearance. The cockpit
partition moved aft by 0.4 m, the chair reach became 1 m, and the route turns before
the moved bulkhead and walks around the chair to the desk. Domain/browser routes, including the narrow crate clearance,
were updated to the actual geometry. [Failure logs and original](failures/) remain;
no collision assertion, pixel tolerance or mandatory check was removed to pass.

`scoutLayout.ts` owns the layout/anchors, the existing runtime owns movement and
interaction, and `TopDownView.tsx` draws the same world. Updated client README,
maintenance routing, overhead direction and Scout design implementation note.
`git diff --check` and new report-link/path checks pass. Mining, processing, storage
transactions, detailed habitation, persistence, automatic frame transfer (#765)
and cockpit seating/navigation (#766) remain outside this issue. The normal
production entry remains radar until #766. Exterior is a fresh development
starting viewpoint (`?scene=top-down&view=exterior`), not an in-play teleport.

Same-session review: local criteria inspected against source/tests and original
captures; remote mandatory CI remains QA pending until the final PR binding
comment records its actual result. Human retains merge authority. No merge,
deployment, release or source-issue closure is performed.
