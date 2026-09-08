# Scout interior fixture template library v1

Blender 4.5.3 source, parameterized by `author.py`; shared Scout materials remain
linked to the existing shared source library. Export with the standard asset
adapter and pinned authoring Python environment:

```sh
blender --background --factory-startup --disable-autoexec --python-exit-code 1 --python models/assets/fixtures/scout-interior-kit/author.py
python models/tools/export_asset.py fixture.scout-interior-kit --blender /path/to/blender-4.5.3
python models/assets/fixtures/scout-interior-kit/validate_templates.py
```

Shelf, Seat, Panel and PanelDamaged are **separate origin-centered templates**,
co-located in the library for extraction, not a composed scene or gameplay room.
The category validator applies ordinary geometry-quality checks separately to
each exact named template, with no overlap exemptions. It also validates the
whole manifest/GLB. All four templates pass normals, winding, degeneracy and
self-overlap checks. Source/runtime are meters, +X forward/+Y runtime up.

Current library: 60 triangles, four primitives, three existing shared materials,
no textures, 8,060 GLB bytes. Caps are 80 triangles / four primitives / three
materials / 32 KiB. No existing Scout cap is raised. Damage here is an authored
prototype variant; choosing it in a synthetic renderer workload is not gameplay
damage authority or presentation approval. Material/variant and pose inputs must
come from the owning physical domain before these prototypes can be installed.

Renderer geometry is immutable/shared across instances. The current runtime berth
still comes from its original named ship meshes and generated hinge, with a
versioned migration rather than an unrelated replacement model. See the
[canonical fixture contract](../../../../docs/modular-interior-fixtures.md).
