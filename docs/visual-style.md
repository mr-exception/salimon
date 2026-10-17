# Salimon visual style

This is the authoritative art bible. Initial direction is **Option A: stylized
low-poly science-fiction**, approved in #273–#276. Low polygon count is a design
constraint, not a quality rating. Option B is a later stylized hard-surface pass:
more deliberate bevels and material detail, with the same metric interfaces.
Neither option changes gameplay, physics, sockets, pivots or asset identity.

## Form and shading

Use broad geometric hull planes, clipped corners and strong engine/wing/landing
support silhouettes. Reserve small shapes for close interaction surfaces. Hull,
tool and console planes have crisp normals; flat geological triangles expose
intentional facets. Smooth normals are allowed on functional spheres, glass and
rounded grips where faceting would confuse their role. Do not smooth rocks into
melted blobs or subdivide flat panels to increase apparent quality.

Bevels must affect a silhouette or catch a readable highlight at the intended
camera distance. Avoid subpixel rivets, repeated tiny vents and coplanar overlays.
A panel has one surface owner; keep the reviewed #244 internal joins. Use shared
meshes/materials for repeated details. Spend geometry on the seated console and
held tool before distant armor ornaments. Keep existing per-asset hard budgets;
style checks are advisory and do not replace spatial validation.

## Proposed palette

The following linear RGB defaults extend the existing authoring protocol. Hex
swatches are **sRGB display equivalents**, rounded after the standard linear to
sRGB transfer; do not paste those hex values as linear shader constants.
Palette/composition approval is pending review of #274 reference renders.

| Role | Linear RGB | Display hex | Use |
| --- | --- | --- | --- |
| Graphite | 0.09, 0.12, 0.16 | #55616F | Frames, floor, primary tool body |
| Slate | 0.24, 0.30, 0.37 | #8695A4 | Secondary panels, housings |
| Cyan | 0.14, 0.65, 0.72 | #69D3DD | Small active/status accents |
| Ochre | 0.83, 0.53, 0.18 | #EBC076 | Mechanical access and support details |

Opaque surfaces dominate. Resource colors retain gameplay identity rather than
inheriting ship cyan. Warm light interior panels preserve navigation contrast;
Core, monitor and tool status colors keep their existing roles. Never communicate
an interaction by hue alone: preserve silhouette, cursor and prompt affordances.
Use restrained emissive regions, no full emissive hulls or bloom-dependent cues.

## Materials, lighting and exceptions

Use base colors and existing material roles. The Scout runtime currently ignores
ordinary PBR metallic/roughness and textures; do not approve readability solely
from a Blender beauty render. Neutral CPU reference lighting uses fixed exposure,
Standard view transform and area key/fill. Native shader verification is separate.
No high-resolution textures are required. Prefer shared opaque materials;
textures need a consumer, UVs and encoded/decoded cost accounting.

Keep transparent, double-sided cockpit glazing, readable physical monitor UVs and
live artwork, lamps and light sources, and UI typography as functional exceptions.
Do not facet monitor quads or tint them with armor materials. Preserve clear lower
windows, door sweep and walking lanes. Collision surfaces stay authoritative and
independent of decorative faceting; terrain style cannot change physical ground.

## Reference views and acceptance

#274 owns reproducible editable reference scenes, cameras and comparable images;
its proposed compositions do not constitute owner sign-off. Use meter units,
Blender +Z up/+X forward and one `(x,z,-y)` runtime conversion.

| Subject | Reproducible view | Review criteria |
| --- | --- | --- |
| Scout exterior | Three-quarter from forward/starboard/above; repeat at twice framing scale | Distinct hull, wings, engines, four feet; major planes dominate; restrained accents |
| Interior | Seated eye toward wide monitor; aft view toward Core/door | Screen and Core readable, walking lanes/window sightlines clear; warm/cool hierarchy |
| Tool | Three-quarter grip-side near and twice framing scale | Muzzle, grip and status separate; grip socket unchanged |
| Deposit | Three-quarter neutral key | Large facets, stable resource color, no noisy micro-surfaces |
| Growing fragment | Sphere beside deposit at known meter radius | Spherical mining identity retained; readable facets without changing mass sizing |
| Terrain | Oblique patch with hill and depression | Coherent broad triangles, deterministic seed/configuration, no hidden ground displacement |

For each review: compare current/proposed with identical camera/light; report
triangles, materials, primitives, textures and consumer draws separately; verify
named/spatial contracts; inspect near/far silhouettes and normals. A screenshot
cannot establish collision validity or native GPU fidelity. Record approval by
owner/revision in the reference report; until then label palette/thresholds
proposed. Current screenshots are authored previews, not native game captures.

## Terrain and future assets

Preserve deterministic configurable planet definitions, streamed patch borders,
LOD continuity and physics ground. Future server-defined planets reuse that data
contract. Color/facets must remain coherent across LOD transitions. Caves,
tunnels and overhangs remain after Phase 0.

Future structures use broad modular masses and readable doors/attachment anchors;
characters use distinct head/torso/limb silhouettes and restrained equipment.
Their rigs, placement and interaction contracts need category extensions before
implementation. This bible does not add either asset class.

## Validation vocabulary

`visualBudget.style` describes Option A/B intent; it is not automatic style
approval. Its existing node, texture-memory and consumer draw caps and the five
`budgets` remain hard gates. Optional style profiles add measurable advisory
signals (density, normal alignment, reuse, LOD cost); shape, color and quality
remain review decisions. Purposeful exceptions require an asset-local reason.
See [manifest](../models/manifest.md) and [shared contracts](../models/contracts.md).
