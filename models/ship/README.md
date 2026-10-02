# Salimon ship authoring

This folder is the Blender authoring home for the Salimon ship.

## Source

Preferred source file:

```text
models/ship/salimon_phase0_ship.blend
```

The file is intentionally not fabricated in Git: create the initial source from the validated
checked-in glTF with `models/scripts/bootstrap_ship.py`, inspect it in Blender, then commit the
resulting `.blend` once the migration is validated.

## Runtime output

The client continues to consume:

```text
client/assets/ship/export/salimon_phase0_ship.glb
```

Do not change that path without updating the renderer.

The current ship has more than visual geometry: node names, monitor UVs, material roles, collision
metadata, interaction markers, and dimensions are validated. A Blender-authored replacement is only
accepted when the existing validator and Rust tests still pass.

## AI-assisted modeling

Codex connected to Blender through MCP should operate on the `.blend` source in this directory.
Prompts should specify physical dimensions, triangle/material budgets, object names, interaction
boundaries, and which parts must remain separate. After every meaningful modeling change, render a
preview and run the export/validation flow rather than editing the runtime GLB by hand.
