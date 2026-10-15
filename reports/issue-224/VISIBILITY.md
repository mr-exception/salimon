# Supplemental visibility/render-order evidence — #224

2026-10-15; inspected revision `406f78d27242380866f8438bf70334110f950905`.
This supplements the [original baseline](README.md) with ray occlusion checks and a distinct transparent/opaque scheduling finding. This is diagnosis before art changes. No production mesh, material, collider,
scale, shader or runtime behavior changed. Option A (cohesive low-poly) remains
the initial direction; Option B hard-surface presentation must preserve the same
spatial contracts. No external model-generation service was used.

## Evidence and reproduction

- [visibility-baseline.json](visibility-baseline.json): measured vertex bounds, budgets, normal checks,
  source/export hashes, material definitions, coplanar face witnesses and camera
  ray intersections. Face numbers are **zero-based triangles within each GLB
  primitive**, not Blender polygon indices. Node names identify the owning meshes.
- [visibility-views.json](visibility-views.json): eight portable camera fixtures, projection, explicit
  ship/world transform and a static/moving-camera capture protocol for #228.
  These are diagnostic definitions, **not executable salimon-test scenarios**.
- [model-tests.log](model-tests.log): complete portable model test results.

Run from repository root with Python 3.10+ (probe itself uses standard library):

```sh
python models/tools/diagnose_scout_visibility.py --views reports/issue-224/visibility-views.json --output artifacts/issue-224/visibility-baseline.json
python -m pip install -r models/tools/requirements.txt
python -m unittest discover -s models/tests -v
python models/assets/ships/salimon-scout/validate.py
```

Compare newly generated JSON with this dated baseline. For an individual witness,
look up its two `nodes` in `coplanarCandidates`; inspect `witness.faces`,
`positionMeters` and the two `views` (eye, target, up, nearest opaque hits).
Coordinates are ship-local runtime meters: +X forward, +Y up, +Z port. Set the
ship to identity at world origin for inspection; for gameplay, compose the given
pose with the current ship's f64 origin/orientation. The fixtures specify this
mapping and a second origin at 1 AU to exercise precision. Closed door is the
probe's only geometric pose; the ramp-open fixture is for a future actual runtime
capture, not an animated ray result.

GPU screenshots and temporal flicker were **not reproduced in this environment**:
there is no Blender, Cargo/Rust toolchain, Xvfb or installed graphical capture
stack on PATH. The evidence below proves structural/render-path problems, not
observed frame-to-frame pixel changes or M1 fidelity. #228 owns portable GPU
capture/diff execution; native Apple M1 signoff remains #229. This initial
cloud-capable issue does not depend on those hardware results.

## Confirmed defects and controls

| ID / classification | Objects and location (meters) | Evidence and reproduction | Priority / regression acceptance |
| --- | --- | --- | --- |
| G1: geometry, exposed coplanar opaque surfaces | `Door_Threshold` primitive 0 face 6 and `Deck_Walkable` primitive 0 face 4; witness approximately (-7.34, 0.24731183, -0.55), plane +Y | `threshold-coplanar` ray first hits both at 1 m. Closed-door asset has positive-area overlap, different materials (Hull Graphite / Floor Grip). The ship uses strict Greater depth and zero bias: an equal-depth tie depends on submission/raster arithmetic; moving the camera can reveal different winning surfaces. Actual temporal flicker remains unmeasured. | P1: remove hidden duplicate/layered top faces or give an intentional authored separation; preserve flush deck/ramp walking plane. New validator detects injected overlap; repeat static and 2 cm moving camera with no unstable pixels outside reviewed tolerance. |
| G2: geometry, exposed console side overlap | `Cockpit_Monitor_Bezels` primitive 0 face 0 / `Cockpit_Monitor_Housings` primitive 0 face 0; (4.44188247, 0.85766882, -1.04999995), plane +Z | `console-edge` ray from starboard hits both at 1 m. Interior Light / Hull Graphite have different colors. This is housing geometry, not a flipped monitor or UV defect. | P1: trim shared faces or deliberately separate visible layers; retain exact monitor corners, normals, UVs, console collider and clear walking lanes. Same static/moving-camera case. |
| R1: renderer, glass vs later opaque draws | `Cockpit_Glazing` at forward X=8.2; fixture eye (6.6,1.8,0), resource center (9.5,1.8,0), radius 0.25 | Center ray hits glass at 1.6 m and no ship opaque geometry. `lib.rs` submits ship opaque **and glass** before resources and held item. Glass has no depth writes; a later resource behind the window passes depth and replaces the previously blended color. This is a deterministic tint/layering defect, independent of z-fighting. | P1: schedule all relevant opaque world draws before transparent ship draw. Compare a resource behind this glass with an equivalent early opaque cuboid; both must receive consistent tint, including after camera motion. Keep two ship draws. |
| R2: renderer/material mismatch | `Deck_Walkable`, `Floor Grip`; eye (0.5,1.99731183,-3), looking down | GLB contains `baseColorTexture` with embedded 16×16 PNG and UVs. Loader uploads base color/emissive only; `ship_mesh.wgsl` samples only live instrument atlas. The floor texture cannot appear in this runtime path. Ray first hits deck at 1.75 m (fixture avoids the woven runners). Metallic/roughness values likewise never reach the shader. | P2: explicitly choose Option A's supported shader/material contract; either support intended channels or document their deliberate nonuse. Do not spend texture/detail budget on invisible channels. Verify actual pixels, not only export metadata. |
| R3: renderer, fixed transparent ordering risk | `Cockpit_Glazing`, `Cabin_Starboard_Glazing`; `glass-layering` eye (12,2,6), target (0,1.4,-6) | Center ray crosses these glass sheets at 6.016660 m and 15.339654 m. Glass vertices remain in GLB node/primitive order, never camera sorted. Alpha composition is order dependent when shaded layer colors differ; ship lighting varies by normal. This confirms the unsupported ordering, **not** that this specific ray flickers or currently has reversed draw order. | P2: inspect both opposite viewpoints before choosing camera sorting or a constrained glass construction. Shared identical-color layers alone are not a visible sorting failure. Capture before enabling a policy. |

