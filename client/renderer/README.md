# Renderer

`salimon-renderer` is the custom `wgpu` 30.0.1 rendering library introduced in
Task 2, instrumented in Task 3, and extended with the Task 4 large-scale camera
prototype. Task 6 adds screen-bounded analytic spheres, original mipmapped
surface textures, local detail, and unshadowed point lighting. Runtime maps the
six bodies to generic sphere/material DTOs and the three precision markers to
separate cuboids. The renderer knows no celestial identities or landing rules.
Task 11 adds a renderer-owned GLB loader and a generic `ShipMeshInstance` pose DTO.
Task 9 splits the asset's cockpit glass into a second, depth-tested alpha-blended
draw so exterior geometry remains visible from inside; the renderer still has no
dependency on character or ship behavior crates.
Task 10 consumes the per-axis-rescaled checked-in GLB without a runtime scale
transform and keeps the 2× horizontal open-door presentation offset at 4.50 m.
The ship's `Interior` asset group receives a restrained warm ambient fill;
material emission stays independent of base color so lamps, displays, the core,
and thrusters remain self-lit. Lighting is evaluated per vertex and still uses
only the opaque and glass draws, with no dynamic shadows or light loops.
See [sphere-rendering.md](sphere-rendering.md) for the precision/LOD technique,
material source, limitations, and future terrain path.

The public integration surface is intentionally narrow:

- `Renderer::new` initializes window-bound GPU state.
- `Renderer::resize` updates the color surface and matching depth target.
- `Renderer::render` accepts a borrowed `SceneFrame`, an optional borrowed
  `OverlayImage`, and the platform presentation callback.
- `CameraFrame`, `SceneInstance`, `SphereInstance`, `ShipMeshInstance`, and `PointLight` carry renderer-facing snapshots without a
  dependency on world, character, or ship crates.

## Large-scale coordinate and depth strategy

Camera and instance centers cross the public boundary as finite `f64` metres.
Every instance center is subtracted from the camera position on the CPU while
both values are still `f64`; only that camera-relative delta is converted to
`f32` for the GPU. Camera target minus camera position follows the same rule.
The GPU therefore never subtracts two rounded, large absolute `f32` positions.

The camera uses a right-handed, infinite reverse-Z perspective projection in
WebGPU's zero-to-one depth range. A `Depth32Float` attachment is cleared to
`0.0`, scene fragments compare with `Greater`, and the near plane maps to `1.0`.
Depth approaches zero with distance and has no finite far plane. The caller
still owns the near-plane choice; `0.05 m` is the Task 4 validation value and
should only be reduced when close geometry requires it. Spheres write analytic
surface depth into that same attachment.

The renderer's adjacent-depth-value test measures the view-space separation
represented by `Depth32Float` with that near plane:

| View distance | Adjacent reverse-Z distance step |
| ---: | ---: |
| `12 m` | about `0.00000134 m` |
| `120 Mm` | about `7.99 m` |

These values quantify depth-buffer resolution, not whole-object precision.
Large geometry can lose more precision while its vertices are reconstructed,
as described below.

Representative IEEE-754 spacing shows the useful envelope and the remaining
limitation:

| Magnitude | `f64` absolute spacing | `f32` camera-relative spacing |
| ---: | ---: | ---: |
| `1 km` | much less than `1 nm` | about `0.061 mm` |
| `1 Mm` | about `0.116 nm` | `0.0625 m` |
| `1 Gm` | about `0.00012 mm` | `64 m` |
| `1 Tm` | about `0.122 mm` | `65.536 km` |

Near-camera detail keeps local `f32` precision even when the absolute camera
origin is `1 Tm` from zero; the translation-invariance unit test covers that
case. Far-away instance centers still inherit `f32` spacing based on their
camera distance. Later LOD or planet rendering must avoid expecting metre-scale
mesh detail to survive at gigametre/terametre relative distances. The established
path does not add logarithmic depth, split coordinates in WGSL, or multiple depth
passes. Task 6 additionally retains the sphere's camera-to-surface distance in
CPU `f64`, avoiding the cancellation of large GPU center/radius operands when
solving the near surface intersection.

## Ownership and metrics

The renderer owns GPU resources, surface/depth configuration, shader and
pipeline state, command encoding, generic overlay composition, timestamp
readback, allocator reporting, and presentation. It does not own the native
event loop, frame clock, lifecycle policy, diagnostics content, or authoritative
game/world state.

Cuboids use conservative bounding-sphere frustum culling; analytic spheres use
conservative projected bounds. There is no occlusion culling. `RenderStats`
reports the combined retained instance count and one draw for each nonempty
geometry class, except that a visible ship uses one opaque draw and one cockpit
glass draw, plus the overlay draw when visible. GPU timestamps remain capability-gated and use
the existing non-blocking three-slot readback ring.

Run the library through `cargo run --locked -p salimon-client`. See
[README.ai.md](README.ai.md), [architecture.md](architecture.md), and
[invariants.md](invariants.md) before changing its contract.

The cockpit includes three physical instrument screens: a central metric speed
readout, a port Core stored/capacity panel, and a starboard nearby-body panel
with surface distance plus approaching/receding/zero radial speed. Both side
panels retain the synchronized thruster percentage. The runtime supplies
`CockpitInstruments` on `ShipMeshInstance` every gameplay frame, including while
the player walks around. A cached 512 × 768 sRGB atlas supplies crisp self-lit
text and segmented bars in the existing opaque ship draw. Only display changes
trigger rasterization/upload; the opaque/glass submission remains two draws.
The renderer retains no ship control or simulation dependency.
