# Silicate fragment: ridge

This is an original low-poly Salimon resource fragment with an editable Blender
source and a checked-in runtime GLB. The ridge variant is one of two
silicate silhouettes selected deterministically from stable fragment identity.

The source uses metric units with Blender +Z up. Export converts once to runtime
+Y up. Geometry is centered in the authoritative one-meter presentation cube and
stays inside ±0.48 m on every axis. Runtime scales the model uniformly from the
world-owned fragment side length; the mesh does not define mass, volume,
collision, carrying, persistence, or mining output.

The visual uses two opaque, texture-free materials:
`Silicate_Dark` and `Silicate_Light`. Collision remains owned by the existing
fragment simulation rather than authored mesh geometry.

Regenerate from repository root:

```sh
blender --background --factory-startup --python-exit-code 1 --python models/assets/resources/silicate-fragment-ridge/create_source.py
python models/tools/export_asset.py resource.silicate-fragment-ridge --blender /path/to/blender
python models/tools/validate_asset.py resource.silicate-fragment-ridge
```

Normal builds embed the checked-in GLB and do not require Blender.
