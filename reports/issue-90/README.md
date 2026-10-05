# Issue #90 — Blender-authored silicate fragments

## Outcome

Silicate fragments now use two editable Blender-authored variants instead of the
procedural three-cuboid presentation. Stable fragment ID parity selects a broad
slab or taller ridge silhouette. Runtime still derives presentation scale from
the authoritative mass/volume-based fragment side length and keeps world-owned
position, mass, mining output, carrying, dropping, persistence, ship-local
movement, and collision behavior unchanged.

Both variants use the existing generic resource asset pipeline and the existing
batched opaque resource-mesh renderer introduced for water ice. No new gameplay
state or renderer/domain dependency was added.

## Asset validation

The editable sources were generated with Blender 4.0.2 on Ubuntu 24.04 and
exported through `models/tools/export_asset.py`.

| Variant | Triangles | Primitives | Materials | Textures | GLB bytes |
| --- | ---: | ---: | ---: | ---: | ---: |
| Slab | 14 | 2 | 2 | 0 | 3,204 |
| Ridge | 16 | 2 | 2 | 0 | 3,360 |

Both exports passed the generic manifest validator and stay inside the ±0.48 m
local presentation bound. Focused Python and Rust regression coverage checks
distinct geometry, stable identity mapping, authoritative sizing, and removal of
procedural silicate cuboids.

The pull request's normal native-build workflow is the final cross-platform
quality gate for formatting, Clippy, workspace tests, model tests, builds, and
Linux graphical scenarios.
