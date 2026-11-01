# Issue #471 — optional alien-world opening and first-hour slice

The canonical traveler wakes outside the familiar landed Scout. The existing
skippable survival tutorial now briefly states that Earth is home, the ship is
familiar, and the voyage/landing are missing. It suggests shelter/Archive,
gathering or exploration without a quest chain. T dismisses the text; ordinary
elapsed time and tutorial save state prevent a replayed onboarding grant.
Remembering Earth still grants no route or catalog search.

The existing ship lighting, machinery, environmental audio, supplies, mining,
Core conversion and Fabricator remain authoritative. The #470 packet/site work
adds a concrete voluntary recovery motivation and inspectable ambiguity rather
than an explanation of the missing journey. See the
[production contract](../../docs/opening-and-investigations.md).

## Evidence

[Terrain-ready opening original](../issue-470/canonical-journey-awakening-outside-scout.png)
and the combined [GPU/state index](../issue-470/native-evidence.json) show the
quiet canonical exterior, warm cabin/Archive contrast, two physical clues and
their source/uncertainty text through production Vulkan. Protocol input, seed 0,
1280×800, scale 1, Mesa llvmpipe are explicit; the initial body and resources are
the published Vesper definition, not an Earth fixture.

The [five portable opening/investigation tests](portable-opening.log) pass,
including an actual walk to the nearby relay and a declared **3,600-second**
production session using one-second ticks, ration crafting/collection, two food
consumptions and exact save/reload. It ignores every narrative lead and gains no
unearned destination. The existing real-material canonical mining/carrying/
Fabricator/disk-restore regression passes separately. Native remote-site and
recovered-packet setups explicitly declare placement/input resources; they do
not claim a measured complete human gathering/flight route.

Production callback PCM exports use actual fresh canonical observations and the
existing ship mixer/control/environment adapters, then the native continuous
mixer. They contain 96,000 stereo frames each at 48 kHz (125 × 16 ms controls):

- [Outside Vesper](vesper-outside.wav): measured peak 0; no atmospheric sound is
  fabricated for this airless world.
- [Inside Scout](scout-inside.wav): measured peak 0.005574023816734552, retaining
  the existing powered machinery soundscape.

[PCM source context, original metrics and hashes](manifest.json) distinguish
synthesis from OS playback. This is not a physical listening or device-latency
trial, approved mix/balance decision or 60 FPS measurement.

## Validation and limits

Exact shared build/lint/script/docs/workspace commands, 1,293 repaired/complete
normal Rust case coverage, 52 normal opt-in ignores, nine original GPU captures,
source/base revision and earlier failures are retained in the
[combined validation report](../issue-470/README.md). Additionally:

```sh
SALIMON_AUDIO_EVIDENCE=/workspace/salimon/artifacts/opening-pcm-v1 \
target/debug/deps/salimon_client-a1438275ed68383a \
--exact scout_audio::verification::opening_production_audio_471 --ignored
```

Passed. The optional one-hour test initially failed when the traveler idled
without enough survival actions (death at 2,800 seconds; one ration extended the
run to 3,520 seconds). The final route uses the existing starter reserve and two
rations without changing depletion, recipes, supplies or recovery rates. Mouse
release is explicitly represented before the next consumption; repeated-held
presses do not silently consume another item. Survival can still end an idle run.

An initial environmental-audio settings preflight failed because no isolated
`SALIMON_SETTINGS_FILE` was supplied. A later contended readiness attempt timed
out at the existing 180-second gate. Both remain failed setup attempts, not
accepted output. Final dedicated opening captures wait for the original
terrain-readiness gate on a quiet executor. The separate settings route then
**passed all 67 steps / four captures** in 98.458 seconds with a fresh settings
file. [Actions, assertions and capture source states](environment-audio-result.json)
retain the successful result, including independent environment audio/volume/cue
settings and the [actual interior vacuum view](step-067-scout-interior-vacuum.png).

```sh
SALIMON_SETTINGS_FILE=/workspace/salimon/artifacts/opening-audio-settings-v2.json \
WGPU_BACKEND=vulkan \
VK_DRIVER_FILES=/workspace/.salimon-tools/sysroot/usr/share/vulkan/icd.d/lvp_icd.json \
xvfb-run -a -s '-screen 0 1280x800x24 -fp /workspace/.salimon-tools/sysroot/usr/share/fonts/X11/misc' \
python scripts/salimon-test run scenarios/evidence/environment-audio.json --binary target/debug/salimon-client
```

Routine objective cloud evidence is delivered here. A subjective one-hour human
experience/listening trial (#661/#721), actual physical input/platform (#720),
hardware budgets (#722), provisional survival/recovery balance (#723), and future
public/commercial source rights (#659) remain separate qualification. No Hall,
confirmed wormhole entry, mandatory investigation or unearned Earth destination
is introduced. Full real-Gaia population/budget acceptance remains explicitly
[incomplete under #524](../issue-524/README.md).
