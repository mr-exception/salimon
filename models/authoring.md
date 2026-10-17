# Blender and AI authoring workflow

Follow the authoritative [visual style](../docs/visual-style.md) for art decisions.

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

For the scout, edit the owning component rather than making assembly links local.
Shared root/groups/materials belong to its shared library; see the
[modular scout workflow](assets/ships/salimon-scout/README.md#open-edit-and-verify).

## Option A art contract (authoring protocol 1.0.0)

Use code-first editable parametric primitives and small named components. Keep
geometry specifications (meter envelope, pivots, anchors, collision proxies,
monitor UVs and walking clearances) independent from visual recipes. Read the
[#224 baseline](../reports/issue-224/README.md) before scout changes. Never fix
flicker by rescaling the ship or obscuring a competing surface.

The [art bible](../docs/visual-style.md) owns silhouette, shading, palette,
lighting and exception policies. Asset manifests own hard budgets. Option B
changes visual recipes with deliberate review, preserving the same spatial
interfaces; it is not an automatic upgrade.

## Local Codex bridge and headless fallback

Supported: Blender **4.5.3 LTS**, Python 3.10+, pinned model requirements,
repository-owned `salimon-authoring` **1.0.0**, MCP protocol **2024-11-05**.
No add-on or third-party MCP package is needed. The bridge launches headless
Blender operations from Codex while local Blender can inspect the resulting
scratch candidate interactively. It does not attach to an unsaved GUI session.

Evaluated [ahujasid Blender MCP](https://github.com/ahujasid/blender-mcp) and its
[security guidance](https://github.com/ahujasid/blender-mcp#limitations--security-considerations)
on 2026-10-15: a live socket/add-on and general Python execution provide broader
authoring control than this task needs. Upstream safe mode is not a project
sandbox. We do not install/support that provider or external model/download
services. Chosen alternative: repository-owned fixed-operation stdio bridge,
versioned with the code and usable without a persistent server or network port.

Opt in by adding this local server to Codex configuration (replace absolute paths):

```toml
[mcp_servers.salimon_authoring]
command = "/path/to/python"
args = ["/path/to/salimon/models/tools/authoring_bridge.py", "--blender", "/path/to/blender"]
```

Keep the normal Codex tool approval controls enabled locally. Review the asset
and requested operation before accepting a tool call. No operation accepts
Python, shell commands or arbitrary source/output paths. The bridge only writes
below `artifacts/authoring`; Blender auto-run scripts are disabled. It still
opens trusted repository `.blend` files and runs the installed Blender binary:
this is not an OS sandbox for hostile assets. Use a restricted OS account for
untrusted files. Production edits require deliberate source review and export;
no bridge operation promotes candidates. Stop Codex/remove the server stanza
for teardown; no daemon/add-on/socket remains. Delete `artifacts/authoring` to
roll back all sample outputs.

Same deterministic CLI without MCP or GUI:

```sh
python models/tools/authoring_bridge.py --blender /path/to/blender --operation inspect_asset --asset item.cargo-container
python models/tools/authoring_bridge.py --blender /path/to/blender --operation preview_asset --asset item.cargo-container
python models/tools/authoring_bridge.py --blender /path/to/blender --operation sample_palette --asset item.cargo-container
```

This vertical slice changes only the four cargo-container material base colors
in an isolated `candidate.blend`, retains all dimensions/transforms/names, and
renders front/side/three-quarter PNGs. Inspect JSON before/after, open candidate
beside original, and run the generic validator on the unchanged runtime asset.
Regenerate by rerunning the command; rollback by deleting the scratch folder.
Source/GLB promotion is a separate reviewed authoring action using the existing
exporter, never a side effect of a preview. With no Blender installed, inspect
runtime geometry via `diagnose_scout.py`/generic validators; source rendering is
unavailable and must be reported as such, not substituted by metadata tests.
