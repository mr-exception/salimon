# Game atmosphere and audio
Approved phase 5 (2026-11-26); [experience](experience-direction.md), [HUD](hud-ui-style-guide.md), [navigation](panoramic-navigation-design.md) and existing domain rules authoritative. Comfort in Scout, isolation outside, curiosity at discoveries, occasional real danger. Silence and continuous player control matter.

## Audio
Centralized Web Audio manager; lightweight library (e.g. Howler) conditional on testing, registry, persistent independent gain channels, spatial stereo, attenuation/room obstruction/filtering, crossfade, instance limits, reuse, disposal, autoplay unlock at first interaction. Initial volumes master 100, music 35, environment 75, machinery 65, character/tools 80, interface 50, warnings 85 percent; tune by review. WAV source, compressed browser-compatible runtime codec selected by test, mono positional where practical, stereo music, seamless loops, category loudness normalization, stable IDs/variations/playback rules, documented origin/rights; offline generation only, no build/runtime AI dependency. Footsteps per material 6–8 variants, mining 4–6, interaction 3–4, transfers 2–3, doors 1–2/model, recognizable warning per severity; small pitch/volume variation except warning identity.
Scout: low Core hum, mechanical Fabricator/bench, airflow, motor/lock doors, quiet cockpit electronics, restrained hull resonance, deep thrust vibration; habitation quieter. Local machinery stops when unsupported/unpowered; closed partitions attenuate. Outside: material footsteps, wind only with atmosphere, biological/geological sounds only with supported sources, vacuum no airborne sound; suit/contact/structure transmission remains. Crossfade door traversal 300–600 ms from actual location/door state, no leakage/life-support mechanic.
Minimal ambient electronic music, sparse melody/synth/orchestral textures, deliberate silence (illustrative 20–30% music time, not mandatory), cooldowns and fades; quiet routine, occasional calm Scout, significant discoveries, travel, known serious danger, return to safety and narrative revelation; no anticipation of unknown threats/spoilers. Independent mute. Loading no world audio until ready; first user gesture unlock; pause stops gameplay-driven sounds/fades ambient, hidden suspends presentation audio; resume current state, no stale replay; view switches preserve settings; death stops gameplay sounds. Domain owns simulation pause/offline behavior.

## Motion and feedback
Natural walk, smooth short turn, settle, breathing, tool/machine motion, restrained damage and collapse; visuals never delay controls. Smooth centered bounded follow, no auto rotation/aggressive look-ahead, keep player clear of HUD, instant option. Bounded smooth wheel zoom/reset, panel scrolling takes priority, no reconnaissance leak. Temporary cyan destination marker, rejected marker with reason, selected approach target, clear on arrival/cancel; no persistent paths/selection ring. 2D↔panoramic sensor transition 250–400 ms fade, no clock/camera reset or changed admission. State-driven engine startup/lift/acceleration/coast/landing/touchdown, no speed-based fake star streaks or airborne vacuum sound. Physical sky, sparse plausible stars, planet silhouettes and atmosphere, minimal overlays, no grid/overdone flare. Vibration rare: none routine, optional damage, light startup, restrained thrust, brief touchdown/collision; visual/input transforms consistent, disable option.
Warnings normal/advisory/warning/critical, actual exposure/protection/damage thresholds, persistent visual + controlled sound cadence, deduplicate; information transient, warning contextual, critical prominent; no focus theft or routine interruption. Fast dialog fades 100–200 ms, context menu immediate, confirmation/rejection/completion/tactile transfers, no hover sound spam.
Accessibility: independent music/environment/warnings etc, stereo/mono, camera smoothing, vibration off/reduced/full, flashing off/reduced, motion/interface effects off/reduced/normal, heat distortion off, particle reduction, essential non-color/non-audio cues. Initial values provisional. Seven acceptance scenes: quiet Scout, expedition, gathering, hazard, power loss, flight/landing, accessibility; test triggers/lifecycle/disposal and actual audible/visual review. No new simulation state or hazard mechanics.

## Browser audio foundation (#942)

The browser expedition uses a single native Web Audio graph shared across its
2D and cockpit views. Native Web Audio supplies gain buses, timed fades,
stereo panning, low-pass filters, decoded buffer reuse and direct lifecycle
control. A library such as Howler would simplify codec fallbacks and playback
conveniences, while adding a dependency and leaving the expedition's scope
and event management in application code. Help → Audio settings controls
independent gains and centered mono playback. Preferences use a separate
browser storage key from the expedition and remain independent of save versions.

The initial cues are short original mono sounds for the committed boarding-door
transition and the existing persistent survival warning. Visual warnings remain
available at zero gain and when the browser cannot play audio. The manager
drops cues before a trusted input unlock, discards pending sounds when the
view, visibility or play state changes, and stops audio on expedition end.
Sound observes the domain's existing state. Sound never advances it.

Authoring WAVs are in `art/sources/audio/`, and compressed runtime clips are in
`client/assets/audio/`. These original Salimon placeholders are redistributable
with the project. Encode mono runtime clips offline with
`ffmpeg -i SOURCE.wav -map_metadata -1 -ac 1 -c:a libmp3lame -b:a 64k CLIP.mp3`.
Opus at 32 kb/s was smaller for these two few-hundred-millisecond clips.
MP3 was chosen for broad browser decoding support and adds about 2.7 kB across
both clips compared with the Opus exports. Use codec and seamless-loop tests
when longer music or environmental content is authored.
