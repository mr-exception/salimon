# Renderer Architecture

## Responsibility

The renderer is a library boundary around native `wgpu` presentation. It owns
the surface, adapter/device/queue state, surface configuration, shader modules,
render pipelines, command encoding, generic overlay composition, and frame
presentation needed by the Task 2 bootstrap and Task 3 instrumentation.

```text
runtime window + lifecycle
    -> Renderer::new / resize / render(optional RGBA overlay)
        -> acquire surface texture
        -> encode clear pass + WGSL triangle + optional overlay quad
        -> resolve optional timestamp queries asynchronously
        -> submit command buffer
        -> present
        -> return renderer measurements
```

The initialization seam may accept a native window/surface handle, but window
creation, event dispatch, redraw policy, frame timing, and error policy remain in
the runtime. Surface acquisition outcomes flow back to the runtime for delayed
retry, recovery, or shutdown.

## Resize and frame flow

Only nonzero physical sizes configure the surface. A successful render acquires
the current surface texture, records the bootstrap pass, submits it to the queue,
and presents it once. The current triangle is renderer-owned validation content;
future scene data must arrive through explicit presentation contracts rather
than by giving the renderer ownership of world simulation.

## Diagnostics contract

Diagnostics supplies already-rasterized RGBA pixels and a revision number. The
renderer validates and caches the image, uploads it only when the revision or
size changes, and composites it with alpha blending. It never formats metrics or
decides when the overlay is visible.

The combined render pass uses capability-gated timestamp queries. A three-slot
readback ring is polled without waiting, so GPU timing does not synchronously
stall presentation. Scene object/draw counts exclude the engineering overlay;
the total draw count includes its one compositing draw. Optional allocator
reports expose GPU allocated/reserved bytes without claiming process memory.

## Evolution

Later tasks will introduce camera/scene inputs and replace bootstrap count
constants with scene-derived values. Preserve the library boundary as those
capabilities grow:
domain modules produce presentation data, the runtime composes it, and the
renderer remains a GPU consumer rather than authoritative state owner.
