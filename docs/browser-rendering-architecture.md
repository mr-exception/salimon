# Browser rendering architecture and evaluation
Approved future direction 2026-11-26; no renderer migration authorized until comparative evidence. Current 2D SVG adapter caches illustrations (64 texels/m, terrain 32); React/TypeScript/Vite and Three.js/R3F panoramic remain. Read [client guide](../client/README.md), [top-down](top-down-art-direction.md), [production](art-production-guide.md).

## Boundary and evaluation
Compare current SVG and candidate PixiJS with equivalent Scout, populated terrain, modular animated character, multiple lights and particles; record production Chromium/WebGL backend, software vs physical GPU, viewport/DPR, seed, build, frame time distribution, memory estimates, asset bytes, draw/object counts and visual parity. Evaluate batching/atlases, masks, transparency, lighting, lifecycle/disposal, React integration, input, cloud tests, deployment; retain SVG if no meaningful benefit or unacceptable regression. If passed adopt PixiJS only for 2D presentation; React HUD and Three.js navigation unchanged. No simulation/inventory/crafting/collision/save rewrite. Preserve single 60Hz authority, local coordinate frames, stable draw/interaction IDs, camera transforms, mouse context/approach and ship-up orientation. Rendering snapshots read-only; no second simulation clock. Context loss recovery or clear compatibility failure.
Layer order back-to-front: base terrain, surface detail, contact/environment shadows, entities/character, foreground/occlusion, lighting composition, weather/particles, contextual markers, React HUD/dialogs. Masks/passes may vary; render/input transforms agree even during smoothing/shake; alpha never changes collision.
Initial engineering targets (not guarantees): 60 FPS normal reference hardware, <=15 MB compressed initial playable assets, 128 MB active 2D texture memory, 2048² default atlases, 128 px/m source. Measure decoded textures including mipmaps/overhead, cull offscreen, deterministic chunks, lazy load/unload, release GPU resources, hidden tab presentation suspension without changing domain pause rules. Desktop Chromium primary, Firefox/Safari supported targets, WebGL2 baseline, WebGPU optional future, mobile deferred; high DPI resolution cap. Low/Medium(default)/High/Custom: contact vs dynamic shadows, particle density 25/60/100%, environmental effects 50/75/100%, reduced/standard/full light, world render scale 75/100/100%; validate defaults. Essential cues unchanged across quality. 
Diagnostics development-only: footprints, anchors, chunks, layers, IDs, timings, texture memory estimates, audio sources and read-only state. No exposed setters. Actual browser evidence and exact revision, no native gate.

## Reproduce the renderer comparison (#939)

From `client/`, run `npm ci`, then
`CHROMIUM_PATH=/path/to/chromium npm run benchmark:renderer`. The opt-in
Playwright configuration builds and serves a separate fixture at port 4375. The
ordinary Vite entry and build graph contain the existing SVG production adapter.
The fixture uses a fixed seed, 960×600 CSS pixels at DPR 1, two trials per
renderer and density, 20 warm-up frames, and 90 real animation-frame intervals
per trial. It captures the same logical frame for visual comparison and repeats
each renderer after teardown and remount. Raw samples, screenshots, scene
comparisons and exact tested revisions are linked from the delivering PR.

Both adapters consume the same ordered read-only sprite list and pixel sources.
The Scout scene reuses production illustrations. The surface scene uses
repeatable terrain tiles and seeded populated details. Both animate a modular
character, translucent radial lights and drifting particles. The SVG fixture
uses cached raster images and a retained DOM. PixiJS uses WebGL2 sprites,
packed texture pages, a rectangular mask and explicit texture disposal.
Both use the same inverse pointer mapping and domain-style logical hit order.
Browser tests cover masked paint, accepted and rejected pointer coordinates,
detachment after disposal, and explicit WebGL context-loss feedback and remount.

The timing excludes React reconciliation, procedural streaming, the HUD, and
gameplay simulation. Animation-frame intervals include browser scheduling and
rendering contention. Submission time measures JavaScript work before async
GPU completion. GPU draw counts are instrumented WebGL calls. SVG publishes
image-element counts because Chromium does not expose comparable GPU draw calls
through this harness. Texture estimates use RGBA8, padding on packed pages, a
mipmap upper bound and an approximate triple framebuffer for PixiJS. Staging
bytes are recorded separately. Compressed bytes include the entire gzip-sized
fixture bundle plus the largest set of generated PNGs as a reproducible upper
bound for these scenes. They do not represent the full future game asset catalog.

### Decision from the cloud run

**Retain SVG for 2D production.** In Chromium 154 on Linux x64, PixiJS 8.23.0
used WebGL2 through ANGLE SwiftShader. The populated surface measured 25.7–27.0
FPS with SVG and 7.3–7.9 FPS with PixiJS across both density settings.
PixiJS issued four GL draw calls per frame and usually submitted frames faster
from JavaScript. That batching advantage did not produce faster end-to-end
software rendering. Both adapters fell short of 60 FPS on the populated scene.
The current-density Scout reached about 60 FPS in the SVG trials.

| Scene and source density | Draw objects | SVG p95 frame time | PixiJS p95 frame time | Cross-adapter changed pixels |
| --- | ---: | ---: | ---: | ---: |
| Scout, current | 76 | 16.7–16.8 ms | 150.0–166.7 ms | 254 |
| Scout, 128 px/m | 76 | 16.8–49.9 ms | 150.0–183.3 ms | 1,019 |
| Surface, current | 443 | 66.7 ms | 216.8–233.2 ms | 1,964 |
| Surface, 128 px/m | 443 | 66.7 ms | 216.7–250.0 ms | 4,487 |

Ranges cover two trials. Each fixed-frame repeat within one adapter passed the
existing image policy. Cross-adapter comparisons failed that policy in all
four cases. Original images show the same layout, layers, masking and visible
cues, with differences along filtered edges, tiny deck labels and particles.
The reference comparison policy and baselines were preserved.

The 128 px/m surface is the upper observed case for these fixtures. It used
about 24.8 MiB estimated decoded image bytes with a mipmap allowance in SVG,
and 36.5 MiB estimated PixiJS textures plus framebuffers under the same
allowance. Largest atlas edge across runs was 2,027 px. The largest compressed
bundle-plus-texture estimate was 767,352 bytes. Those results bound the
measured scenes at 128 px/m under the 128 MiB, 2,048 px and 15 MB targets.
They leave complete playable-asset coverage, physical GPU speed and
Firefox/Safari qualification open.

PixiJS offers a packed texture and mask path suitable for a future 2D adapter.
A production change would need to integrate that path into the React
lifecycle, culling and streaming while preserving logical input transforms and
the existing 60 Hz session. SVG keeps the current browser compatibility and
presentation lifecycle. The production build output in the benchmark revision
was byte-identical to the parent build, and the opt-in fixture has no production
entry point.
