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
original 160-byte uniform carried the bus bit in its previously unused `center.w`.
Additional vertex arithmetic is bounded (two smoothstep zones, no light loop).
The machinery extension below enlarges that uniform to 192 bytes.
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

## Soundscape boundary (#465)

Runtime `scout_audio.rs` observes immutable session/ship/character snapshots. Its
private typed cues cover Core hum/conversion, funded Fabricator work, selected
powered thrust, moving ramp, grounded interior footsteps, bus loss/restoration
and low-power crossing. E02 may consume the same authoritative observations;
there is no second machinery controller, timer or gameplay event bus. Existing
#383 heart/hunger/breath cues keep their content and threshold owner, sharing
only `native_audio.rs` OS playback and the master mute preference.

The listener and machinery anchors are ship-local metres; left/right pan follows
look direction. A softened inverse-square curve `1/(1+d*d/9)` and 20-metre cutoff
bound sources, with equal-power stereo panning. There is no HRTF or obstruction
solver. Interior footfalls require actual horizontal distance, walking input,
power/artificial gravity and floor-level eye height. Ship translation, airborne
drift and teleport/reset do not imply footsteps. No exterior wind/atmosphere,
transmitted machinery in Surface/Space/NearbyBody, music, voices or chatter is
invented. DoorwayBlend retains cabin sound until the location leaves the ship.
A powerless cabin becomes quiet after its finite blackout cue; idle power retains
only restrained Core hum. Existing prompts/screens remain visual alternatives.

Pause -> S settings adds M master mute, B ship audio and G ship volume, cycling
0/20/40/60/80/100 percent. A still controls only survival condition cues. Version-1
settings gain defaulted fields (master mute false, ship audio true, volume 60),
preserving older preferences; invalid volume and unknown versions fail. These
preferences live outside run saves and survive new runs/restarts. Pause, focus,
occlusion, suspend, terminal, tour and run/time reset discard pending scene cues;
E2E and benchmarks never start scene playback. Missing or failed playback is
explicitly reported and stops retrying until a lifecycle/settings suppression.

Original deterministic 22050-Hz stereo 16-bit synthesis uses 0.5-second chunks,
short edge fades and <=8 simultaneous voices. Four queued transients are bounded;
power alerts take priority over footsteps. One 44,144-byte WAV is rewritten only
after its single playback child exits, uses a PID-specific filename beside the
settings file, and is removed on normal presenter drop. One scene child plus the
existing one condition child are the maximum; no sample library or Cargo
link/dependency is introduced. Absolute sample peak is bounded to 30,000.
The scene's scratch PCM is ~43 KiB; event/mix storage is bounded independently
of elapsed play time. There is no persistent gameplay audio history.

This minimal OS boundary reuses afplay, Windows SoundPlayer or optional Linux
paplay/aplay. It can have audible chunk gaps and up to a chunk of response latency;
OS process startup/file IO are not realtime streaming. Native listening and
matched whole-frame costs remain [#485](https://github.com/mr-exception/salimon/issues/485),
which must decide whether a continuous backend is needed. Synthesis measurements
are not audio-device or M1 budget approval. See the
[audio report](../reports/issue-465/README.md) for actual tests and limits.

## Machinery feedback boundary (#464)

`runtime/scout_feedback.rs` maps immutable ship and Fabricator snapshots to typed
`MachineryFeedback`. This is a stateless presentation adapter; physical power,
energy, recipes, interaction, transforms, elapsed time and saves retain their
existing owners. Automation exposes the same mapping in `machinery` inspection.
Loading/restoration takes the current snapshot immediately; no event replay,
work catch-up or extra animation history enters a checkpoint.

| Surface | Authoritative observation and feedback |
| --- | --- |
| Engine glow | Signed powered free-flight throttle; magnitude controls drive emission, zero while coasting/landed/assisted/unpowered; separate running lamps remain bus powered |
| Core / spine / charge gauge | Charge-proportional steady emission, with a restrained four-second conversion modulation while the bus is powered; blackout extinguishes electrical accents |
| Fabricator panels | Quiet idle, progress-positioned teal work sweep only when funded, steady amber blocked tray, no emission during blackout; progress freezes with the actual job |
| Cockpit | Ready indicators follow powered cockpit authority; opposed rudder pedals move at most 1.5 cm from actual ramped yaw; monitor retains signed throttle/telemetry |
| Door / ramp / landing supports | Existing presented door fraction and hinge/extension geometry; supports remain authored fixed structure attached to the ship, including touchdown/bounce motion |
| Output / carried / loose cargo | Existing deliberate tray appearance/collection, carried grip and coherent fragment/ship transforms; no invented collection animation or duplicate item |

The phase uses integer active elapsed modulo four seconds before conversion to
f32. Paused/terminal worlds freeze it. Fabricator work uses actual normalized
job progress, so interruption cannot accumulate a hidden animation backlog.
Only named functional surfaces receive tags; structural meshes, colliders,
walking lanes, GLB authoring and mass accounting are unchanged. Existing physical
screen statuses and prompts are the nonanimated/text alternatives. There are no
flashing alerts, camera shake, particles, lights or postprocessing passes.

Renderer adds one f32 role per immutable ship vertex (72 -> 76-byte stride) and
two vec4 uniform slots (160 -> 192 bytes per rendered ship frame). Geometry counts,
textures, two opaque/glass draws, buffers and pipelines remain fixed. The role
shader adds bounded arithmetic with no source loop or dynamic allocation.
[Report](../reports/issue-464/README.md) records actual software-Vulkan debug costs
and limits, rather than claiming a full-frame or M1 budget pass.

An opt-in offscreen production-pipeline test records 14 synthetic reference
states, raw CPU/GPU samples and byte-identical repeated PNGs without a desktop.
These images test shader/DTO behavior, not production input or a gameplay run.
Run with an absolute output directory:

```sh
SALIMON_SCOUT_FEEDBACK_DIR="$PWD/artifacts/scout-feedback" VK_DRIVER_FILES=/usr/share/vulkan/icd.d/lvp_icd.json cargo test -p salimon-renderer --locked scout_feedback_gpu_evidence -- --ignored --nocapture
```

The normal-play interaction walkthrough, reference hardware budgets and observed
Fabricator reference-text orientation remain [#489](https://github.com/mr-exception/salimon/issues/489).
The absent scanner hologram will integrate through #430/#466 when available.
No native appearance, mask/tolerance or baseline approval is granted here.
