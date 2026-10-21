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
