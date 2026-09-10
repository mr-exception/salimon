# Renderer Architecture

## Responsibility

The renderer is a library boundary around native `wgpu` presentation. It owns
the surface, adapter/device/queue state, surface configuration, shader modules,
render pipelines, command encoding, camera-relative GPU conversion,
renderer-owned projection and depth, generic overlay composition, and frame
presentation for the Task 5 proxy scene and Task 3 instrumentation.

```text
salimon-world absolute f64 snapshot
    -> runtime maps domain types to renderer DTOs (no rebasing)
    -> Renderer::render(SceneFrame, optional RGBA overlay)
        -> validate absolute f64 camera and generic cuboid inputs
        -> subtract camera in f64, then cast relative values to f32
        -> conservatively frustum-cull cuboid bounds
        -> build renderer-owned view + infinite reverse-Z projection
        -> acquire surface texture
        -> encode depth-tested instanced cuboids + optional overlay quad
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
the current surface texture, prepares and uploads the borrowed scene, records
the validation pass, submits it to the queue, and presents it once. The runtime
owns mapping from portable world structs into `CameraFrame` and `SceneInstance`,
but preserves their absolute `f64` positions. The renderer subtracts camera
position from instance centers and the camera target in `f64`, converts the
relative results to `f32`, and builds the right-handed infinite reverse-Z
projection and depth state.

The instanced cuboids are renderer-facing presentation content. For Task 5 the
runtime emits six radius-scaled colored body proxies followed by three precision
markers. They arrive through the generic `SceneFrame` contract and do not give
the renderer knowledge or ownership of celestial identities, landing volumes,
the world catalog, or future simulation. Task 6 owns the explicit sphere and
Sun-lighting path.

The scene uses a conservative bounding-sphere frustum test before upload.
Objects retained by that test define the visible/rendered counters; Task 5 does
not add occlusion culling.

## Precision boundary

Camera-relative conversion preserves local detail around a large absolute
origin, but it cannot make every large GPU operand precise. The radius-scaled
Earth proxy reconstructs its near face in WGSL from a relative center and
half-extent near `6 Mm`. `f32` spacing at that magnitude is `0.5 m`, which permits
up to about `0.25 m` of rounding error at the face. This known proxy artifact does
not reduce the precision of the small nearby markers: their centers are represented
at their much smaller camera-relative magnitudes.

## Diagnostics contract

Diagnostics supplies already-rasterized RGBA pixels and a revision number. The
renderer validates and caches the image, uploads it only when the revision or
size changes, uniformly scales oversized panels to stay inside 16-pixel drawable
margins, and composites them with alpha blending. It never formats metrics or
decides when the overlay is visible.

The combined render pass uses capability-gated timestamp queries. A three-slot
readback ring is polled without waiting, so GPU timing does not synchronously
stall presentation. Scene object/draw counts exclude the engineering overlay;
the total draw count includes its one compositing draw. Optional allocator
reports expose GPU allocated/reserved bytes without claiming process memory.

## Evolution

Task 6 may replace the body-proxy cuboids with richer renderer-facing sphere data.
Preserve the dependency direction as those capabilities grow: domain
modules produce absolute snapshots, the runtime maps them into presentation
DTOs without doing precision conversion, and the renderer remains a GPU
consumer rather than an authoritative state owner.
