# Mining tool

`item.mining-tool` is an original Salimon handheld extractor authored with
Blender 4.5.3 LTS Python automation. The editable `source.blend` owns geometry
and materials; no external assets, textures, linked libraries or add-ons are
used. The optional `create_source.py` reconstructs the initial design and
**overwrites** the source; normal art edits should use Blender and export the
saved source instead.

The chamfered housing, battery, ribbed grip, cooling fins and cylindrical emitter
use four opaque, texture-free materials. `Tool_Status` identifies the lens and
indicator panels, amber when idle and teal while actively extracting. The
renderer changes this material region only; it does not derive mining policy.

## Grip and runtime contract

- Metric source: +X forward, +Z up. Shared export maps `(x,y,z)` to `(x,z,-y)`.
- `ASSET_mining-tool` parents `Visual` and `Sockets`; `SOCKET_Grip` is identity
  at the grip center. Every node has identity transforms, with shape baked into
  mesh vertices. The renderer rejects transformed nodes or a missing grip.
- Runtime bounds are approximately X -0.095…0.291 m, Y -0.115…0.1495 m,
  Z -0.058…0.058 m. No collision, use marker, animation, textures or LODs.
- Grip placement is camera-local: 0.55 m forward, 0.20 m right, 0.20 m down.
  Local +X follows camera forward and +Y follows orthonormal camera up. The
  renderer uses projection directly, so pitch, yaw, gravity and distant world
  origins cannot move the grip on screen. Regular reverse-Z depth remains active.
- This visual neither supplies the mining ray nor changes reach, extraction,
  targeting, equip/stow controls or resource identity. The aim marker stays
  separate. Surface/gameplay/equipped gates remain runtime-owned.
- Budgets: 2,000 triangles, 16 primitives, 4 materials, 0 texture bytes (schema
  cap 1), 192 KiB GLB. Current export: 640 triangles, 12 primitives, 4 materials,
  no textures, 53,044 bytes, rendered in one draw.

## Edit, export, validate

Preserve the grip-centered pivot, identity transforms, hierarchy, names and
`Tool_Status` material. Edit mesh vertices rather than leaving object transforms.

```sh
python models/tools/export_asset.py item.mining-tool --blender /path/to/blender
python models/tools/validate_asset.py item.mining-tool
BLENDER=/path/to/blender python -m unittest discover -s models/tests -v
cargo test -p salimon-renderer --locked
```

Normal Cargo builds embed `client/assets/items/mining-tool/model.glb` and never
invoke Blender. Native appearance/behavior evidence lives in
[issue #92's report](../../../../reports/issue-92/README.md).
