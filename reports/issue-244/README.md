# #244 — authored surface remediation

Baseline: `04609cb53b9555f12212e623cc1615aa145da070` (2026-10-16).
Portable authored-geometry work is complete. Native Apple M1 visual/temporal
signoff remains [#229](https://github.com/mr-exception/salimon/issues/229).

## Outcome

Inspected the evaluated linked scout assembly, owning source libraries, and item
sources in Blender 4.5.3 LTS. Partitioned coplanar panels to give different-material
surfaces one owner; trimmed lower glazing against opaque frame/nose surfaces.
Edited six owning scout components without making assembly links local. Cargo
panel/frame/trim surfaces now have one owner. Mining-tool reconstruction removes
32 collapsed bevel triangles rather than granting a warning allowance.

The repair preserves the meter envelope, pivots, transforms, material definitions,
anchors, colliders, monitor UVs and spatial contracts. Generated Rust spatial,
landing and ramp files are byte-identical to the baseline. No scale, palette,
depth bias, runtime renderer or gameplay policy changes.

| Asset | Overlap candidates before → after | Degenerates before → after | Triangles / cap | GLB bytes / cap |
| --- | ---: | ---: | ---: | ---: |
| Scout | 935 → 821 | 0 → 0 | 6,281 / 6,500 | 525,356 / 557,056 |
| Cargo container | 36 → 32 | 0 → 0 | 945 / 1,000 | 73,704 / 131,072 |
| Mining tool | 70 → 70 | 32 → 0 | 2,064 / 3,000 | 225,176 / 393,216 |

Counts are geometric candidates, not confirmed visible flicker counts. Splitting
panels changes triangle-pair counts even where a contact area is unchanged.
All remaining exemptions have named construction reasons and exact count caps;
the temporary remediation reasons and mining-tool degenerate warning are gone.
The [complete named-pair classification](CLASSIFICATION.md) and
[structured witnesses](surface-classification.json) include every baseline pair,
materials, facing direction, count changes and representative runtime coordinates.

Retained contacts are opposing internal attachment caps, same-material planar
primitive joins, or concealed articulated leaf/tray/frame end and side caps.
These must remain available when the door opens or the tray extends. For mixed
articulated witnesses, normal-ray probes locate surrounding frame/leaf/header
surfaces; they do not establish visibility from every camera or every door pose.
The new monitor-housing seam witness is an approximately 9e-9 m² same-material
float32 tessellation edge; two same-material belly edge witnesses are similarly
microscopic. They are included explicitly rather than reported as eliminated.
The guidance-light rear caps, ceiling fixture attachment caps and threshold/deck
contact are opposing flush joins; their outward visible faces are separate.

## Reproduction

`models/tools/repair_authored_surfaces.py` is an explicit Blender source migration,
not an export hook. It assigns coplanar material ownership with convex clipping,
interpolates positions/UVs, preserves original normals and modifies owning
libraries only. `surface_partition.py` contains the tested clipping operation.
Do not run the migration as a normal build step. Export saved sources normally:

```sh
python models/assets/ships/salimon-scout/export.py --blender /path/to/blender
python models/tools/export_asset.py item.cargo-container --blender /path/to/blender
python models/tools/export_asset.py item.mining-tool --blender /path/to/blender
```

The mining-tool `create_source.py` calls the same cleanup before saving, so a
fresh reconstruction cannot reintroduce the 32 degenerate bevel triangles.
Reconstructed it separately, exported and validated it, then restored the reviewed
source checkpoint: 2,064 triangles / 15 primitives / 4 materials / 68,079 texture
bytes / 225,176 GLB bytes. Regenerated the toolbar icon from the saved source.

## Validation

Linux x86_64; Python 3.12; Blender 4.5.3 LTS; jsonschema 4.26.0;
Rust 1.99.1; pinned local Khronos validator installed with
`npm ci --prefix models/tools/gltf`.

Passed:

- All three export commands above, with staged validation and unchanged budgets.
- `blender -b --python-exit-code 1 --python models/assets/ships/salimon-scout/verify_source.py`:
  saved source/export geometry, hierarchy, material and monitor UV parity.
- `blender -b --python-exit-code 1 --python models/assets/ships/salimon-scout/verify_modular.py`:
  nine portable linked components and independent hull-edit propagation.
- `python models/assets/ships/salimon-scout/validate.py`.
- `python models/tools/validate_quality.py <asset>` for all three logical IDs.
- `BLENDER=/path/to/blender python -m unittest discover -s models/tests -v`:
  **86 tests passed, no skips**, including real Blender and Khronos integrations.
- `python -m unittest discover -s models/tests -p test_surface_partition.py -v`:
  three focused tests rerun after environment resume.
- Affected consumer test binaries produced during
  `cargo test -p salimon-renderer -p salimon-character -p salimon-ship --locked`:
  renderer **58 passed / 9 opt-in tests ignored**, character **64 passed**, ship
  **61 passed**. The resumed environment lost its installed Rust toolchain;
  reran the preserved compiled executables directly under `target/debug/deps/`
  (`salimon_renderer-89e0f0844b1445db`, `salimon_character-030653bdbc5f4c76`,
  `salimon_ship-0a00678ec75d3019`). No successful final Cargo invocation is claimed.
- `git diff --check`.

No native game launch, Metal capture, temporal flicker or M1 performance result is
claimed. This issue deliberately excludes native signoff, which remains #229.

## Export inspection

CPU Cycles, neutral fixed exposure/key, front/side/three-quarter views of the
**re-imported runtime GLBs**, not substitute gameplay screenshots. Inspected all
three assets; source parity is established separately by the checks above.

| Asset | Front | Side | Three-quarter |
| --- | --- | --- | --- |
| Scout | [view](scout/front.png) | [view](scout/side.png) | [view](scout/three-quarter.png) |
| Container | [view](cargo/front.png) | [view](cargo/side.png) | [view](cargo/three-quarter.png) |
| Mining tool | [view](tool/front.png) | [view](tool/side.png) | [view](tool/three-quarter.png) |
