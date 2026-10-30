# Planetary environmental sound (#627)

`client/runtime/src/environment_audio.rs` observes the published world definitions,
body-fixed listener position and shared TT epoch. Audio never advances simulation,
grants knowledge or mutates saves. Normal play uses Vesper's actual vacuum climate,
sterile physical population and explicitly dormant geological activity. Positive
atmospheric/biological fixtures do not populate or change this frozen world.

## Sources and transmission

Five reserved continuous callback channels (27–31) carry local wind, actual
precipitation/explicit airborne dust, local geological activity, moving flexible
physical organisms and contact-coupled harvesting. No biome/music/wildlife loop
is present. Local weather is queried at the listener, with scale-height altitude
attenuation; remote cloud/storm grids cannot become listener rainfall. Dry weather
alone never supplies dust; canonical loading is zero. Wind pans upwind in the
body-fixed listener's right axis. Flexible Frond/Filament appendages must exist in
the stable physical population, have wind and atmosphere, and be within 40 m;
render LOD and cosmetic cover do not contribute voices.

Geological sources must be published activity events within 1 km. In vacuum only
actual ground contact couples vibration: Surface feet within 15 cm of ground,
or a landed Scout interior. EVA, jumps and a flying Scout cannot conduct ground
sound. Airborne geological paths require both listener atmosphere and source
AirAndContact medium. Source distance uses inverse-square-style bounded falloff;
terrain obstruction reduces airborne source gain. This is a restrained authored
acoustic approximation, not wave propagation, atmospheric pressure simulation or
measured material acoustics. Flexible-material rustle has the same distance and
terrain obstruction policy. At most the strongest source per band is retained.

The suit attenuates/filter exterior airborne sound. Inside Scout, ramp fraction
interpolates closed transmission 0.025/filter 0.04 to open 1/0.6, multiplied by
existing physical cargo-door/privacy partitions. Interior machinery retains its
own sources and mixing. These values are presentation tuning, not physical seal
or pressure authority. Contact harvest uses measured increasing extracted mass,
never held input alone; organic tissue, medicinal tissue, ice and mineral paths
have distinct filters. Restore/reset cannot replay prior extraction.

## Mixer, accessibility and lifecycle

The existing CPAL callback has 32 fixed slots, including the original 27 Scout
and survival slots. Environmental channels allocate nothing in the callback;
existing bounded mailbox, stale-control retirement, focus, pause, terminal,
master mute and device recovery apply. No subprocess/file playback is introduced.

Pause → S opens settings. E toggles environment audio, F cycles its independent
0–100% volume, T toggles environmental text cues. These backward-defaulted
version-1 preferences are separate from saves and ship/condition audio. Local
storms/reduced visibility and significant local ground activity have action-bar
text independent of audio enable/mute, below active modal/interaction guidance.
No new environmental injury or automatic exploration objective is implied.

## Verification and limits

`environment_audio/tests.rs` checks actual climate/activity/population definitions,
vacuum/contact/range, weather locality, upwind stereo, altitude, explicit dust,
biological obstruction, mute/volume and unchanged save bytes. Scout audio tests
exercise real canonical mining, measured mass admission and restore retirement.
`export_environment_audio_627` exports production continuous callback PCM with
metrics; `scenarios/evidence/environment-audio.json` checks native quiet Vesper,
settings and lifecycle through the real window adapter.

See [completion evidence](../reports/issue-627/README.md). Exported PCM is listening
material for [#561](https://github.com/mr-exception/salimon/issues/561), not a device
recording, physical acoustic measurement or subjective sign-off. Physical device,
Apple/Windows and M1 qualification remain unverified release work.

E2E normally leaves personal preferences unopened. An explicit
`SALIMON_SETTINGS_FILE` enables the real preference IO adapter against a
caller-owned isolated file for native settings verification.
