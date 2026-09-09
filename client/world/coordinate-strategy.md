# Phase 0 Coordinate Strategy

## Strategy

World positions are Cartesian `f64` meters. The Task 4 prototype places its
surface anchor at `(1.0e12, -7.5e11, 2.5e11)` meters and moves the camera along
the anchor's positive Z axis. World exposes each camera and primitive center as
an absolute `f64` value. The runtime maps those values into renderer DTOs without
rebasing. During frame preparation, the renderer subtracts the camera position
from each primitive center and from the camera target while the operands remain
`f64`; only the relative vectors are converted to `f32` for presentation.
Primitive dimensions stay in meters.

The camera follows an eight-second smooth logarithmic approach from 120,000 km
to 12 m, dwells for two seconds, retreats over eight seconds, and dwells at the
far endpoint for two seconds. Logarithmic distance distributes screen time
across orders of magnitude; smoothstep easing removes endpoint velocity jumps.
The camera uses a 60-degree vertical field of view and a physical near plane of
5 cm.

Clip-space conventions and depth resources do not belong to world state. The
renderer produces the camera-relative values and owns the infinite-far,
reverse-Z projection and depth comparison used to cover both scale regimes.

## Measured representation limits

`PrecisionReport::for_prototype` measures adjacent representable values with the
same scalar types used by the implementation. Unit tests lock these values:

| Measurement | Spacing |
| --- | ---: |
| `f64` at the largest `1.0e12 m` anchor component | `0.0001220703125 m` |
| camera-relative `f32` at the `120,000,000 m` far endpoint | `8 m` |
| camera-relative `f32` at the `12 m` near endpoint | `0.00000095367431640625 m` |
| near-view giant-proxy center near `6,000,012 m` in `f32` | `0.5 m` |
| near-view giant-proxy relative-center rounding-error bound | `0.25 m` |

A dedicated test shows that a 1 m offset at `1e12 m` survives subtracting in
`f64` before conversion, while converting both global positions to `f32` first
collapses that offset to zero. The prototype budget therefore permits less than
1 mm global anchor spacing, no more than 8 m scalar spacing at the far endpoint,
and less than 0.01 mm scalar spacing at the near endpoint.

The giant blue surface proxy exposes a separate GPU reconstruction limit. Its
near face is formed in the vertex shader by combining a camera-relative center
and half-extent near `6 Mm`. `f32` values at that magnitude have `0.5 m` spacing,
so the reconstructed face can carry up to about `0.25 m` of rounding error. This
does not erase the local precision of the small nearby markers: their centers
remain near the camera and use the much finer spacing represented by the near
endpoint row above.

The renderer separately tests `Depth32Float` quantization under the `0.05 m`
infinite reverse-Z projection. Adjacent depth values represent about
`0.00000134 m` of view-space separation at `12 m` and about `7.99 m` at
`120,000,000 m`. The far endpoint is therefore intentionally an order-of-scale
validation view; neither the coordinate conversion nor depth buffer promises
meter detail there.

## Limitations

- Far-space `f32` relative positions remain coarse. Renderer-side camera
  rebasing protects nearby geometry; it does not make distant geometry locally
  precise.
- Camera-relative centers do not solve precision inside a single enormous mesh;
  the giant surface proxy deliberately demonstrates that remaining limit.
- Uniform world scaling alone would not improve physical precision, so this
  prototype keeps meters explicit rather than hiding precision loss in units.
- The six cuboids are precision/depth references, not canonical planets,
  terrain, scale, or gameplay content. Task 5 supplies real static world data.
- Large meshes spanning multiple local regions will eventually need chunk-local
  anchors or high/low GPU position encoding. Renderer-side per-instance CPU
  rebasing is enough for the small Phase 0 validation set, not a shipping scene.
- Reversed-Z improves useful depth distribution and removes a finite far clip;
  it cannot distinguish exactly coplanar surfaces or compensate for poorly
  chosen near geometry. Visual GPU limitations must be recorded by the runtime
  and renderer smoke test on the reference MacBook Air M1.
