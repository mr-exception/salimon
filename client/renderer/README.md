# Renderer

`salimon-renderer` is the custom rendering library introduced in Task 2. It uses
`wgpu` 30.0.1 to initialize the native surface/device/queue, configure the
surface, and draw a bootstrap triangle with a small WGSL shader. The triangle is
a presentation-path proof, not game or world content.

The public integration surface is intentionally narrow:

- `Renderer::new` initializes window-bound GPU state.
- `Renderer::resize` updates the drawable surface size.
- `Renderer::render` records, submits, and presents one frame.

The renderer owns GPU resources, surface configuration, shader/pipeline state,
render passes, and presentation. It does not own the native event loop, frame
clock, lifecycle policy, diagnostics overlay, or authoritative game/world state.
Surface acquisition failures are reported to the runtime so the runtime can
choose recovery or shutdown behavior.

Run the library through `cargo run --locked -p salimon-client`. See
[README.ai.md](README.ai.md), [architecture.md](architecture.md), and
[invariants.md](invariants.md) before changing its contract. Task 3 will add the
optional renderer/runtime diagnostics overlay; this bootstrap does not claim the
later Phase 0 1920x1080 performance target.
