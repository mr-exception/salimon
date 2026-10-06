# Diagnostics Architecture

## Responsibility

Diagnostics is a client-side observation boundary between measurement producers
and renderer composition:

```text
runtime timing ------\
renderer counters ----> FrameSample + DomainMetrics -> Diagnostics
world/domain state --/                               -> OverlayImage
                                                           |
                                                           v
                                                  renderer composition
```

The runtime owns input, lifecycle, redraw scheduling, and measurement cadence.
The renderer owns GPU timestamps, allocator reports, render counters, texture
upload, and blending. `salimon-world` owns the current camera prototype and
static six-body catalog; character and ship own current player/ship state, while
world and physics own resource and physical-object rules. Diagnostics borrows
those values for one call and stores only a formatted snapshot; it never
receives raw platform events or GPU/domain resources.

## Data flow

1. A successfully presented frame produces a `FrameSample` and borrowed
   `DomainMetrics`.
2. Nonzero intervals enter a 120-element rolling window. A zero first/resumed
   interval can update current counters but cannot distort FPS or percentiles.
3. At the first sample and every 250 milliseconds of valid presented-frame time,
   diagnostics computes averages and nearest-rank p95 frame time, copies the
   small domain snapshot, and formats the engineering text.
4. While visible, the text is rasterized with the embedded font into a
   translucent RGBA panel. `OverlayImage::revision` changes with each pixel
   rebuild so the renderer can skip redundant uploads.
5. Hiding the overlay suppresses `overlay` and `overlay_text` without discarding
   measurement history. `reset_frame_window` clears lifecycle-separated frame
   statistics while retaining the last known domain snapshot.

## Availability model

GPU timing is a three-state value because timestamp queries are optional and
readback is asynchronous. Memory and domain fields are optional because platform
support varies. Runtime supplies six catalog camera-to-nominal-surface
observations, from which diagnostics displays the closest. Engineering camera
telemetry remains named separately from live player and ship positions so an
engineering camera is never presented as gameplay state.
The formatted panel states gaps explicitly instead of treating zero as missing
or fabricating sample data.

## Performance model

Frame insertion is bounded and allocation-free after the deque reaches its
capacity. Formatting and domain-name copies happen at overlay refresh cadence,
not on every frame. Pixel rasterization happens only while visible or after an
explicit visible-state change. The renderer receives borrowed pixels, so this
crate performs no GPU allocation or synchronization.
