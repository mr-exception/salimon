# Diagnostics Invariants

1. Diagnostics is hidden by default, and hidden diagnostics return no overlay
   image or text view.
2. Frame statistics contain at most 120 successfully presented samples with a
   nonzero frame interval.
3. A first frame or a frame after a lifecycle reset cannot contribute a zero
   interval to FPS, average frame time, or p95 frame time.
4. Normal text and pixel refreshes occur no more often than once per 250
   milliseconds of accumulated valid presented-frame time. Explicit toggle,
   density, reset, and warning changes may refresh immediately.
5. CPU render timing and GPU execution timing remain separately labeled. GPU
   unsupported, pending, and measured states are never conflated.
6. Missing, non-finite, negative distance/speed, empty camera phase, or
   out-of-range thruster values render as `N/A`; diagnostics never creates
   authoritative gameplay values or labels camera state as player/ship state.
7. Diagnostics observes borrowed domain values without retaining domain
   references or gaining authority to mutate them.
8. `OverlayImage` borrows an internally owned RGBA8 buffer whose byte length is
   exactly `width * height * 4`; its revision changes whenever those pixels are
   rebuilt.
9. Display scaling changes glyph pixel density but does not change metric values
   or their units.
10. The crate remains standard-library-only and contains no `winit`, `wgpu`,
    engine, backend, persistence, or networking dependency.
