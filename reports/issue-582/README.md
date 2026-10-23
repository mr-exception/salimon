# Continuous Scout audio (#582)

Fourth PR in the requested stack. One maintained CPAL 0.18.2 stream now mixes
ship and survival audio continuously through CoreAudio/WASAPI/ALSA defaults.
The old OS-process/file presenters are removed from normal playback. Existing
production observation/threshold owners, local attenuation/pan, controls and save
authority are preserved. Quiet powered ventilation joins restrained Core idle;
actual conversion, Fabricator, thrust, ramp, footsteps and power cues remain.
EVA/exterior has no invented transmitted machinery. Blackout retires loop targets.

The callback uses fixed 13-voice state, a 240-byte coherent atomic mailbox and
1,080-byte mixer. It neither allocates, locks, logs, writes files nor invokes
platform APIs. Samples carry phase across arbitrary callbacks. A 10 ms gain/pan
convergence and smooth output saturation bound transitions/headroom. Event
counters coalesce rather than building a delayed queue. Pause/focus/master mute,
terminal, reset and stale-producer admission retire cues; a 250 ms watchdog fades
stalled input. Scene/survival switches remain independent. Default device changes
or failures drop the old stream and retry at most once per second from current
observations. Silent streams remain resident until replacement/application drop.

[Maintained contract](../../docs/scout-presentation.md#soundscape-boundary-465),
validation, runtime guides and maintenance map are updated. Linux CI installs
ALSA development headers. No audio data enters checkpoints or domain commands.
[CPAL API source](https://docs.rs/cpal/0.18.2/cpal/) was checked alongside the
locked crate source; all default PCM sample variants are handled explicitly.

## Evidence

Actual host is macOS 26.2 / Apple M1. Original logs, source/binary context and
inventories are retained beside this report. WAVs are deterministic mixer output,
not microphone or loopback recordings. Production exemplars are driven by the
existing walking/Core/Fabricator/engine/ramp/EVA scenario observation suite,
which also asserts unchanged saved authority. Separately labeled 10-second
fixtures exercise quiet idle, work/thrust, survival, blackout, stress and mute at
48 kHz stereo with 60 Hz control publication.

The old presenter admitted new observations only after a half-second WAV child
finished, plus process/file/device startup. Its historical measurements remain
in [#485](../issue-485/README.md); that source does not establish speaker latency.
The new deterministic mixer has no scheduled half-second gap: identical input
produces exactly identical samples across irregular 127/512/143/11,025-frame
callback partitions. Transient onset appears within the first 48 generated
frames in the full-mix test. Actual device buffering is separately observed;
this is not an acoustic end-to-end latency measurement on all three platforms.

Native CoreAudio smoke evidence opens the actual built-in speaker, publishes
quiet source observations, then retires them. It demonstrates continued device
callbacks, 48 kHz stereo, 512-frame buffers (10.67 ms), and retained telemetry
including callback CPU maximum/overruns. It does not physically unplug/switch
outputs, certify Windows/Linux hardware, or prove listening comfort. Device
failure/replacement is bounded in the explicit main-thread lifecycle boundary;
physical switching/latency/cadence remains #557/#563 and human listening #561.

Release synthetic timing includes sample synthesis, mailbox updates and PCM
recording allocation/copies, and excludes device/OS scheduling/game simulation.
All samples/cases, including mute and any spikes, remain in the CSV. The debug
native smoke observed zero callback CPU overruns; release telemetry is retained
separately. Whole-frame budgets are not approved here.

Checks: workspace tests, focused ship/survival/mixer tests, opt-in original PCM,
actual default-device smoke, Clippy, formatting and documentation gates. Exact
final outcomes and timing values are in the retained validation logs and CSV.
No PR is merged by this task.

## Final results

- Workspace: pass; runtime 368 passed, 16 opt-in ignored. Focused ship mapping
  10 passed, survival 8 passed, callback mixer 3 passed; opt-in production PCM,
  10-second synthetic PCM and actual-device smoke each pass.
- Clippy with warnings denied, format/diff and per-parent documentation gates pass.
- Release 10-second/480,000-frame cases use 90.56–95.05 ms CPU including PCM
  export overhead. Mute remains exactly zero; stress peak is 0.132 normalized.
  [Every case](timings.csv) is retained, without clipping or discarded timings.
- [Release CoreAudio telemetry](native-output.json): 244 active callbacks,
  124,928 frames, 48 kHz stereo / 512 frames, maximum CPU 750,459 ns, zero
  callback CPU overruns; silence retirement continues to 255 callbacks.
  This short smoke does not establish a long-run hardware budget.

[Original production and synthetic WAV archive](audio-fixtures.tar.gz) ·
[Idle/coasting](cockpit-coasting.wav) · [Fabricator](fabricator-working.wav) ·
[Blackout transition](blackout.wav). Production exemplars hold an actual source
observation for two seconds; they do not claim a recorded continuous play session.
Source/binary hashes are in [context](context.json); historical failures and
platform limitations above are retained. No audible-device pass is inferred
for Windows/Linux or physical output switching.
