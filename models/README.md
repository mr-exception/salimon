# 3D model workspace

This directory owns **editable 3D authoring sources** for Salimon. Runtime code must not load files from this directory directly.

The asset flow is:

```text
models/<asset>/*.blend
        |
        | Blender / Codex + Blender MCP
        v
client/assets/<asset>/export/*.glb
        |
        v
client/renderer
```

## Rules

- Blender `.blend` files are the preferred source for hand-authored game models.
- Keep model scale in meters and use the same axis conventions documented by the target runtime asset.
- Preserve stable node/object names that the runtime or validators consume.
- Export game-ready GLB files into `client/assets/<asset>/export/`.
- Runtime code only consumes exported assets; it must not depend on Blender, MCP, or Python authoring tools.
- Generated GLB files remain checked in so native builds do not require Blender.
- Blender backup files (`*.blend1`, `*.blend2`, …) are ignored.
- Procedural runtime geometry such as analytic planets remains renderer-owned and does not need a Blender source.

## Current ship migration

The existing Phase 0 ship predates this workspace. Its deterministic Python generator under
`client/assets/ship/source/` remains authoritative until a Blender source has been created and
validated against the current runtime/metadata contract.

To bootstrap an editable Blender file from the checked-in ship:

```sh
blender --background --python models/scripts/bootstrap_ship.py
```

This writes `models/ship/salimon_phase0_ship.blend` without changing the runtime GLB.

After editing the Blender source, export it with:

```sh
blender --background models/ship/salimon_phase0_ship.blend \
  --python models/scripts/export_glb.py -- \
  client/assets/ship/export/salimon_phase0_ship.glb
```

Then run the existing ship validator before accepting the export:

```sh
python3 client/assets/ship/source/validate_salimon_phase0_ship.py
cargo test --workspace --locked
```

Until the migration is proven equivalent, do not delete the legacy generator.
