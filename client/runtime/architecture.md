# Runtime Architecture

## Responsibility

The runtime is the native host and composition root. It translates `winit`
lifecycle/window events into a small set of renderer operations and owns timing
that future portable simulation code may consume.

```text
winit event loop
    -> salimon-client lifecycle + frame/update clocks + typed key mapping
        -> salimon-character movement / camera snapshot
        -> salimon-ship pose / interaction snapshot
        -> salimon-world static Solar System + camera/precision snapshot
        -> salimon-diagnostics aggregation / RGBA view
        -> runtime contextual action-bar state / RGBA view
        -> map snapshot -> salimon-renderer new / resize / render
            -> wgpu surface, camera-relative conversion, depth, and presentation
```

The dependency direction is one-way: the runtime depends on character, ship,
world, diagnostics, and renderer. Supporting crates never call into runtime;
renderer and diagnostics do not depend on behavior domains. Runtime mapping
prevents portable types from acquiring `wgpu` or `winit` dependencies.

## Lifecycle flow

1. On resume, create the native window and initialize its renderer when needed.
2. On a nonzero resize, update the renderer's drawable size. A zero-sized window
   is not configured or rendered.
3. Schedule redraws while the application has a live, drawable window, and use a
   short delayed retry when the presentation surface is temporarily unavailable.
4. Before each drawable render attempt, advance portable character/ship/camera
   state with a bounded monotonic delta. Map the six catalog bodies to `f64` sphere/material
   DTOs, map Sun lighting, preserve separate marker cuboids, calculate camera-to-surface
   distances, and measure that real update work.
5. For each successfully presented redraw, record monotonic frame timing after
   the renderer submits and presents the frame, then combine it with renderer,
   update, and camera measurements for diagnostics.
6. Recover from surface loss through renderer reconstruction/reconfiguration;
   treat transient acquisition failures as nonfatal and report unrecoverable
   renderer failures before exiting.
7. On a close request, stop the event loop and release window/GPU state cleanly.

## Diagnostics flow

F2 toggles gameplay/precision-tour view and F3 toggles diagnostics. P toggles the camera fixture, R
restarts it, and N selects and pauses the exact near-surface inspection view;
releases and key-repeat events are ignored. The runtime maps native keys to typed
commands; 1–6 restart inspection of Sun, Mercury, Venus, Earth, Moon, and Mars.
It maps renderer/world measurements to diagnostics without sharing `wgpu` or
`winit` types. All six catalog names and nonnegative camera-to-nominal-surface
observations flow through `BodyDistance`; diagnostics selects the closest for its
single nearby-body row. Diagnostics owns aggregation and the RGBA panel, while
the renderer owns only generic image composition. Gameplay view supplies live
player/ship diagnostics; precision-tour view supplies its camera metrics.

## Contextual action-bar flow

The runtime maps typed ship messages and the aimed cockpit interaction into one
bottom-centered action bar for the normal gameplay view. State-derived actions
remain visible only while applicable. Immediate blocked-door feedback overrides
the current action for three seconds and then expires without changing ship
state. The renderer receives a borrowed RGBA image and placement only, allowing
the action bar and optional diagnostics panel to be composited independently.

## Evolution

Diagnostics remains observational and non-authoritative. When additional native
targets or platform services appear, move OS-specific policy behind
`client/platform/` without moving portable timing or client orchestration out of
the runtime.
