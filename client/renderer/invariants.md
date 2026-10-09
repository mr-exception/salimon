# Renderer Invariants

1. The surface is configured and rendered only with a nonzero physical width and
   height.
2. The active surface configuration matches the latest accepted drawable size.
3. `wgpu` device, queue, surface, pipeline, shader, and presentation resources
   remain owned by the renderer boundary.
4. One successful `render` call submits and presents one validation-scene frame
   and returns measurements for that presented work.
5. Surface acquisition outcomes are returned to the runtime; the renderer does
   not exit the application or own lifecycle recovery or retry policy.
6. Rendering never advances simulation, mutates authoritative world/game state,
   or consumes raw gameplay-domain internals.
7. `CameraFrame`, `SceneInstance`, `SphereInstance`, `ShipMeshInstance`, and `PointLight` are renderer DTOs, not authoritative world
   state. The runtime maps domain snapshots into them while preserving absolute
   `f64` positions; it does not rebase those positions.
8. The renderer subtracts camera position from instance centers and camera
   target in `f64` before converting relative values to GPU-facing `f32`.
9. View/projection construction and depth policy are renderer-owned. The scene
   path uses a right-handed infinite reverse-Z projection and matching depth
   target/comparison state.
10. Cuboids, analytic spheres, generic materials, and point lighting exercise
    presentation only, with no celestial identity, landing rules, or world
    dependency. Sphere surface altitude is subtracted in `f64` before GPU
    conversion; its fragment depth uses the rationalized ray-intersection root.
11. The renderer uses low-level `wgpu` directly and must not introduce a full
    game engine.
12. Overlay inputs are validated, borrowed RGBA images with typed screen-space
    or absolute-world placement. The renderer may cache, uniformly fit, and composite them but
    never owns diagnostics aggregation, action text, expiry/toggle policy, or
    gameplay data.
13. GPU timestamps are requested only when the selected adapter supports them;
    readback must not block the presentation loop. Unsupported or pending data
    is reported explicitly.
14. Scene object counts describe instances retained by conservative frustum
    culling, and scene draw counts exclude overlay presentation. Total draw calls
    include each diagnostics/action/reticle overlay when it is visible.
15. Texture LOD varies continuously with projected footprint. Local detail uses
    a body-relative origin wrapped in CPU `f64`; no universe-scale `f32` texture
    coordinates, atmosphere, clouds, shadows, or post-effect dependency is used.
16. The checked-in ship GLB is validated and loaded by the renderer without a
    dependency on ship or character behavior. Pose and door state cross only as
    generic presentation data and remain camera-relative on the GPU.
17. Cockpit glass is the ship's only blended material. It renders after opaque
    ship geometry with reverse-Z depth testing and no depth writes, shadows, or
    post effects, preserving exterior visibility from either side.
18. Ship vertices use the authored scale baked into the checked-in GLB; the open
    door offset is 4.50 m and no compensating runtime mesh scale is applied.

19. Cockpit instruments consume only renderer-facing speed/power, Core-energy,
    and optional nearby-body DTO values. The center speed, port Core, and
    starboard proximity/radial panels use one cached atlas in the existing opaque
    ship draw, never screen-space overlays or separate scene cameras. Both side
    panels retain common thruster power. Only changed displayed values rebuild
    and upload their atlas panels; no target renders an explicit out-of-range state.
    Monitor UVs use top-left (0, 0); each named surface is exactly two triangles.

20. Center-placed overlays use the current drawable dimensions, place their
    midpoint at NDC (0, 0), and ignore scene occlusion. The renderer consumes
    image/revision/placement only, never equipment or targeting state.

21. Held mining-tool geometry is the checked-in authored GLB, grip-centered and
    baked with identity node transforms. Camera-local presentation preserves
    orientation through look/gravity changes and never supplies a gameplay ray.
    Its embedded surface atlas/UVs and metallic/roughness factors reach the
    fragment shader; unsupported texture/material contracts fail asset loading.
    Active feedback changes only status regions; normal reverse-Z depth applies.

22. World prompt anchors are rebased in `f64` and projected with the scene camera.
    Invalid, behind-camera, clipped or non-fitting labels are hidden, never
    clamped to screen edges. The separate overlay pass samples stored reverse-Z
    scene depth at the anchor to hide an entire occluded prompt. Its conservative
    radius avoids target self-occlusion and never changes gameplay eligibility.

23. Equipment toolbar contents/selection cross as renderer-neutral DTOs. Renderer
    only rasterizes/composites them; five slots remain visible even when all are
    empty. Absence hides the toolbar. Resize/DPI recomputes readable fitting, and
    bottom-center global/transient messages stack above its reserved band.
    Mining icon placement uses visible alpha bounds so asymmetric canvas padding
    cannot shift artwork from the slot center; glyphs/borders remain legible.
