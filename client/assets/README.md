# Assets

Owns checked-in game-ready assets and legacy editable sources. New offline 3D
authoring sources belong in the root [models workspace](../../models/README.md);
its [shared contracts](../../models/contracts.md) separate visual meshes,
colliders, sockets, interaction markers, and optional LODs. Normal client builds
consume checked-in exports and never require Blender. Task 7 adds the custom
[Salimon Phase 0 Scout](ship/README.md), including deterministic procedural source,
Blender-importable glTF, a self-contained GLB, collision/interaction metadata,
an original floor texture, validation tooling, and licensing documentation.
Task 9 revises that same source/export contract with modeled cockpit glazing and
validated seated/standing exterior sightlines.

The first generic authored resource export is
`resources/iron-fragment/model.glb`. Its editable Blender source, manifest,
preview, and regeneration instructions live in
[the iron fragment authoring directory](../../models/assets/resources/iron-fragment/README.md).
It is ready for future consumer integration; current resource gameplay rendering
does not load it yet.

The first generic authored item export is `items/cargo-container/model.glb`.
Its source, manifest, preview, and editing/export instructions live in
[the cargo container authoring directory](../../models/assets/items/cargo-container/README.md).
It is a standalone asset-pipeline proof for future consumer integration.

Task 6's original generic spherical albedo and detail textures are generated
deterministically by `client/renderer/src/surface_textures.rs` at initialization.
That code is their editable source; no external imagery, downloaded assets, or
additional license obligations are involved. See the renderer's
[material documentation](../renderer/sphere-rendering.md).
