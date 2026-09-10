# Assets

Owns editable art sources and exported game-ready assets. Task 7 adds the custom
[Salimon Phase 0 Scout](ship/README.md), including deterministic procedural source,
Blender-importable glTF, a self-contained GLB, collision/interaction metadata,
an original floor texture, validation tooling, and licensing documentation.
Task 9 revises that same source/export contract with modeled cockpit glazing and
validated seated/standing exterior sightlines.

Task 6's original generic spherical albedo and detail textures are generated
deterministically by `client/renderer/src/surface_textures.rs` at initialization.
That code is their editable source; no external imagery, downloaded assets, or
additional license obligations are involved. See the renderer's
[material documentation](../renderer/sphere-rendering.md).
