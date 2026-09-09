# Renderer

`salimon-renderer` is the custom rendering library introduced in Task 2 and
instrumented in Task 3. It uses `wgpu` 30.0.1 to initialize the native
surface/device/queue, draw the bootstrap triangle, composite an optional generic
RGBA overlay, and report renderer-owned measurements. The triangle remains a
presentation-path proof, not game or world content.

The public integration surface is intentionally narrow:

- `Renderer::new` initializes window-bound GPU state.
- `Renderer::resize` updates the drawable surface size.
- `Renderer::render` accepts an optional borrowed `OverlayImage`, records,
  submits, and presents one frame, then returns `RenderStats` with a successful
  `RenderOutcome::Presented`.

The renderer owns GPU resources, surface configuration, shader/pipeline state,
render passes, generic overlay composition, and presentation. It does not own
the native event loop, frame clock, lifecycle policy, diagnostics content or
visibility policy, or authoritative game/world state. Surface acquisition
failures are reported to the runtime so the runtime can choose recovery or
shutdown behavior.

`RenderStats` reports CPU encoding/submission wall time, the bootstrap scene's
visible/rendered object count, scene draw calls, total draw calls including the
optional overlay, GPU pass time, and GPU allocator totals. Timestamp queries are requested only when the adapter
optional overlay, GPU pass time, and GPU allocator totals. Timestamp queries are
requested only when the adapter supports them and are read through a
non-blocking three-slot ring. Until the first result arrives, timing is
`Pending`; unsupported adapters report `Unsupported`. Allocator totals are
sampled periodically and remain unavailable when the backend cannot generate a
report. These are renderer/GPU metrics, not process RSS or gameplay state.

Run the library through `cargo run --locked -p salimon-client`. See
[README.ai.md](README.ai.md), [architecture.md](architecture.md), and
[invariants.md](invariants.md) before changing its contract. The Task 3 overlay
does not claim the later Phase 0 1920x1080 performance target; Task 11 owns that
benchmark.
