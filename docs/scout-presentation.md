# Scout presentation foundation

## Lighting and material boundary (#463)

Warm cockpit/workshop fill at ship-local X=4/-3 metres contrasts with a cooler
exterior. Real Core bus loss removes warm fill and electrical emission while
retaining a cool visibility floor. This does not simulate emergency electrical
power, windows transmitting light, physical shadows or indirect illumination.
Physical screen atlas blank/restoration remains the existing explicit-power path.
Functional chassis/base colors and glass alpha are preserved; ordinary ship PBR
metallic/roughness/maps remain ignored. Do not use Cycles output to certify them.

Budget delta: zero new vertices, textures, buffers, draws or pipelines. The same
160-byte uniform carries the bus bit in its previously unused `center.w`.
Additional vertex arithmetic is bounded (two smoothstep zones, no light loop).
Measured Linux costs and unrun reference-hardware gates belong in the
[delivery report](../reports/issue-463/README.md), not a claimed FPS target.

## Comparable native and offline reference cameras

[scout-cameras.json](../scenarios/visual/scout-cameras.json) owns cockpit, Core,
Fabricator and aft-door positions/targets in runtime ship-local metres. Native
E2E maps them through the presented ship pose without changing player/aim state.
Blender uses the same inverse axis conversion as the existing Fabricator preview.
Both use 70-degree vertical FOV and 0.05-metre near clip. Use 1280x800 for corpus
comparisons and repeat at 1920x1080 for readability review; record actual DPI.
The shared power pairs are fixed-time renders, not normal-play interpolation QA.

```sh
python scripts/model_visual_tests.py capture --binary target/debug/salimon-client --fixtures scenarios/visual/scout-fixtures.json --captures artifacts/scout-lighting/candidates
python scripts/model_visual_tests.py compare --fixtures scenarios/visual/scout-fixtures.json --captures artifacts/scout-lighting/candidates --temporal-only --output artifacts/scout-lighting/repeat-check
blender --background --python-exit-code 1 --python models/assets/ships/salimon-scout/preview_lighting.py -- artifacts/scout-lighting/offline
```

On headless Linux use an X display and software Vulkan per the validation guide.
The scenario starts at real blackout, captures all four cameras, feeds the Core
through the production interaction, advances one tick, and captures them again.
Repeated frames must stay identical within the declared tolerance. Keep native
captures separate from Blender mood references and from approved image corpora.
Inspect screens, glass, walking-lane edges and dark housings; never promote a
candidate without human image/mask/tolerance review. M1 signoff remains #451/#477;
these cloud/reference fixtures do not grant that acceptance.
