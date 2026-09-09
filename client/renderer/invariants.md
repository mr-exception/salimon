# Renderer Invariants

1. The surface is configured and rendered only with a nonzero physical width and
   height.
2. The active surface configuration matches the latest accepted drawable size.
3. `wgpu` device, queue, surface, pipeline, shader, and presentation resources
   remain owned by the renderer boundary.
4. One successful `render` call submits and presents one bootstrap frame and
   returns measurements for that presented work.
5. Surface acquisition outcomes are returned to the runtime; the renderer does
   not exit the application or own lifecycle recovery or retry policy.
6. Rendering never advances simulation, mutates authoritative world/game state,
   or consumes raw gameplay-domain internals.
7. The bootstrap triangle and shader validate presentation only and must not be
   treated as canonical scene data.
8. The renderer uses low-level `wgpu` directly and must not introduce a full game
   engine.
9. Overlay input is a validated, borrowed RGBA image. The renderer may cache and
   composite it but never owns diagnostics aggregation, text, toggle policy, or
   gameplay data.
10. GPU timestamps are requested only when the selected adapter supports them;
    readback must not block the presentation loop. Unsupported or pending data
    is reported explicitly.
11. Scene draw/object counts exclude diagnostics presentation. Total draw calls
    include the overlay when it is visible.
