# #224 — ship geometry/render diagnosis

Baseline: `406f78d27242380866f8438bf70334110f950905` (2026-10-15).
This investigation preserves all source meshes, exports, runtime code and budgets.
It confirms overlapping exported surfaces and a material-consumption mismatch;
**temporal flicker and subjective incorrect proportions were not reproduced in a
live renderer here**. Do not treat the candidate count as a visible-defect count.

## Reproduce the portable evidence

From the repository root:

```sh
python models/tools/diagnose_scout.py > artifacts/scout-diagnosis.json
python -m unittest discover -s models/tests -v
python models/assets/ships/salimon-scout/validate.py
```

Create `artifacts/` first if absent. [geometry-baseline.json](geometry-baseline.json)
is the captured diagnostic output: hashes, every mesh's actual bounds, triangle
indices/vertices/materials for representative overlaps, normals and budgets.
The probe is read-only and standard-library-only. It decodes the actual GLB
accessors (offsets/strides), rejects non-baked mesh/parent transforms, checks
normal length/winding and clips projected triangles to exclude shared edges and
AABB-only overlaps. Plane buckets round to 10 µm/1e-5 normal components with a
10 µm point-to-plane verification and 1e-8 m² area cutoff. This is a small scout
investigation, **not** #226's accelerated general validator. Bucketing may miss
near-plane pairs on bucket boundaries; no non-manifold or full self-intersection
claim is made. Pair areas are summed, not a union area.

## Measured baseline and provenance

| Quantity | Measured | Existing limit/contract |
| --- | --- | --- |
| Triangles | 6,102 | 6,500 |
| Primitives | 116 | 124 |
| Materials | 13 | 13 |
| GLB | 513,096 bytes | 557,056 bytes (544 KiB) |
| Embedded image bytes | 116 | 65,536 (encoded bytes, not decoded GPU memory) |
| Envelope (forward/up/port) | 18.329999924 × 6 × 20 m | 18.33 × 6 × 20 m |
| Bounds | (-9.800000191, -2.107526779, -10) to (8.529999733, 3.892473221, 10) m | Baked runtime coordinates |
| Draw policy | 1 opaque + 1 glass | No new draw/mesh scale |

Runtime SHA-256: `3ec25f1ee25fb9ee73f488d7b9daa1df909158a5ccd4cfa1560d77f3f619646c`.
All 11 assembly/shared/component `.blend` SHA-256 values match the export report.
These hashes establish provenance consistency, **not a fresh source evaluation**.
Blender is unavailable, so reopening linked libraries and comparing evaluated
source vertices via `verify_source.py`/`verify_modular.py` was not run. Reviewed
those scripts, shared `blender_export.py`, scout `export.py`, manifest,
preservation/spatial metadata, runtime GLB and renderer source directly.

The nine component libraries link into `assembly/salimon-scout.blend` with the
shared hierarchy/material library. Offline export selects the authored hierarchy,
requires metric scale 1, applies modifiers, exports Y-up once, preserves extras,
then scout validation checks metadata/spatial contracts before publication.
Current visual nodes and their hierarchy groups have identity transforms. The
runtime importer applies only each node's local matrix; future transformed
parents would need cumulative transforms or explicit rejection. No present
parent-transform defect was found.

## Findings and classification

