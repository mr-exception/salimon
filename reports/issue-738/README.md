# #738 — viewport-reaching directional contact lines

Off-screen scanned contacts now use subdued ship-to-boundary lines instead of
isolated edge glyphs. Projection uses the real orthographic camera and actual
viewport, with linear ship-relative split coordinates. Labels are inset; line
endpoints remain on the exact viewport boundary with no arrow or interior cap.

Finite segment clipping handles every edge/corner, parallel/coincident/tangent
inputs, ship-off-screen entry/exit and paths that miss the viewport. Depth-only
exclusions retain a label without inventing a 2D direction. Right-drag pans the
independent camera; reset also restores its target. Existing orbit/zoom controls,
ship motion, five-second scanner cadence and contact expiration retain authority.

Stable perimeter slots show one subdued ray per group. Selected contacts supply
the exact highlighted ray and pressed label; otherwise a visible crossing ray, then nearest distance/ID
chooses a deterministic representative. Every ID remains accessible through the
existing keyboard-compatible paged contact menu/search. Visible grouping now also
renders the selected representative at that body's own position.

Parent: PR #739, branch `issue-737-faceted-navigation`, at
`bc6ccdc44adae64688243b065bf49e17202bfb51`. This child targets that branch.
Revalidated unchanged parent/default refs before publication; prototype #731 and
scanner/input prerequisites #728/#730 are merged. Parent hull and camera-reset corrections are included in this ancestry. Parent functional-source CI passed (before final whitespace normalization): [run 41721378488](https://github.com/mr-exception/salimon/actions/runs/41721378488). Child current-head CI links and outcomes are recorded on its PR and source issue.
[Source and fixture hashes](source.sha256). Merge #739 before this child.

## Prepublication corrections

- Screenshot review found title/legend overlap in the first label layout.
  [Original all-edge capture](initial-layout/all-edges.png) and
  [selected capture](initial-layout/selected-edge.png) are retained. Labels now
  reserve banner space; browser assertions check banner and pairwise label bounds.
- The first spacing revision split near-collinear contacts at the midpoint slot
  boundary: 15/16 browser routes passed. [Failed log](initial-grouping/browser.log),
  [results](initial-grouping/results.json), [original](initial-grouping/overlap-failed.png).
  Odd slot counts give cardinal rays stable central slots; a new regression checks
  both axes. No count assertion or scanner requirement was relaxed.
- With wider top groups, the nearest representative could miss the panned view
  and hide a crossing ray in the same group: 15/16 routes passed.
  [Failed log](initial-representative/browser.log),
  [results](initial-representative/results.json),
  [original](initial-representative/ship-off-screen-failed.png).
  Unselected groups now prioritize a visible crossing before distance/ID;
  selection still uses the exact selected path, including honest misses.
- Parent #739 received an outward hull-face correction and regression; the child
  was fast-forwarded onto that fix before its final checks. A subsequent parent CI failure exposed residual camera damping on reset; its correction and the exact projected-position regression are also inherited. See the parent report for retained CI evidence.

## Validation

Linux 6.18.57 x86_64, Node 24.19.0/npm 11.9.0, Chromium 154.0.8037.57,
actual ANGLE Vulkan SwiftShader WebGL 2. Same locked dependencies as #739.
Desktop viewport 1280×800; existing mobile regression remains in the full suite.
Browser mouse/keyboard events exercise production OrbitControls and accessible
labels. Deterministic catalog/camera/ship fixtures are development-only protocol
inputs into the production App/runtime; physical input is not claimed.

- `npm test --prefix client`: 29/29 pass. New clipping tests cover all edges,
  corners, misses, parallel/coincident/nonfinite inputs, 200 arbitrary projections,
  collinearity/direction, exact viewport classification, pan/rotation, depth-only
  clipping, invalid size and 2,000-contact deterministic selected grouping.
  Existing scanner precision/range/cadence/expiry and physics/input tests pass.
  [Raw unit output](domain-tests.log).
- `npm run build --prefix client`: pass. [Raw output](build.log).
  The existing Vite >500 kB bundle advisory is retained.
- `CHROMIUM_PATH=/usr/bin/chromium npm run test:browser --prefix client`:
  16/16 pass, [raw output](browser.log), [results](browser-results.json).
  Three new production-App browser routes check eight exact edge/corner rays,
  selected keyboard/click actions, actual mouse pan/orbit/wheel with unchanged
  ship stats, off-screen ship crossings and same-side misses, injected ship
  motion with independent projected-line collinearity assertions, no early scan
  reveal, out-of-range suppression, selected-contact expiry and 2,000-contact
  overlap/paged selection. Existing visual-pixel/resource, physics, input,
  mobile and built-production checks all pass. Final local checks pass; prepublication findings and failed attempts are retained below.
- `git diff --check`: pass; reviewed unique child diff and unchanged parent.

Original captures: [all edges/corners](all-edges.png),
[selected ray](selected-edge.png), [pan/orbit/zoom](camera-moved.png),
[ship off-screen](ship-off-screen.png), [2,000-contact overlap](overlap.png).
Raw [projected endpoint/camera/ship/backend observations](edge-state.json),
[overlap ray](overlap-state.json), [built production backend](backend.json).
No screenshots or tolerances were automatically promoted to approved baselines.

CI repeats the full locked build/unit/browser checks on the published head; its
run and result are recorded on the PR/source issue. Revision-named CI artifacts
retain full screenshots/results/traces for 14 days; useful originals above are
permanent. Cloud software rendering does not establish hardware FPS or physical
input/device qualification. This changes navigation presentation only; sample
catalog and existing simulation/migration limitations remain.

Final evidence formatting removes trailing whitespace from retained text logs. Parent formatting-only corrections were merged without rewriting either branch. Executable inputs are unchanged; local verification above is reused and CI repeats the complete gate on both final published heads.
