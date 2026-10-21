# #465 — Scout spatial audio presentation

Delivered on 2026-10-21, stacked on [PR #484](https://github.com/mr-exception/salimon/pull/484)
(#463 lighting/reference cameras), not merged automatically.

## Implemented

- Snapshot-only typed mapping for Core idle/conversion, funded Fabricator work,
  powered thrust, moving ramp, grounded/distance-driven interior footfalls,
  blackout/restoration and low-power crossing. No domain state or run-save changes.
- Ship-local listener/source positions, equal-power stereo pan, bounded inverse
  square attenuation; Surface/EVA is silent rather than inventing an atmosphere.
  Queue/reset/mute behavior preserves deliberate quiet and rejects stale cues.
- Original bounded stereo PCM with short edge fades and headroom. Shared optional
  OS playback extracted from existing #383 code, without duplicating survival
  content or adding Cargo/native link dependencies.
- Persisted master mute, independent ship mute and 0–100% ship gain in the existing
  accessibility settings. Legacy version-1 preferences default/migrate in place;
  invalid gain rejects. Existing survival cues honor master mute.

## Resource bounds and measured synthesis

Maximum four queued transients, eight mixed voices and one scene child (plus
existing one condition child). One PID-specific 44,144-byte half-second WAV is
rewritten after its child exits and removed on normal drop. Child exit/error
and 3-second timeout are observed; errors disable further scene playback until
suppression/reset. No elapsed-time-growing PCM bank or audio history.

Ubuntu 24.04 x86_64, Rust 1.99.1 debug profile: the focused 8-voice synthesis test
produced 32 half-second chunks in 186,580 microseconds (about 5.83 ms/chunk),
44,144 bytes/chunk. This is a single host sample including bounded test checks,
not an M1/native OS playback or application frame-budget measurement. Voice/memory
bounds are portable; timbre, latency and system process costs are not proven.

## Validation

Rust/cargo 1.99.1, Python 3.12.14.

- `cargo test --workspace --locked`: 695 tests passed; 22 opt-in tests ignored.
- `cargo test -p salimon-client --locked scout_audio -- --nocapture`: eight focused
  event/spatial/PCM/silence/production-Core tests passed, including the timing above.
- `cargo build --workspace --locked`: passed.
- `cargo clippy --workspace --all-targets --locked -- -D warnings`: passed.
- `cargo fmt --all -- --check` and `git diff --check`: passed.
- The inherited Python suite from #463 passes (95 tests); this change adds no
  Python tooling changes.
- Tests cover explicit alert transitions, queue/mix saturation, attenuation and
  stereo channels, mute/focus/interior/run/time suppression, teleport/drift guards,
  real production Core feeding/restoration/pause and no observation-side mutation.
  Preference tests cover legacy load, new-field save/reload, invalid gain,
  settings key toggles and master suppression of existing survival cues.

## Unrun gates and delivery boundary

No audio-device listening, OS playback, seamless-continuity, native UI screenshot
or actual application frame-tail measurement was possible here. Display capture
is blocked as recorded in #463; no native playback pass is inferred from WAV
headers or synth timing. The minimal existing OS adapter starts a process for
half-second chunks, so gaps and startup/response latency are possible. A continuous
backend has not been introduced under the guise of native verification.

[#485](https://github.com/mr-exception/salimon/issues/485) owns complete Scout loop
listening, stereo/quality/accessibility review, playback/lifecycle/device checks,
matched whole-frame cost and the decision whether continuous playback is needed.
Source closure records delivered portable/native adapter work and explicit
transfer, not native audio acceptance. See the
[canonical mix/quiet-zone/budget contract](../../docs/scout-presentation.md#soundscape-boundary-465).
