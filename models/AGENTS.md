# 3D authoring instructions

This directory is the editable DCC workspace for Salimon models.

When using Blender manually or through Codex/Blender MCP:

1. Edit `.blend` sources here, not exported GLB files under `client/assets`.
2. Use meters and preserve the target asset's documented coordinate system.
3. Keep runtime-significant object names stable. For the Phase 0 ship this includes names such as
   `Exit_Door`, `Monitor_Center`, `Monitor_Port`, and `Monitor_Starboard`.
4. Prefer separate named objects for independently animated, interactive, emissive, glass, or
   collision-significant parts.
5. Keep topology intentionally modest; game-ready geometry is preferred over subdivision-heavy DCC
   meshes.
6. Export to the matching `client/assets/<asset>/export/` path and run that asset's validator plus
   workspace tests.
7. Do not add Blender or MCP dependencies to the native client.

The current Phase 0 ship still has legacy generated-source metadata. Treat
`client/assets/ship/README.md` and its validator as the compatibility contract while migrating it.
