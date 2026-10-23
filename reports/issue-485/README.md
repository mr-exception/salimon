# #485 — Scout audio cloud verification

Cloud acceptance delivered 2026-10-23, stacked after #483 / PR #571. No audio
content, simulation, save schema, gain policy or native player selection changed.
A narrow process-creation function lets the existing presenter exercise real child
failure/lifecycle paths without changing global PATH or requiring an audio device.

## Evidence and results

[Environment/source/binary hashes](environment.json), [raw logs](logs/) and
[original stereo WAVs plus manifest](audio/manifest.json) retain the tested context.
Linux x86_64 / AMD EPYC 9V74, Rust 1.99.1, Python 3.12.14, release diagnostics.
These are synthesized PCM exports, **not recordings from an audio device**.
Each WAV is a bounded half-second, 22050 Hz, 16-bit stereo production chunk.
The manifest labels production observations and explicit channel controls; no
sample library or external source is used. All sample peaks are <=30,000.

- Production loop: interior/cockpit, actual walking/footstep and jump rejection,
  exact carried Core intake/restoration/conversion, explicit power-loss fixture,
  funded Fabricator work, pause/no-power/blocked work, powered thrust/engine off,
  and ramp movement pass. Audio observation preserves byte-identical saved
  authority around each observation, including identity/mass/power state.
- Existing movement/EVA/nearby-EVA/surface routes run production protocol commands;
  every observed exterior state has zero ship voices. Tests require actually
  reaching outside. [EVA silence](audio/eva-silence.wav),
  [nearby silence](audio/nearby-eva-silence.wav),
  [surface silence](audio/surface-silence.wav),
  [settled blackout](audio/blackout-settled-silence.wav),
  [no-power work](audio/fabricator-no-power-silence.wav) and
  [pause](audio/paused-silence.wav) are all-zero PCM.
- Spatial tests pass heading-dependent pan, distance falloff/cutoff, channel
  isolation, bounded voices/queue, alert priority and deterministic PCM/headroom.
  [Left control](audio/channel-left.wav) and [right control](audio/channel-right.wav)
  isolate their named channel. New-run/restore/death/restart and focus-suppression
  gates discard queued transients; restart/restoration does not replay blackout.
- Real Unix children: missing player and disk failure latch unavailable; no retry
  until suppression; nonzero exits/timeouts stop; no child overlaps; suppression
  and presenter drop kill and reap children; drop removes the scratch WAV.
  The successful substitute-player diagnostics exercise normal completed children
  and repeated file writes through the same boundary. Unix process checks are
  portable-host evidence, not macOS/Windows playback-device acceptance.
- Existing versioned preference tests pass legacy defaults, mute/ship gain roundtrip,
  invalid values and separate settings storage. App source maps pause/focus/
  occlusion/suspend through the same stop/active gates; native-host tests exercise
  focus/modal/background admission. Physical sleep/occlusion/audio-device behavior
  is not claimed by these tests.

17 production-observation/silence examples plus two channel controls are retained.
Listen in particular to [interior](audio/interior.wav),
[movement](audio/movement.wav), [Core](audio/core-conversion.wav),
[Fabricator](audio/fabricator-working.wav), [engine](audio/engine-thrust.wav),
[ramp](audio/ramp.wav), [blackout](audio/blackout.wav) and
[restoration](audio/restoration.wav). These short examples cannot establish
continuous OS playback, chunk gaps or end-to-end audible latency.

## Matched release diagnostics

Five serial enabled/muted trials alternate order with identical session workload.
Each synthesis trial performs 32 observations/mixes; enabled work synthesizes one
Core chunk per update. Raw trial totals and outliers are in the manifest.
Median microseconds per update: muted **0.34375**, enabled **1672.65625**.

A second five-trial matrix performs eight updates through the real presenter,
WAV write and subprocess boundary with `true` as an explicitly substituted player;
enabled trials start and wait for eight real successful children, muted trials
start none. Median microseconds/update: muted **0.625**, enabled **2526.5**.
This measures bounded synthesis/file/process overhead on this CPU. It measures
neither audio-device latency nor application frame tails, and establishes no
hardware budget or continuous backend approval. Hardware output is unavailable.

## Commands and checks

From repository root, with the installed Cargo toolchain:

```sh
cargo test -p salimon-client --locked scout_audio -- --nocapture
SALIMON_AUDIO_EVIDENCE="$PWD/artifacts/485-audio" cargo test --release -p salimon-client --locked cloud_audio_evidence_485 -- --ignored --nocapture
cargo test --workspace --locked
cargo build --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo fmt --all -- --check
python scripts/check_docs.py
python scripts/check_docs.py --base cloud/483-lighting-verification
```

Export requires a fresh directory. Focused audio: **11 passed, one ignored**;
release evidence opt-in: **one passed**; workspace: **864 passed, 31 ignored**.
Build/Clippy/fmt and documentation/link/base-impact checks passed. Initial new
surface replay omitted the route's seed and correctly failed its outside guard;
using the actual route seed fixed that fixture. Initial Clippy required the
standard fixed-array chunk iterator; corrected and rerun successfully.

## Remaining human criteria and documentation impact

[#561](https://github.com/mr-exception/salimon/issues/561) owns listening to the
linked channel-labeled WAVs for orientation, timbre, headroom/harshness, meaningful
quiet and cue audibility. It also owns actual audible continuity/chunk-gap review
using a recording on suitable output; exported chunks cannot pass that criterion.
If those observations require a continuous backend, retain the failure and open
a focused implementation follow-up rather than approving the current boundary.
[#557](https://github.com/mr-exception/salimon/issues/557) owns actual macOS/Linux/
Windows playback, focus/occlusion and physical suspend/resume with devices.
[#563](https://github.com/mr-exception/salimon/issues/563) owns hardware costs and
matched whole-frame CPU/render/GPU/tail budgets. These remain unrun here.
Rights/balance decisions remain [#537](https://github.com/mr-exception/salimon/issues/537).
No listening, physical platform, baseline, hardware FPS or Metal approval is granted.

Runtime architecture, Scout presentation and validation now document the process
seam, production observation suite, export commands and exact follow-up owners.
Owning runtime invariants still hold: audio remains observational, shares the
native player and preference store, and never enters domain/save authority.
Source mapping: `scout_audio.rs` presenter -> `scout_audio/verification.rs`
process/session regressions; `synth.rs` -> existing PCM/spatial tests and exports;
`condition_feedback/settings.rs` -> existing persistence/invalid-setting tests.
