# Iron fragment — chunk variant

`resource.iron-fragment` is the original Blender-authored iron resource from #84,
now integrated into gameplay as the even-ID chunk variant. Its existing 42
vertices, 80 triangles, material assignments and hierarchy are retained. The
bottom-centered mesh was recentered and uniformly enlarged by 2.2 for the
fragment renderer's unit-cube contract. No duplicate legacy asset remains.

![Exported chunk](preview.png)

The preview re-imports the runtime GLB in Blender; preview lights/camera are not
part of the source or export. Runtime uses flat base colors and local facet
shading, so Blender's metallic lighting differs from the game's presentation.

## Editable contract

- `source.blend` owns editable mesh/materials, authored originally with Blender
  4.5.3 LTS and adapted with Blender 4.0.2. No add-ons, external assets, textures
  or linked libraries are needed. Original Salimon geometry; no third-party art.
- Metric scale 1, Blender +Z up; export maps `(x,y,z)` to runtime `(x,z,-y)` once.
- Centered bounds: runtime X ±0.44 m, Y ±0.33 m, Z ±0.352 m. All vertices fit
  inside ±0.48 m. Root, group and mesh transforms are identity.
- Hierarchy: `ASSET_iron-fragment` → `Visual` → `Iron_Fragment`.
- Materials: `Iron_Ore`, `Iron_Inclusions`, `Oxide_Crust`; ordinary opaque glTF
  materials preserving dark iron faces, metallic inclusions and oxide patches.
- Root extras: `salimon.assetId = resource.iron-fragment`; mesh extras:
  `salimon.role = fragment-visual`. Collision policy remains `none`.

The renderer uniformly scales the mesh by the authoritative fragment cube side
and positions it at the world-owned center. Geometry never supplies gameplay
volume, mass, collision or mining yield. Stable fragment ID parity chooses this
chunk for even IDs and [the shard](../iron-fragment-shard/README.md) for odd IDs,
including carrying, dropping, ship-local movement and session streaming.

## Edit and export

Open `source.blend` and edit `Iron_Fragment`. Preserve baked identity transforms,
names, extras, material roles and centered ±0.48 m bounds. Save before exporting:

```sh
python models/tools/export_asset.py resource.iron-fragment --blender /path/to/blender
python models/tools/validate_asset.py resource.iron-fragment
BLENDER=/path/to/blender python -m unittest discover -s models/tests -v
```

The shared pipeline produces `client/assets/resources/iron-fragment/model.glb`;
Cargo embeds it without requiring Blender. Current metrics: 80 triangles, three
primitives/materials, no textures, 11,492 GLB bytes. See the
[completion report](../../../../reports/issue-89/README.md) for validation.