The full scan finds **82 node-pair groups / 933 triangle-pair overlap candidates**
within 10 µm plane tolerance and above 1e-8 m² projected area. Counts include
self-overlap, buried box faces and intentional intersections; they are **not 933
visible defects**. Areas sum projected triangle-pair intersections and can double
count coverage. `Ceiling_Inner` / lamp housings, guidance lights / walls and
landing-leg contacts are candidates to inspect, not automatic failures.

`starboard-shell` is a useful negative control: its center ray hits the outer
shell at 0.5 m before the bulkhead/shell overlap at 1 m. `lower-glazing` similarly
hits a frame surface before the coplanar glass edge; opaque/glass seams are not
equivalent to exposed opaque/opaque ties. Candidate witness rays are numerical
probes, not pixel-area visibility proofs or reachable player positions. Tangent
rays at solid boundaries can differ with tiny camera changes.

## Findings that do not establish a defect

- **Scale/proportions:** measured bounds are min (-9.80000019, -2.10752678, -10)
  to max (8.52999973, 3.89247322, 10), giving **18.32999992 × 6 × 20 m**.
  This matches the declared 18.33 × 6 × 20 m contract. No global scaling/anchor
  failure was reproduced. Subjective proportions require a reference/approval;
  the earlier much smaller scout is historical, not the current target.
- **Normals/winding:** zero degenerate triangles, nonunit corner normals
  (1e-4 tolerance), or corners opposed to their triangle winding (< -1e-5 dot).
  This does not prove a watertight/manifold mesh, correct smooth shading, or
  consistent outward orientation of every closed shell. Faceting is permitted.
- **Depth/precision:** runtime uses Depth32Float, reverse-Z cleared to 0,
  Greater testing, infinite projection, physical near plane 0.05 m. Ship center
  is camera-subtracted in f64 before f32 conversion; authored vertices remain
  local. There is no evidence here of absolute universe coordinates entering
  the ship vertex shader. Reverse-Z improves precision but cannot separate
  coplanar surfaces. Do not change near plane/world scale as a speculative fix.
- **Transforms:** baked visual nodes have identity rotation/scale and identity
  ancestors. The loader uses node-local transforms and ordinary direction
  transforms for normals: acceptable for this asset, not a general glTF scene
  graph/nonuniform-scale importer. Future optimization must not introduce
  transformed parents without updating this consumer.
- **Lighting:** the shader uses fixed normal-based exterior light, warm
  Interior-ancestor fill, absolute normal dot products and additive emission.
  It does not implement PBR, shadows or back-face-specific lighting; pipeline
  default culling is disabled. A Blender/render mismatch here is not proof of
  bad normals. Emissive clipping and shimmering highlights were not observed.

## Baseline budgets and migration invariants

| Measurement | Current | Manifest cap |
| --- | ---: | ---: |
| Triangles | 6,102 | 6,500 |
| Mesh primitives | 116 | 124 |
| Materials | 13 | 13 |
| GLB bytes | 513,096 | 557,056 |
| Embedded compressed texture bytes | 116 | 65,536 |
| Ship runtime draws | 2 (opaque + glass, source inspection) | 2 |

The packed floor PNG is 16×16, approximately 1,024 bytes as RGBA8 base mip.
The manifest's texture-byte metric measures compressed embedding, not GPU memory.
The current ship renderer does not upload this floor image. Its separate live
instrument atlas is 2048×256 RGBA8 (~2 MiB base mip), independent of that budget.

