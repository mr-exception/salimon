# Runtime Architecture

## Responsibility

The runtime is the native host and composition root. It translates `winit`
lifecycle/window events into a small set of renderer operations and owns timing
that future portable simulation code may consume.

```text
winit event loop
    -> salimon-client lifecycle + frame/update clocks + typed key mapping
        -> salimon-world camera/precision prototype + renderer-neutral snapshot
        -> salimon-diagnostics aggregation / RGBA view
        -> map snapshot -> salimon-renderer new / resize / render
            -> wgpu surface, camera-relative conversion, depth, and presentation
```

The dependency direction is one-way: the runtime depends on world, diagnostics,
and the renderer. Supporting crates never call into the runtime; the renderer
and diagnostics do not depend on world. Runtime mapping prevents portable world
types from acquiring `wgpu` or `winit` dependencies.

## Lifecycle flow

1. On resume, create the native window and initialize its renderer when needed.
2. On a nonzero resize, update the renderer's drawable size. A zero-sized window
   is not configured or rendered.
3. Schedule redraws while the application has a live, drawable window, and use a
   short delayed retry when the presentation surface is temporarily unavailable.
4. Before each drawable render attempt, advance the Task 4 portable prototype
   with a bounded monotonic delta, map its snapshot to renderer DTOs, and measure
   that real update work.
5. For each successfully presented redraw, record monotonic frame timing after
   the renderer submits and presents the frame, then combine it with renderer,
   update, and camera measurements for diagnostics.
6. Recover from surface loss through renderer reconstruction/reconfiguration;
   treat transient acquisition failures as nonfatal and report unrecoverable
   renderer failures before exiting.
7. On a close request, stop the event loop and release window/GPU state cleanly.

## Diagnostics flow

F3 initial key presses toggle diagnostics. P toggles the camera fixture, R
restarts it, and N selects and pauses the exact near-surface inspection view;
releases and key-repeat events are ignored. The runtime maps native keys to typed
commands and renderer/world measurements to diagnostics without sharing `wgpu`
or `winit` types. Diagnostics owns aggregation and the RGBA panel, while the
renderer owns only generic image composition. Camera metrics remain separate
from unavailable future player/ship state.

## Evolution

Diagnostics remains observational and non-authoritative. When additional native
targets or platform services appear, move OS-specific policy behind
`client/platform/` without moving portable timing or client orchestration out of
the runtime.
