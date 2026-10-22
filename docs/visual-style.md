# Salimon visual style

This is the authoritative art bible. Initial direction is **Option A: stylized
low-poly science-fiction**, approved in #273–#276. Low polygon count is a design
constraint, not a quality rating. Option B is a later stylized hard-surface pass:
more deliberate bevels and material detail, with the same metric interfaces.
Neither option changes gameplay, physics, sockets, pivots or asset identity.

## Experience-level direction (2026-10-21)

The [2026-10-22 approved future starter Scout design](scout-and-modular-construction.md) adds an aft physical cargo bay, integrated living facilities and a 3D modular expansion language. This is a **future layout target**, not a claim the current Blender asset already incorporates it, and it does not silently relax this art bible's existing budgets or native-verification requirements.

The approved overarching mood is **grounded cinematic sci-fi**, using the existing low-poly geometry identity but unifying it with believable physical machinery, coherent warm Scout-versus-cold-world lighting, spatial sound and deliberate quiet, meaningful rare encounters, consistent on-board UI/animation/VFX, environmental clues and strong composition. **Asset style alone does not establish experience direction.** The first reference vertical slice is the Scout interior and all on-board interactions. See [experience-direction.md](experience-direction.md) for the authoritative experiential contract. Native renderer verification and performance remain mandatory; do not assume advanced materials supported merely from authored Blender output.

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

The production tool/deposit palette recipe and preserved shared-sphere policy
are documented in [resource Option A](../models/assets/resources/OPTION-A.md).

## Procedural body and terrain recipes (#278)

`client/renderer/src/planet_style.rs` owns the compact version-1 visual recipes;
`surface_textures.rs` deterministically generates their shared 512×256 linear RGB
maps and unchanged mip chains. No textures/mesh payloads cross a server boundary.
These are generic renderer materials. Runtime's `app/scene.rs` selects them for
bodies; unknown future bodies can select an existing preset. A future custom
recipe API must validate the version, finite [0,1] colors and seed and key the
texture cache by the complete visual recipe. Do not substitute its visual seed
for the authoritative terrain seed/definition or resource-generation identity.
No new persistence/network contract is introduced here. For future fictional planets, the [canonical multi-seed definition](canonical-planet-generation.md) gives the **visual** domain its own published seed, algorithm version and complete cache key, independent of the physical terrain, resources and fixed discoveries. Renderer LOD/visual quality may differ between clients; visual variation must never move physical geometry or determine multiplayer gameplay identity. All **real observed solid bodies** retain established physical terrain from the first authoritative real-scale publication (today's representative six-body fixtures are excluded); later scientific/material passes may be non-displacing only. Geology-derived physical forms belong to the single canonical world terrain query, not render-only displacement.

| Body / material | Visual seed | Linear dark → light palette |
| --- | ---: | --- |
| Mercury / stone | 91 | (0.25,0.23,0.21) → (0.55,0.51,0.45) |
| Venus / ochre | 734378 | (0.38,0.24,0.10) → (0.72,0.53,0.25) |
| Earth / oceanic | 1468665 | (0.03,0.12,0.25) → (0.08,0.30,0.46) |
| Earth / land color region | same | (0.18,0.27,0.18) → (0.43,0.48,0.28) |
| Moon / slate | 2202952 | (0.24,0.27,0.30) → (0.56,0.57,0.53) |
| Mars / rust | 2937239 | (0.32,0.13,0.07) → (0.66,0.34,0.17) |
| Sun / emissive | 3671526 | unchanged emissive recipe |

Five deliberate swatches per geological region replace fine albedo noise. The
Earth color-region split remains the seeded broad field at 0.50, independent of
physical ground; it implies no new ocean or biome simulation. Longitude/poles
sample the same continuous 3D field. Shared texture filtering/mips avoid hard
screen-distance switches. Existing 128 m detail is restrained to a 0.94–1.06
multiplier on solids, and Sun detail/emission remains unchanged.

Near terrain derives one outward face normal from derivatives of the already
morphed geometric triangle. There is no mesh expansion, additional vertex data,
or displacement. Full face shading below 80 m fades continuously to sampled
normals by 240 m; startup/outer-cap geometry blends also suppress facets. Hills,
craters and their surrounding terrain use the same treatment. Deposits keep
actual authored silhouettes; distant analytic planets intentionally retain
continuous spherical silhouettes and rationalized depth. Normals-only styling
does not add rocky silhouette geometry or change walking/landing contact.

Preserve the existing stable patch keys, shared-edge samples and 2:1 fans,
4,096–6,144 m outer morph, 0.25 s activation blend, conservative sphere envelope,
and upload/memory limits in [terrain validation](terrain-validation.md). No
randomness is generated per frame and no LOD triangle is moved for art direction.
The [GPU comparison recipe and report](../reports/issue-278/README.md) describes
matched near/far/orbit scenes, fade-boundary continuity, unchanged coverage and
costs. Native M1 gameplay/visual approval remains #279/#229.

Scout light zoning and reproducible native/offline cameras (#463) are defined in
[Scout presentation](scout-presentation.md); CPU/Blender results do not grant native visual signoff.