| Finding | Evidence and reproduction | Classification / confidence |
| --- | --- | --- |
| Guidance light and wall share a visible-side plane | `Cabin_Low_Guidance_Lights` primitive 0 triangle 20 and `Wall_Port_Inner` primitive 0 triangle 8 both lie at Z=4.460000038 m; 4 overlapping triangle pairs, summed area 0.451613 m². Warm Lamp vs Interior Light. Run probe; inspect port-guidance-light pose. Starboard has equivalent evidence. | Confirmed geometry overlap; strong z-fighting candidate. Whether the light is visibly suppressed or flickers requires capture. Owner: cabin-interior. |
| Threshold/deck overlap | `Deck_Walkable` p0/t4 and `Door_Threshold` p0/t6 at Y=0.247311831 m; 2 pairs / 1.232000 m². Floor Grip vs Hull Graphite. Run probe; aft-threshold pose with closed/open ramp. | Confirmed geometry overlap at walking plane; candidate depth conflict. Owners: cabin-interior / exit-door. |
| Ceiling/housing and other construction overlaps | `Ceiling_Inner` p0/t6 and `Ceiling_Lamp_Housings` p0/t4 at Y=2.806451559 m; 12 pairs / 9.196800 m². Overall 82 node-pair groups / 927 triangle pairs, including within-node joins. JSON lists all. | Confirmed geometric coincidence; some are concealed touching caps or identical colors and must be exempted or inspected, not blindly deleted. Owner depends on pair. |
| Floor/ship material appearance differs from Blender | `Floor Grip` has `baseColorTexture.index=0`; the GLB embeds the 116-byte image. `ship_mesh.rs::load_geometry` reads color/emission and only display UVs are consumed by `ship_mesh.wgsl`. No floor texture binding/sample exists. Metallic and roughness factors on the 13 materials are not passed to this ship shader. | Confirmed renderer material-support gap, not z-fighting. Verify Floor Grip in Blender against runtime deck; decide deliberate stylized simplification vs support. |
| Glass ordering | Only Cockpit Glass is BLEND (alpha ≈0.10), double-sided. Import appends glass in node/primitive order, immutable across camera moves; one draw after opaque geometry, Greater depth test, no glass depth writes and no sorting. Cabin and sloped cockpit panes have different normal lighting. | Renderer order-dependence risk where differently shaded panes overlap; no captured alpha-sorting defect. Identically shaded panes can commute, so fixed order alone is not proof of visible flicker. |
| Normals / winding | Zero degenerate triangles at 1e-10 cross-product magnitude; zero non-unit/nonfinite/opposing vertex-normal findings at 1e-4 tolerance. Shader uses `abs(dot(normal,key))` and no cull mode, which can hide reversed faces visually. | No sampled normal/winding defect; does not prove watertight/manifold topology. Do not introduce normal fixes without a failing witness. |
| Scale / anchors | Envelope matches manifest. Monitor_Wide plane X=4.420000076, width 1.980000019, height 0.333199978 m; housing starts X=4.429999828 (~10 mm behind). Deck top Y=0.247311831; lowest feet Y=-2.107526779. Core monitor bounds and all mesh bounds in JSON. | No objective rescale or monitor-plane overlap defect found. Subjective proportions unresolved; no second runtime scale or unit-conversion error identified. |
| Depth / precision | Depth32Float, infinite reverse-Z, clear=0, Greater comparison; opaque writes / glass read-only. Ship center is camera-rebased in f64 before f32; small local vertices remain meters. Exact coincident faces remain ambiguous even with reverse-Z. | No demonstrated universe-coordinate precision bug. Changing near plane/depth bias would not repair duplicate surface ownership. |
| Lighting / aliasing | Vertex shader uses fixed world key with absolute dot, warm ship-local cabin fill, and additive emission; no PBR BRDF/shadows in this path. Default single-sample pipelines, no temporal noise/jitter. | Deliberate simple shading explains DCC differences; subpixel edges/emissive detail can shimmer in motion. No frame sequence captured to quantify it. |

## Camera and visual reproduction contract

[camera-fixtures.json](camera-fixtures.json) gives exact eyes/targets, up, FOV,
near plane, viewport, door states and static/moving capture steps for eight views.
It is a portable fixture specification for #228, **not a new runnable scenario
schema or a claim that these frames were captured**. Coordinates use runtime
axes (+X forward, +Y up, -Z starboard) on an isolated ship at world zero with
identity rotation, so the given coordinates are also explicit world coordinates.
In a real planet scene use `worldEye = shipPosition + shipRotation * localEye`
(and the same mapping for target/up); preserve the resulting inspected state.
Blender comparison maps runtime (x,y,z) to source (x,-z,y).