GLB SHA-256 is
`3ec25f1ee25fb9ee73f488d7b9daa1df909158a5ccd4cfa1560d77f3f619646c`.
All 11 assembly/shared/component source hashes and runtime hash match the
checked-in export report. This establishes provenance consistency, **not** a
fresh Blender source-to-export comparison. The exporter stages/validates output
before publishing; nine linked component libraries plus shared hierarchy/materials
remain authoritative. Blender semantic checks were not executed here.

Preserve the following through Option A and later Option B:

- Runtime +Y up, +X forward, -Z starboard, meters and ship-local reference pivot;
  Blender metric +Z up mapping `(x,z,-y)` once. No runtime compensating scale.
- Frozen hierarchy/material roles and stable names; Interior ancestry controls
  warm fill. Cockpit Glass remains the sole BLEND, double-sided material.
- The **28 authored box colliders/four markers**, their geometry/metadata and
  generated character spatial layouts; these are intentional gameplay proxies,
  not a mandate to make all visual meshes solid or manifold.
- Four landing feet centered at X=-5.6/+4.8 and Z=±3.6 m; foot plane
  Y=-2.10752678 m; lowest non-gear mesh is 2 m higher. Deck Y=0.24731183 m is
  2.35483861 m above that plane.
- Bottom hinge (-7.68,0.24731182796,0), 4.4 m deployed ramp, 2.13118279596 m
  tray travel, authored angle 2.13552971042 rad and 0.08 m face offset. Preserve
  collision/sweep interlocks and generated ramp geometry.
- Seat/exit/start/engine anchors in [spatial-contracts.json](../../client/assets/ship/spatial-contracts.json),
  nose/cabin routes and lower glazing sightlines. Wide monitor is 1.98×0.3332 m;
  Core monitor is 1.30×0.65 m. Preserve both two-triangle surfaces, exact
  authored corners, pilot-facing normals, top-left UVs and live atlas regions.

Canonical contracts: [scout source](../../models/assets/ships/salimon-scout/README.md),
[manifest](../../models/assets/ships/salimon-scout/manifest.json),
[renderer](../../client/renderer/invariants.md). The stale renderer invariant mentioning a "4.50 m open door offset" was corrected
to reference the current generated ramp constants; animation behavior is unchanged.

## Follow-up order and acceptance guidance

1. **#225** can start from this baseline: preserve geometry interfaces and linked
   ownership while defining low-poly recipes/palette. Use one small asset slice;
   ship restyling must not hide G1/G2. It is optional for validators.
2. **#226** should turn G1/G2 and synthetic bad/good triangles into actionable
   geometry checks. Separate visible ties from intentional buried intersections;
   support per-category policy/exemptions. This diagnostic's rounded-normal
   buckets can miss bucket-boundary/skewed near-parallel pairs, its per-bucket
   scan can be quadratic, and it is not a general collision/topology validator.
   Do not promote candidate counts into a CI failure threshold.
3. **#227** depends on #226 and must recheck dimensions, names, parent transforms,
   monitor UVs, material roles and source hashes before/after optimization.
   Material/primitive reduction alone does not reduce the existing two ship draws.
4. **#228** consumes the eight fixtures after #226/#227. Add actual renderer
   capture wiring, open-door pose, resource presentation, view reversal and
   static/moving sequences; freeze animation/displays. Exercise G1/G2, the R1
   early/late opaque comparison, glass seams and 1 AU origin translation.
   These JSON fixtures are not approved pixel baselines. Record which cases
   remain unobservable rather than accepting a synthetic render as game evidence.
5. **#229** remains native-only. Verify Metal/M1 temporal behavior/performance
   locally after #227/#228; do not require it for closure of #224.

R1 and R2 are runtime renderer work, outside the geometry/export validator's
ownership. They need explicit fixes when following issues address render quality;
#228 can expose them but an image harness alone will not repair them. This report
recommends them without making unrequested production changes.

## Validation

Environment: Linux x86_64, Python 3.12.14, jsonschema 4.26.0.

- **PASS:** read-only baseline probe, all 11 source hashes/runtime hash match;
  measured dimensions/budgets above. Repeat-output byte equality checked.
- **PASS:** `python -m unittest discover -s models/tests -v`: 63 tests,
  2 skips (Blender-dependent export/modular propagation). Four new geometric
  tests cover edge-only contact, disjoint/contained triangles, reversed winding,
  separated planes, double-sided rays and hidden vs exposed witnesses.
- **PASS:** `python models/assets/ships/salimon-scout/validate.py` (same budgets).
- **PASS:** `git diff --check`; report/guide relative links and JSON fixtures checked.
- **NOT RUN:** Blender `verify_source.py` / `verify_modular.py`, real export,
  Rust build/tests/Clippy, native/GPU screenshots or temporal captures. Tools
  unavailable; production Rust/assets unchanged. No M1 or FPS claim.
