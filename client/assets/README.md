# Assets

Reserved for editable art sources and exported game-ready assets. Phase 0 calls
for a custom Salimon ship created with Blender or an equivalent external workflow,
with source files committed alongside glTF/GLB exports. Supporting public assets
must have documented sources/licenses. No art assets are included in task 1.

Task 6's original generic spherical albedo and detail textures are generated
deterministically by `client/renderer/src/surface_textures.rs` at initialization.
That code is their editable source; no external imagery, downloaded assets, or
additional license obligations are involved. See the renderer's
[material documentation](../renderer/sphere-rendering.md).