1. Run the probe to locate exact mesh triangles; open the owning linked component
   in Blender, isolate the pair and inspect both face planes/materials. Inspect
   evaluated assembly with read-only `verify_source.py` when Blender 4.5 is available.
2. #228 should render these fixtures in Salimon's actual shader, capture static
   sequences first, then the specified ±2 cm sweep. Do not compare moving frames
   as if pixels should remain identical. Inspect concealed overlaps separately.
3. Existing runnable routes are `scenarios/evidence/cockpit-wide-monitor.json`,
   `lower-cockpit-windows.json` in the same evidence folder,
   `scenarios/cockpit-nose-starboard.json`, and `scenarios/space-airlock.json`.
   Run with `python scripts/salimon-test run <scenario> --binary <staged binary>`;
   evidence routes additionally need the platform screenshot command documented
   in `scripts/README.md`. These establish navigation/state, not flicker absence.
4. Check both cabin sides, closed/open ramp, floor at shallow angles, ceiling,
   console/Core faces, and exterior through multiple glass layers. Preserve exact
   camera/ship state and consecutive frames with any observed artifact.

## Prioritized follow-ups and acceptance witnesses

1. **#226:** turn selected harmful overlaps into actionable validation policy with
   per-pair exemptions for hidden construction caps/layered glass. Use the JSON
   triangle witnesses and the new synthetic clipping tests. Mutation tests must
   catch real positive-area overlap while accepting shared edges/disjoint faces.
   Preserve real normal, dimensions and anchor checks; baseline is not a waiver.
2. **Geometry remediation after classification:** guidance strip faces need one
   visible owner or a deliberate offset; threshold must have one noncompeting
   top surface while retaining the exact walk plane and ramp pivot. Re-export
   owning components and require coplanar witnesses removed/exempted, spatial
   validation passing and static/sweep frames clear. Do not restyle to hide bugs.
3. **#225:** use these bounds/owners for inspect/edit/validate workflow and Option A
   style rules. Keep silhouette/facets, mostly opaque palette and selective
   emission separable from gameplay contracts; no external mesh-generation API.
4. **Renderer follow-up:** explicitly decide Floor Grip texture/PBR support vs
   documenting a simplified Option A material contract. Compare deck/metal/emission
   fixtures before changing code. Evaluate glass sorting only with a reproducing
   mixed-shading view; preserve opaque-first/read-only glass depth semantics.
5. **#227 after #226:** optimize only after preserving identities/material roles,
   UVs, spatial sidecars and overlap policies. Baseline bytes/primitives are not
   GPU texture memory or total frame draw counts. Do not merge glass into opaque.
6. **#228 after #226/#227:** implement these camera fixtures as actual static and
   temporal captures, add an injected-bad case and retain separate geometry,
   material, alpha, aliasing and precision diagnoses. **#229** retains all Apple
   M1 captures/performance signoff; no cloud check substitutes for that issue.

Option B may replace visual recipes with bevels/armor/richer materials later, but
must retain ship-local meter dimensions/pivot, collider boxes, all four interaction
markers, landing-foot/deck planes, ramp hinge/extension/animation, both monitor
normals/top-left UVs, lower-window sightlines, body-clear walking lanes, one
opaque/one glass policy (unless deliberately revised), and exporter-generated
character/renderer/ship contracts. Do not infer colliders from a new visual mesh.

## Validation and limits

Linux x86_64, Python 3.12.14. Model dependencies: pinned jsonschema 4.26.0.

- `python models/tools/diagnose_scout.py`: passed; JSON captured above.
- `PATH=/root/.cargo/bin:$PATH python -m unittest discover -s models/tests -v`:
  passed, 59 tests, 2 explicit Blender-dependent skips.
- No source/export regeneration, live GPU captures or Apple M1 tests were run.
  Reported flicker/proportion symptoms remain unconfirmed visually; this is the
  explicit unavailable-environment result allowed by #224, not a visual signoff.
