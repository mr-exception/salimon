# Assets

Owns checked-in game-ready assets and runtime metadata. Offline 3D
authoring sources belong in the root [models workspace](../../models/README.md);
its [shared contracts](../../models/contracts.md) separate visual meshes,
colliders, sockets, interaction markers, and optional LODs. Normal client builds
consume checked-in exports and never require Blender. The custom
[Salimon Scout](ship/README.md) includes matching glTF interchange,
a self-contained GLB, collision/interaction metadata,
an original floor texture, validation tooling, and licensing documentation.
The source/export contract includes modeled cockpit glazing and validated
seated/standing exterior sightlines.

The first generic authored resource export is
`resources/iron-fragment/model.glb`. Its editable Blender source, manifest,
preview, and regeneration instructions live in
[the iron fragment authoring directory](../../models/assets/resources/iron-fragment/README.md).
Runtime now embeds it as the even-ID iron chunk variant alongside
`resources/iron-fragment-shard/model.glb` for odd IDs. Both use the authoritative
fragment center and cube side; their visuals do not define mass or collision.

The first generic authored item export is `items/cargo-container/model.glb`.
Its source, manifest, preview, and editing/export instructions live in
[the cargo container authoring directory](../../models/assets/items/cargo-container/README.md).
It is a standalone asset-pipeline proof for future consumer integration.

Original generic spherical albedo and detail textures are generated
deterministically by `client/renderer/src/surface_textures.rs` at initialization.
That code is their editable source; no external imagery, downloaded assets, or
additional license obligations are involved. See the renderer's
[material documentation](../renderer/sphere-rendering.md).

The equipped mining tool uses `items/mining-tool/model.glb` at runtime. Its
[editable source and grip/material contract](../../models/assets/items/mining-tool/README.md)
use the generic item pipeline; the renderer embeds it independently of Blender.

Silicate deposits embed four opaque authored boulder/slab/ridge/scree GLBs.
The [source and scaling guide](../../models/assets/resources/silicate-deposit-boulder/README.md)
defines stable variant selection and presentation bounds; gameplay mass/mining
and streaming remain independent of mesh geometry.

Iron deposits embed four opaque authored nodule/vein/ledge/rubble GLBs.
The [source and scaling guide](../../models/assets/resources/iron-deposit-nodule/README.md)
defines stable selection and spherical presentation bounds independently of gameplay.
