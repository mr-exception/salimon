# 3D model authoring workspace

`models/` owns editable, offline 3D art and its authoring contracts. It supports
ships, resources, items, structures, props, characters, and vehicles without
making a ship's cockpit or collision rules mandatory for other assets.

The runtime consumes checked-in, validated exports under `client/assets/`.
Blender is an **authoring dependency only**: normal Cargo builds, native build
scripts, and gameplay must not invoke Blender, require a Blender installation,
or read `.blend` files. No game engine is introduced.

## Layout and ownership

Each new authored asset lives at `models/assets/<category>/<asset-slug>/`.
Category directories use the plural names `ships`, `resources`, `items`,
`structures`, `props`, `characters`, and `vehicles`. Add categories through the
same shared contract; do not copy ship tooling for each category.

| Location | Owner and purpose |
| --- | --- |
| Manifest-defined `.blend` source (normally `source.blend`; scout uses a linked assembly) | Editable geometry, materials, collision proxies, and spatial anchors |
| The asset's `textures/` directory | Editable texture inputs; pack or resolve dependencies reproducibly |
| The asset's `manifest.json` | Logical identity, source/export paths, budgets, required names, and category contract; see [manifest v1](manifest.md) |
| The asset's `README.md` | Design intent, origin/pivot, licensing, authoring instructions, and contract notes |
| `models/tools/` | Generic offline export and validation tooling |
| `client/assets/<category>/<asset-slug>/` | Checked-in runtime GLB and any explicitly required runtime metadata/textures |
| Rust domain modules | Gameplay policy and authoritative simulation; consume spatial contracts through typed boundaries |
| `client/renderer/` | Rendering and asset import; no dependency on editable authoring sources or Blender |

Use a stable logical ID such as `ship.salimon-scout`, `resource.iron-fragment`,
or `item.mining-tool`. Identity is independent of a folder name or runtime file
path. Moving an asset must not change its gameplay identity. The manifest will
map identity to paths; this issue does not add an asset catalog or a loader.

Runtime destinations are declared per asset, so the existing singular
`client/assets/ship/export/salimon_phase0_ship.glb` remains a valid destination.
Do not relocate it merely to match the default layout for new assets.

Read [manifest v1](manifest.md) and [shared contracts](contracts.md) before
creating an asset, and [the authoring workflow](authoring.md) before using
Blender or an AI bridge.

## Current implementation and migration boundary

This workspace defines the authoring design and [manifest schema v1](manifest.md).
The ship/resource/item examples are design fixtures with future source paths,
not migrated assets. The [generic export command](tools/README.md) is available;
the [generic validator](tools/README.md#generic-validation) checks runtime
contracts. The [iron fragment](assets/resources/iron-fragment/README.md) is the
first committed Blender-authored resource, with a validated runtime GLB. It
uses shared tooling without a category extension; its evolved chunk and shard
variants now render gameplay fragments with stable identity and physical sizing.
The [cargo container](assets/items/cargo-container/README.md) is the first
committed Blender-authored item, using the same pipeline and no category extension.

The [scout Blender source](assets/ships/salimon-scout/README.md) now owns visual
geometry/materials and produces the checked-in runtime GLB through its ship
adapter (#80). Shared export and budget checks run with the scout category
validator, which reads the authored GLB and declarative preservation/monitor
contracts directly. Spatial layouts come from authored proxies/markers (#81).
The procedural generator and reference preview tool are retired (#83). Blender
source plus shared export/validation and the scout adapter are the supported
authoring path; generated spatial layouts come from authored proxies/markers.
Runtime-generated spherical textures remain owned by the renderer as described
in [client assets](../client/assets/README.md); they need no `.blend` source.

| Implemented issue | Deliverable |
| --- | --- |
| #76 | Shared manifest schema and ship/resource/item examples |
| #77 | Generic, opt-in headless Blender export command |
| #78 | Generic validation and category extension points |
| #79 | Editable scout `.blend` and preservation contract |
| #80 | Switch the current ship visual export to Blender source |
| #81–#83 | Migrate spatial contracts, decouple validation, retire the legacy generator |
| #84/#85 | Prove the shared pipeline with a resource and an item |

These completed migration tasks preserve the existing runtime behavior and
metadata contract. Future authoring changes must pass the same validation gates.

The scout uses nine independently editable component libraries and a final linked
assembly. See its [component ownership and editing workflow](assets/ships/salimon-scout/README.md#open-edit-and-verify).

The [mining tool](assets/items/mining-tool/README.md) is a generic authored item
integrated into first-person rendering, with an identity grip socket and
runtime-driven status material.

Four [silicate deposit variants](assets/resources/silicate-deposit-boulder/README.md)
use the generic resource pipeline and render with stable identity selection and
authoritative spherical bounds.
