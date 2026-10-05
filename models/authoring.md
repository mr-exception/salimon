# Blender and AI authoring workflow

Blender edits offline sources. Codex may author through a configured Blender MCP
bridge or produce reviewed Blender Python automation. A bridge is optional and
is not a Cargo dependency, a runtime service, or an assumed built-in Blender
feature. This repository does not install a bridge or prescribe a third-party
provider. Read that provider's actual capabilities before using it; do not
invent tool names or require it for repeatable exports.

## Before editing

1. Read the asset's [manifest](manifest.md), README, shared
   [contracts](contracts.md), category extension, and dependent runtime consumers.
2. For an existing asset, open a copy/checkpoint of its source and inspect the
   hierarchy, dimensions, materials, proxies, and anchors. Preserve stable names
   and metadata. Commit deliberate changes to the tracked source; do not use an
   untracked open Blender session as the only source of truth.
3. For a new asset, use the [workspace layout](README.md#layout-and-ownership),
   choose a stable logical ID, define budgets and origin, and record original or
   third-party asset provenance/license. Keep external references out of exports.

## Prompt and inspect

Give the AI an asset-local brief: category, intended use, dimensions in meters,
pivot, visual style, material restrictions, budgets, required contracts, and
the names/transforms that must remain stable. Ask for small editable changes
and inspect the result before continuing. A preview alone cannot prove correct
collision bounds, UV orientation, metadata, scale, or runtime behavior.

For example, a future iron fragment brief could request a small low-poly rock,
bottom-centered origin, metric dimensions, an explicit collision policy, and
no sockets. A future mining-tool brief could request a grip socket with a
documented local orientation. Neither needs a cockpit, seat, or ship monitor.
Only request those assets once their own issues and pipeline blockers permit it.

Save the `.blend` with its required texture inputs, keeping paths portable.
Do not commit workstation-specific absolute paths or `.blend1`/temporary
autosave files. Document the Blender version and any add-ons needed to reproduce
the source/export. Do not depend on a bridge's session history to reconstruct it.

## Export and review

1. Save the source, then run the generic opt-in export for its logical identity
   or asset path. Headless export must use saved source and manifest data, not
   UI selection, current view, or a persistent MCP session.
2. Export only declared runtime content to the manifest-defined destination under
   `client/assets/`; preserve contract names and custom properties. Run common
   validation and the category validator. Failed validation must prevent an
   invalid output from being accepted as the runtime artifact.
3. Inspect the exported model as well as the editable source. Check dimensions,
   materials, hierarchy, collision/anchor transforms, budgets, and dependencies.
   Inspect the export diff and record intentional visual/contract differences.
4. For assets integrated into gameplay, run the relevant consumer tests and
   native E2E/smoke scenario. Screenshots prove appearance; structured state and
   tests prove interactions. Commit the source, manifest, runtime export, and
   required consumer/documentation updates together in a functional state.

Use the [generic export command](tools/README.md) for authored assets. The
[generic validator](tools/README.md#generic-validation) supports category extensions.
The scout migration is complete. Use the ship category adapter and
[scout export and validation commands](../client/assets/ship/README.md#source-and-regeneration)
for that asset. Normal `cargo build --workspace --locked` and
`scripts/build_game.py` continue to consume checked-in runtime assets without
Blender; editing a `.blend` alone will not change the game.
