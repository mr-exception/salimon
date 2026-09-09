# Renderer Invariants

1. The surface is configured and rendered only with a nonzero physical width and
   height.
2. The active surface configuration matches the latest accepted drawable size.
3. `wgpu` device, queue, surface, pipeline, shader, and presentation resources
   remain owned by the renderer boundary.
4. One successful `render` call submits and presents one validation-scene frame
   and returns measurements for that presented work.
5. Surface acquisition outcomes are returned to the runtime; the renderer does
   not exit the application or own lifecycle recovery or retry policy.
6. Rendering never advances simulation, mutates authoritative world/game state,
   or consumes raw gameplay-domain internals.
7. `CameraFrame` and `SceneInstance` are renderer DTOs, not authoritative world
   state. The runtime maps domain snapshots into them while preserving absolute
   `f64` positions; it does not rebase those positions.
8. The renderer subtracts camera position from instance centers and camera
   target in `f64` before converting relative values to GPU-facing `f32`.
9. View/projection construction and depth policy are renderer-owned. The scene
   path uses a right-handed infinite reverse-Z projection and matching depth
   target/comparison state.
10. The validation cuboids and shader exercise presentation only and must not be
    treated as canonical scene data.
11. The renderer uses low-level `wgpu` directly and must not introduce a full
    game engine.
12. Overlay input is a validated, borrowed RGBA image. The renderer may cache and
    uniformly fit and composite it but never owns diagnostics aggregation, text,
    toggle policy, or gameplay data.
13. GPU timestamps are requested only when the selected adapter supports them;
    readback must not block the presentation loop. Unsupported or pending data
    is reported explicitly.
14. Scene object counts describe instances retained by conservative frustum
    culling, and scene draw counts exclude diagnostics presentation. Total draw
    calls include the overlay when it is visible.
