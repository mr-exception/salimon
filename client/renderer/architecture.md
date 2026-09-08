# Renderer Architecture

## Responsibility

The renderer is a library boundary around native `wgpu` presentation. It owns
the surface, adapter/device/queue state, surface configuration, shader module,
render pipeline, command encoding, and frame presentation needed by the Task 2
bootstrap.

```text
runtime window + lifecycle
    -> Renderer::new / resize / render
        -> acquire surface texture
        -> encode clear pass + WGSL triangle
        -> submit command buffer
        -> present
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

## Evolution

Task 3 may add observable rendering measurements, and later tasks will introduce
camera/scene inputs. Preserve the library boundary as those capabilities grow:
domain modules produce presentation data, the runtime composes it, and the
renderer remains a GPU consumer rather than authoritative state owner.
