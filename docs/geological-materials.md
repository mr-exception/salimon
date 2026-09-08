# Geological surface materials (#613)

`runtime::terrain_materials` maps the [canonical geology](planetary-geology.md)
profile, weathering, exposure and layer depth into neutral luminance and strata
parameters. It never alters terrain height, normal, resource generation, collision,
planet definition or player knowledge. LOD, light, render quality, camera and
mutable random state are absent from the mapping. Resource mesh/material palettes
and the existing six body swatches remain untouched. Optional visual micro-cover
is not enabled; no new physical or visual seed coupling is introduced.

Rock shading darkens exposed bedrock and retains a coherent weathering response.
Ice receives restrained layer contrast and a new generic `SurfaceMaterial::Ice`
style using the approved pale U11 ice hue [181,222,229] sRGB, converted to linear
texture values. It occupies array slot 7; slot 6 remains the exact existing
shared detail tile, and the first six slots retain their numerical texel goldens.
This adds an ice presentation capability, not new ice deposits or an icy Vesper.
Current runtime profiles stay the #612 selections; explicit ice authoring is
supported by the recipe/renderer and exercised in portable/offscreen tests.

Frozen Vesper's homogeneous profile maps to unit luminance and zero layer strength.
Its existing Slate ground/palette and published terrain remain unchanged. No
new region colors or unapproved physical geography are introduced. Representative
Earth/Moon/Mars fixtures demonstrate the nonhomogeneous rock mapping; they remain
historical compressed fixtures, not scientific planetary reconstructions.

## Runtime and GPU boundary

`terrain_patches::generate` samples the same geology-owned authoritative ground
for vertex position/normal and material observations. Material cache keys contain
`SLMMAT01` plus the complete `SLMGEO01` bytes, binding the upstream terrain and
profile/version. Tile regeneration/revisit is deterministic. Physics continues
to query full terrain independently of render LOD and uses no GPU attributes.

`TerrainVertex` grows from 36 to 52 bytes: position/normal/far-position followed
by a luminance scalar and [elevation meters, layer thickness meters, strength].
GPU attributes are locations 0–4. All values must be finite, luminance is bounded
0.5–1.5, thickness is at least 0.01 m and strength is 0–1. Encode/layout/upload
accounting change together. The existing 32 MiB cache and 2 MiB/frame upload
bounds still apply to the actual larger buffers; draw/triangle topology is unchanged.
One extra 512×256 RGBA mip chain holds ice; no extra draw or pipeline is added.

The fragment shader interpolates these observations and uses elevation/thickness
for restrained layer bands. Derivative filtering smoothly converges to the same
mean when bands become subpixel; it does not use a quality-dependent random field.
The same factor multiplies existing day/night lighting and palette. It changes
neither normals, vertex morphing, reverse-Z depth nor physical silhouettes. This
is scalable geological presentation, not physical excavation or new atmosphere.

## Validation

- Runtime `terrain_materials` tests: profile/codec/query parity and Vesper neutrality.
- `terrain_patches::geology_material_tests`: shared-location LOD factors,
  terrain/vertex agreement and material-bound cache identity.
- Existing terrain mesh/seam/contact/cache regressions retain physical ownership.
- Renderer `surface_textures` goldens protect old swatches/detail; ice checks
  preserve its hue family and slot. `terrain_style_reference` exports the neutral
  production pipeline, while `terrain_geology_reference` exports rock/ice under
  day/night light at near/mid/far/transition/orbital distances, frozen-repeat and
  transition-continuity assertions plus actual buffer/timing observations.
- `scenarios/evidence/geology-{earth,moon,mars}.json` exercise the real runtime
  mapping at 40/400/4,000 m above the sampled north-pole ground using opt-in
  engineering cameras, without moving gameplay state. `geology-vesper.json`
  retains the unchanged canonical control. All four explicitly request a
  1280×800 drawable matching the capture desktop. `scout_reference` tests
  guard sampled-ground clearance and reject unsupported bodies.
- Run workspace test/build/Clippy/format/docs gates, Linux production wgpu scenarios
  and retain original captures/results in `reports/issue-613/`.

Offscreen patches are declared synthetic shader fixtures, not proof of live world
mapping. Native runtime scenarios plus portable mapping/geometry tests cover that
boundary. Linux llvmpipe timing is actual software-renderer evidence, not M1/Metal
FPS or subjective appearance approval; those stay #563/#561. No baseline is promoted.
