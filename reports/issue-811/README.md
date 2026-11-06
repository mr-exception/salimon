# #811 — Physical optical sky

Implemented the separate panoramic optical layer on `issue-811-physical-optical-sky`,
base `main` at `56481ab5d56fafd6d3bd436e4a71ccdbdc170730` (merged #810/#817;
approved design #809 is also in ancestry). Production cockpit and standalone radar
remain unchanged until final integration #815. No physics, collision, scanner,
range, resources or save rules changed.

`opticalSky.ts` derives identity-free render data from catalog radii and split
ship-relative positions. `OpticalSky.tsx` analytically renders spheres, terminators,
subpixel PSFs and a deterministic sparse starfield in one draw/two triangles.
CPU-factored surface clearance, perpendicular-ray disk tests and logarithmic hit
distance ordering avoid global float narrowing, tiny-disk cancellation and giant
depth-buffer ranges. The surface fixture consumes `startingScoutLocation()`;
Vesper's real catalog radius leaves 5 m clearance from the shared ship-bound anchor.
No fabricated terrain, atmosphere, geology, names or interaction targets are added.

## Verification and same-session review

Reviewed by the implementing Codex agent in this active chat; this is not an
independent execution. Reviewed the final source/diff, criteria, raw measurements
and original wide/telescope/ground captures. Source hashes in `source.sha256`
bind evidence to code; documentation/evidence packaging adds no runtime changes.

| Criterion | Local result / evidence |
| --- | --- |
| Pre-pulse optical visibility, no identity/selection | PASS — OPT-1 pixel assertions, empty scanner, rejected selection and identity-free render records |
| Physical centers/radii/FOV; no zoom translation | PASS — Five geometry tests; 75° disk boundary and 0.1° resolved/tiny-moon pixels; render camera remains zero |
| Shared grounded globe/horizon, downward occlusion | PASS — OPT-2 surface and below captures using shared Scout/Vesper anchor |
| Occultation, split origin and repeat stability | PASS — OPT-1 byte comparisons: pulse, hidden-moon removal, translated origin and repeated scene |
| Bounded rendering/performance | PASS (structural budget) — WebGL observations: one call/two triangles, explicit 32-body limit; no per-body tessellation, geometry churn or depth frustum expansion |
| Simulation and existing app preserved | PASS (source/unit/build) — Full unit-file suite and production build; no production entry/scene/physics source modifications |
| Remote regression/baseline gates | UNVERIFIED — CI awaiting user report; no approved baseline changed |

Commands from `client/`:

- `npm test` — 24 test files passed, 0 failed.
- `node --experimental-strip-types src/opticalSky.test.ts` — 5 focused tests passed.
- `npm run build` — passed typecheck and production Vite build; existing large-chunk warning.
- `CHROMIUM_PATH=/usr/bin/chromium npm run test:browser -- --project=optical` — **2 passed (2.5m)**; `browser.log` / `results.json`.
- `git diff --check` — passed.

Earlier fixture runs found button-focus screenshot differences, bigint JSON
serialization, 30-second multi-scenario software-rendering timeouts and a missing
favicon console error. Fixed masking/serialization, matched the existing
multi-capture timeout policy (120s), and supplied a fixture favicon. The initial
failure log and one original timeout capture remain in `initial-failure/`.
Source review also fixed float distance overflow and material disposal before the
final run. No required assertion, tolerance or baseline gate was weakened.

Original captures: [wide](wide.png), [telescope](telescope.png),
[surface horizon](surface.png), [downward occlusion](below.png). Raw states:
[wide/pulse/repeat](wide-measurements.json), [telescope/surface](surface-measurements.json).
The Pulse test control is masked consistently in canvas captures.

Local review: no remaining blocking source findings. **QA pending; CI UNVERIFIED**.

## Limits and documentation

Analytic disks are the distance/size LOD: uniform shaded surfaces when resolved,
energy-limited PSFs when unresolved. Colors/brightness are illustrative, not
canonical photometry. The explicit catalog limit is 32 (overflow throws); streaming
and larger catalogs need upstream partitioning. WebGL ran on Chromium/ANGLE
SwiftShader at 1280×800, DPR 1; no measured hardware GPU/Metal/60 FPS qualification.
Scanner overlays belong to #812, production integration to #815, broader integrated
browser qualification to #816. Broad existing browser/visual/negative/baseline CI
remains required and awaits the user's report. QA acceptance is pending that report.

Updated owner guide `client/README.md`, ownership routing
`docs/maintenance-map.md`, and `docs/current-client-status.md`. Existing approved
panoramic design and simulation/coordinate contracts are preserved. Relative report
and guide links were checked. New captures are evidence, not approved baselines.

## Environment additions

Reusable template additions: **None**. Reused Linux x86_64, Node v24.19.0, npm
11.9.0 and Chromium 154.0.8037.57. Project setup only: `cd client && npm ci`
installed 98 locked packages. No global/system/browser installation or template
change. Local git/network and Vite/Chromium required the approved expanded sandbox
context; no credentials or proxy configuration changed.
