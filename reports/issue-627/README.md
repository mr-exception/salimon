# Issue #627 — environmental audio

Parent: `issue-626-planetary-chart` at
`f3d29e30837b5164d67fc810125e7529207e94ab` (PR #708).
Cloud Linux x86_64, Rust/Cargo 1.99.1, Python 3.12.14, Xvfb/Mesa lavapipe;
production CPAL mixer exported directly to 48 kHz stereo 16-bit WAV.

## Outcome and source mapping

Local body-fixed weather, published geological activity and stable physical
flexible organisms feed five bounded continuous channels. Actual medium,
listener altitude/contact, source distance, terrain occlusion and Scout ramp/
cargo/privacy boundaries govern transmission. Airless Vesper remains sterile
and quiet. Real increasing extracted mass admits contact harvesting with
material-dependent filtering; restore cannot replay it. Separate environment
volume/audio/text preferences preserve ship/condition settings and master mute.
Local storm/ground-activity text is independent of sound. No new world content,
fauna, damage, music, save authority or external audio assets were introduced.

| Behavior | Source / regression | Canonical guide |
| --- | --- | --- |
| Local source and media admission | `environment_audio.rs`, its tests | [planetary audio](../../docs/planetary-audio.md) |
| Actual harvest mass and restore | `scout_audio.rs`, `scout_audio/tests.rs` | same guide |
| Fixed-memory continuous callback | `native_audio/mix.rs`, its tests | [acoustics](../../docs/scout-acoustics.md) |
| Independent settings/native inputs | `condition_feedback`, `app.rs`, `automation/native.rs` | same guide |

Documentation impact: updated owning [AI guide](../../client/runtime/README.ai.md),
[architecture](../../client/runtime/architecture.md),
[invariants](../../client/runtime/invariants.md), root technical architecture,
maintenance map, validation, approved art-direction implementation note, Scout
acoustic/presentation contracts and runner guide. Save schema remains 28 because
all sound state is observational and preferences have backward-defaulted fields.

## Verification

Commands use the workspace
Rust/ALSA/Mesa environment described in previous stack reports.

- `cargo test --workspace --locked --offline`
- `cargo build --workspace --locked --offline`
- `cargo clippy --workspace --all-targets --locked --offline -- -D warnings`
- `cargo fmt --all -- --check`, `git diff --check`
- `python3 scripts/check_docs.py --base issue-626-planetary-chart`
- `python3 -m unittest discover -s scripts -p 'test_*.py'`
- `cargo test -p salimon-client environment_audio --locked --offline`
- `SALIMON_AUDIO_EVIDENCE=$PWD/reports/issue-627 cargo test -p salimon-client export_environment_audio_627 --locked --offline -- --ignored --exact environment_audio::tests::export_environment_audio_627`
- `scripts/salimon-test run scenarios/evidence/environment-audio.json --binary target/debug/salimon-client --screenshot-command '["native"]' --summary --artifacts artifacts/issue-627-native` under Xvfb/lavapipe.

## Original listening material and limits

[Metrics](metrics.json) describe original production continuous-callback PCM,
not an OS recording. Each clip is 2 s / 96,000 stereo frames at 48 kHz. Explicit
fictional atmospheric fixtures use actual ClimateDefinition/ActivityDefinition/
Population publications; hand-selected boundary transmission and material cases
are controlled mappings, not a claimed atmospheric Vesper visit. Native E2E is
silent device-preview mapping and real wgpu output. No hardware playback,
physical latency or subjective acoustic approval is claimed.

- [Vacuum quiet](vacuum-quiet.wav)
- [Suit local weather](suit-local-weather.wav)
- [Scout ramp open](scout-ramp-open.wav), [closed](scout-ramp-closed.wav)
- [Organic contact harvest](organic-contact-harvest.wav)
- [Mineral contact harvest](mineral-contact-harvest.wav)
- [Vacuum geological contact](vacuum-contact-geology.wav)
- [Physical flexible biology](physical-flexible-biology.wav)

Sound synthesis and material/coupling levels are restrained provisional
presentation tuning. Terrain occlusion is approximate; no ray-traced diffraction,
pressure/airlock simulation or environmental injury is claimed. The strongest
source per environmental band is retained within fixed distance budgets.
[#561](https://github.com/mr-exception/salimon/issues/561) owns actual listening,
subjective tuning and hardware release qualification; #557/#563 retain native
input/platform and M1/Metal performance qualification. These remain unverified.

## Corrections during verification

Initial compile checks caught fixed-array sizes in old test-only exports and
private adapter/type mismatches; corrected. The initial wind test sampled a
motionless pole; moved the controlled observer to a real nonpolar wind sample,
without changing climate or tolerances. Initial failed attempts are not passes.

The first native settings attempts correctly left preferences unchanged because
E2E intentionally does not open user preference storage. The harness now opts
in only when a test-owned `SALIMON_SETTINGS_FILE` is explicitly supplied. Final
evidence uses an isolated workspace file; no user save/preference path is used.

## Final native and focused results

Native production route: **67/67 steps passed**, 107.770 s. Ordinary canonical
Surface → ramp → InsideShip movement retained vacuum environmental silence;
independent E/F/T settings changed and persisted via real native menu dispatch.
Original [settings](step-017-environment-settings.png),
[changed preferences](step-030-independent-settings.png),
[airless exterior](step-010-vacuum-quiet.png) and
[Scout interior](step-067-scout-interior-vacuum.png) PNGs were inspected.
The final prompt-priority adjustment keeps interaction guidance above weather
text; its focused native/app tests passed 53 tests (3 opt-in exports ignored).

Controlled environment regressions: 5 passed, 1 opt-in export ignored; real
harvesting regression: 1 passed. Explicit production PCM export: 1 passed.
Build, Clippy warnings denied, fmt, diff and documentation checks passed.
Python runner tests: 136 passed. [PCM validation](pcm-validation.json) confirms
exact vacuum silence, different organic/mineral content and lower closed-ramp
energy. [Source context](source-context.json) retains actual local weather and
definition IDs. Mixer/mailbox storage is 3,240/672 bytes; timings in metrics
are shared cloud CPU measurements, not device latency or M1 performance.

Final full workspace suite: **1,231 passed, 0 failed, 50 opt-in tests ignored**
([complete log](workspace-tests.log.gz)). No ignored hardware test is counted
as a pass. The explicitly requested callback export passed separately.
