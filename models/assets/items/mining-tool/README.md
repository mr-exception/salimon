# Mining tool

`item.mining-tool` is original Salimon equipment authored in Blender 4.5.3 LTS.
Editable `source.blend` owns geometry, UVs and four opaque materials. The
reconstruction recipe `create_source.py` **overwrites** the source and its original
`surface.png`; normal art edits should edit/save the Blender source instead.
All geometry, texture imagery and icon artwork are original project assets; no external texture sources, add-ons or linked libraries.

## Manufactured surfaces and runtime texture contract (#142)

Chamfered orange coated-polymer housing and battery panels, dark ribbed rubber,
brushed steel rails/collars/fasteners, panel seams, cooling vents, battery contacts,
trigger/guard and emitter rings make the extractor readable at handheld scale.
A deterministic 256×256 opaque RGB PNG provides four padded 128×128 atlas tiles:
polymer grain/seams/hazard marking and light scuffs, rubber ribs, brushed metal
and the status surface. `surface.png` is packed in the source and embedded in the
GLB. UVs are baked planar face projections into the appropriate padded tile;
the export has one image, shared by every material's `baseColorTexture` on
`TEXCOORD_0`. Colors are in sRGB; material metallic/roughness factors remain
separate. No normal, metallic/roughness, emissive or occlusion maps are supported.

`held_item.rs` validates/decodes the embedded PNG once, uploads nine sRGB mip
levels (averaged in linear color), and samples authored UVs in `held_item.wgsl`.
Camera-local fill and metallic/roughness-dependent highlights distinguish matte
rubber/polymer from steel. This is inexpensive manufactured-material shading,
not world PBR/IBL. `Tool_Status` remains amber idle / teal actively extracting;
only that region changes. The source's status tile is amber for icon rendering.

## Grip and runtime contract

- Metric source: +X forward, +Z up. Shared export maps `(x,y,z)` to `(x,z,-y)`.
- `ASSET_mining-tool` parents `Visual` and `Sockets`; `SOCKET_Grip` is identity
  at the grip center. Every node has identity transforms, with shape baked into
  mesh vertices. The renderer rejects transformed nodes or a missing grip.
- Runtime bounds: approximately X -0.103…0.291 m, Y -0.118…0.1495 m,
  Z -0.064…0.066 m. No collision, use marker, animation or LODs.
- Grip placement stays camera-local: 0.55 m forward, 0.20 m right, 0.20 m down.
  The existing `GRIP_TO_VIEW`/camera projection composition and ordinary
  reverse-Z depth preserve placement through pitch, yaw, gravity and distant origins.
- This visual neither supplies the mining ray nor changes reach, extraction,
  targeting, equip/stow controls, carrying or resource identity. Reticle and
  surface/gameplay/equipment gates remain runtime-owned.
- Budgets: 3,000 triangles, 16 primitives, 4 materials, 96 KiB encoded embedded
  textures, 384 KiB GLB. Current export: 2,096 triangles, 15 primitives,
  4 materials, one 68,079-byte PNG, 226,380-byte GLB and one scene draw.
- GPU atlas budget: 256 KiB base level / 349,524 bytes including all nine mips.
  CPU decoding uses 256 KiB RGBA plus temporary mip/decode buffers at startup;
  no per-frame texture regeneration/upload. Icon: 128×128 RGBA (64 KiB decoded),
  bounded PNG decoder shared with the atlas; excluded from GLB texture metrics.

## Toolbar artwork

`render_icon.py` renders the saved source orthographically with transparent
padding into `client/assets/items/mining-tool/icon.png`. It adds temporary
camera/lights outside the asset hierarchy and does not save them to the source.
Regenerate the icon after source art changes so model/toolbar shape and palette agree.
Renderer crops visible alpha bounds (threshold 16/255), uniformly fits artwork
within a 26×26 logical-pixel box and centers that box in the 34×34 slot. Canvas
margins never affect alignment. Number glyphs draw last in their upper-left
reserved area; the 4-pixel artwork padding protects selection borders. The
existing 1x–2x DPI/cache and viewport fitting policy remains in place.

## Edit, export, validate

Preserve grip pivot, identity hierarchy/names, authored UVs and `Tool_Status`.
Edit mesh vertices rather than leaving object transforms. The packed image
makes the editable source self-contained.

```sh
python models/tools/export_asset.py item.mining-tool --blender /path/to/blender
/path/to/blender --background models/assets/items/mining-tool/source.blend --python-exit-code 1 --python models/assets/items/mining-tool/render_icon.py
python models/tools/validate_asset.py item.mining-tool
BLENDER=/path/to/blender python -m unittest discover -s models/tests -v
cargo test -p salimon-renderer --locked
```

To deliberately reconstruct the design, run Blender with `--factory-startup
--python models/assets/items/mining-tool/create_source.py`, then export and render
the icon using the commands above. Normal Cargo builds embed the GLB/icon and
never invoke Blender. Reviewed native before/after evidence and limits are in
[issue #142's report](../../../../reports/issue-142/README.md).
