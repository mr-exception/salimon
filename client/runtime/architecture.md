# Runtime Architecture

## Responsibility

The runtime is the native host and composition root. It translates `winit`
lifecycle/window events into a small set of renderer operations and owns timing
that future portable simulation code may consume.

```text
winit event loop
    -> salimon-client lifecycle + frame clock
        -> salimon-renderer new / resize / render
            -> wgpu surface and GPU work
```

The dependency direction is one-way: the runtime depends on the renderer. The
renderer does not call into the runtime, and neither component depends on future
world, character, or ship modules.

## Lifecycle flow

1. On resume, create the native window and initialize its renderer when needed.
2. On a nonzero resize, update the renderer's drawable size. A zero-sized window
   is not configured or rendered.
3. Schedule redraws while the application has a live, drawable window, and use a
   short delayed retry when the presentation surface is temporarily unavailable.
4. For each successfully presented redraw, record monotonic frame timing after
   the renderer submits and presents the frame.
5. Recover from surface loss through renderer reconstruction/reconfiguration;
   treat transient acquisition failures as nonfatal and report unrecoverable
   renderer failures before exiting.
6. On a close request, stop the event loop and release window/GPU state cleanly.

## Evolution

Task 3 may observe runtime timing through an explicit diagnostics interface, but
must not make the overlay authoritative. When additional native targets or
platform services appear, move OS-specific policy behind `client/platform/`
without moving portable timing or client orchestration out of the runtime.
