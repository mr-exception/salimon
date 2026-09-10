# Runtime Invariants

1. Window-bound renderer state is created only when the application lifecycle
   permits a native window and is never rendered before initialization.
2. Zero-width or zero-height drawable sizes are not used to configure or render
   a GPU surface.
3. Every accepted nonzero resize reaches the renderer before the next presented
   frame.
4. Frame and update intervals come from monotonic clocks and clamp unreported
   long OS stalls; the renderer never owns or advances simulation time.
5. Recoverable surface loss rebuilds or reconfigures rendering state. A transient
   surface acquisition failure is delayed and does not crash or hot-loop the
   application.
6. A close request or destruction of the sole window exits the event loop
   cleanly. Unrecoverable renderer failures are reported before termination.
7. The runtime may orchestrate renderer/platform capabilities but must not own
   rendering implementation, authoritative domain state, or backend behavior.
8. Future world, character, and ship code must not receive raw `winit` events or
   `wgpu` resources from the runtime.
9. The engineering overlay is hidden by default and toggles only on an initial
   physical F3 press, never on release or key repeat.
10. Diagnostics observes typed runtime, renderer, and future domain snapshots;
    it never becomes authoritative state or changes simulation behavior.
11. Suspension, occlusion, zero-sized drawables, and renderer reconstruction
    reset the rolling timing window before presentation resumes.
12. Native input is translated into typed portable commands; `winit` event types
    never enter the world crate.
13. Renderer reconstruction preserves portable camera state. Lifecycle gaps
    reset the update clock so the prototype cannot jump forward on resume.
14. Each world snapshot maps exactly six body proxies followed by exactly three
    noncanonical precision markers. Body centers remain absolute `f64`; only
    radii narrow to `f32` as renderer-facing half-extents.
15. Diagnostics body observations preserve catalog order/names and report finite,
    nonnegative camera-to-nominal-surface distances for all six bodies.
