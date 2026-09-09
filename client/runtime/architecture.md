# Runtime Architecture

## Responsibility

The runtime is the native host and composition root. It translates `winit`
lifecycle/window events into a small set of renderer operations and owns timing
that future portable simulation code may consume.

```text
winit event loop
    -> salimon-client lifecycle + frame clock + diagnostics composition
        -> salimon-diagnostics aggregation / RGBA view
        -> salimon-renderer new / resize / render
            -> wgpu surface and GPU work
```

The dependency direction is one-way: the runtime depends on diagnostics and the
renderer. Neither supporting crate calls into the runtime, and no current crate
depends on future world, character, or ship modules.

## Lifecycle flow

1. On resume, create the native window and initialize its renderer when needed.
2. On a nonzero resize, update the renderer's drawable size. A zero-sized window
   is not configured or rendered.
3. Schedule redraws while the application has a live, drawable window, and use a
   short delayed retry when the presentation surface is temporarily unavailable.
4. For each successfully presented redraw, record monotonic frame timing after
   the renderer submits and presents the frame, then combine it with renderer
   measurements for diagnostics.
5. Recover from surface loss through renderer reconstruction/reconfiguration;
   treat transient acquisition failures as nonfatal and report unrecoverable
   renderer failures before exiting.
6. On a close request, stop the event loop and release window/GPU state cleanly.

## Diagnostics flow

F3 initial key presses toggle the diagnostics crate; releases and key-repeat
events are ignored. The runtime maps renderer measurements into diagnostics
observations without sharing `wgpu` or `winit` types. The diagnostics crate owns
aggregation and the RGBA panel, while the renderer owns only generic image
composition. Future world/ship tasks provide optional typed domain metrics at
this composition point.

## Evolution

Diagnostics remains observational and non-authoritative. When additional native
targets or platform services appear, move OS-specific policy behind
`client/platform/` without moving portable timing or client orchestration out of
the runtime.
