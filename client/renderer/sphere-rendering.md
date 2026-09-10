# Scalable sphere presentation

Task 6 uses analytic spheres. Each retained sphere draws a six-vertex screen
rectangle; fragments solve a ray/sphere intersection, sample body-local material,
and write the actual surface's reverse-Z depth. This produces a continuous curved
silhouette from distant space to the 12 m inspection point without tessellation
changes, mesh cracks, or per-frame mesh uploads. Small precision-marker cuboids
share the same depth attachment and can occlude or be occluded by the sphere.

## Precision

`SphereInstance` carries absolute `f64` center/radius. On the CPU the renderer
subtracts the camera, calculates center distance `d`, and retains surface altitude
`a = d - R` in `f64` before conversion. WGSL intersects a normalized camera ray
using the perpendicular cross product and a rationalized quadratic root:

```text
q = d / R
discriminant = 1 - q² × |ray × center_direction|²
b = d × dot(ray, center_direction)
t = a × (d + R) / (b + R × sqrt(discriminant))
depth = near / (t × dot(ray, camera_forward))
```

The near root never subtracts million-meter GPU values to recover meter-scale
altitude. The CPU-side `f64` world spacing (about 0.122 mm at the fixture's 1 Tm
anchor) remains the ultimate absolute-position limit. Unit regressions cover all
six compressed radii at 0.125 m, 1.875 m, 12.125 m, landing range `0.15R`, and
120 Mm; oblique rays; translation to the large origin; inside-volume exit; and
near-plane crossings. `Depth32Float` clear 0 / Greater / infinite far is shared
with other scene geometry. No logarithmic depth or extra depth pass is used.

A camera inside a sphere sees the positive exit root. This supports viewing the
visual-only Sun from inside; renderer geometry never implies collision. A camera
exactly on a surface clips its zero-distance near root against the physical near
plane, as expected. Tangent/silhouette pixels remain subject to normal `f32`
rounding and pixel sampling; this is not a claim of infinite precision.

## Materials, lighting, and LOD

`surface_textures.rs` is the editable source of six original 512×256 linear-color
albedo maps: stone, ochre, oceanic, slate, rust, and emissive. Three-dimensional
seeded noise sampled over a sphere makes fictional features with matching
longitude/pole topology. There is no recognizable planetary imagery. A seventh
periodic grayscale map supplies subtle 128 m triplanar surface detail.

All ten mip levels down to 1×1 are generated once on the CPU in linear color and
uploaded as one texture array (about 4.67 MiB). Per-pixel projected footprint,
grazing angle, and longitude distortion select explicit trilinear texture LOD.
This avoids longitude-wrap derivative artifacts and fades subpixel detail
continuously during retreat. Near detail uses body-local camera coordinates
wrapped in `f64` before upload plus the small camera-relative hit point, so its
phase stays fixed during motion and rebasing. No texture streaming or disk I/O
occurs during approach.

Runtime maps the canonical Sun center into a generic `PointLight`. Solid surfaces
use unshadowed diffuse illumination plus a small constant ambient term; emissive
surfaces bypass diffuse lighting. There is no light-distance attenuation in this
material showcase. Atmospheres, clouds, shadows, and post effects are absent.

The CPU culls conservative screen bounds and clips them to the viewport. Near
plane crossings/camera-inside cases use viewport bounds. All retained spheres
share one instanced draw and material array; cuboid markers add at most one draw,
and the optional overlay adds one. Object counts mean retained bounds, not exact
occlusion visibility. Cost scales with covered pixels rather than sphere radius.
Fragment depth and discarded silhouette fragments can limit early-depth
optimizations; profile overlapping large bodies before increasing scene size.
The later Task 11 benchmark must establish the M1 fixed-1080p performance target.

## Future higher-detail terrain

The absolute sphere DTO, renderer-owned material style, and point-light DTO are
presentation inputs; the renderer has no dependency on world/ship/character.
The body-local analytic sphere provides the far representation and nominal
surface/depth reference for future terrain.

A terrain task can add renderer-neutral patch DTOs with an absolute `f64` anchor,
small body-local vertex offsets, material handle, and conservative bounds.
Runtime would map terrain snapshots into those DTOs; renderer would subtract the
camera from each patch anchor in `f64`, then upload the small local geometry.
Use projected geometric error to select patches, explicit parent/child coverage
to replace the matching analytic surface region, and shared reverse-Z depth.
Patch stitching, displacement, transition coverage, and collision remain future
work. Do not overlay coplanar patches over the whole sphere or reconstruct
meter-scale terrain by adding an `f32` radius to a large `f32` center.

The existing sphere path deliberately does not claim terrain/elevation support.
It establishes stable bounds, surface depth, body-local material coordinates,
and an isolated presentation contract on which that work can build.
