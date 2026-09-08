# #561 — current Metal review packet

Collected 2026-10-31 on the actual M1/Metal/scale-2 desktop, optimized binary
identified in [#557](../issue-557/README.md). This is **candidate evidence**;
there is no image, mask, tolerance, audio, comfort or baseline approval.
The owner deferred physical/personal review; all remaining decisions transfer to
[#721](https://github.com/mr-exception/salimon/issues/721).

## Automated results

- All **nine corpus groups / 128 original PNGs / 38 distinct byte contents**
  captured successfully from production GPU surfaces at 1280×800, scale 2.
- Frozen repeats passed temporal comparison in all nine groups; compared repeats
  have zero changed pixels. This establishes repeat stability, not appearance.
- The additional 21 universe routes and 32 construction routes passed and retain
  **163 original PNGs** plus full state/measurement traces. These fixed-step
  routes do not measure ordinary-play performance or physical input.
- The paid Recovery Module/Archive route passed with nine additional originals
  at 1280×800 and 1920×1080: [#660 packet](../issue-660/README.md).
- Original 48 kHz stereo callback WAV exports and actual default-device callback
  telemetry are available. No acoustic recording or human listening was performed.

## Originals for review

[Candidate corpus/provenance](metal-corpus/corpus.json) contains exact source,
binary, fixture-contract, context and per-image hashes. It retains status
`candidate`. Capture used clean tracked tooling revision `125944f1` and the
unchanged parent runtime bytes. Original contracted names/repeats are preserved.

| Group | Example original | Temporal result |
| --- | --- | --- |
| Cockpit | [forward landed](metal-corpus/cockpit/forward-landed-repeat-00.png) | [passed](metal-temporal/cockpit/summary.json) |
| Interior | [dashboard approaching](metal-corpus/interior/dashboard-approaching-repeat-00.png) | [passed](metal-temporal/interior/summary.json) |
| Resources | [active mining](metal-corpus/resources/active-mining-context-repeat-00.png) | [passed](metal-temporal/resources/summary.json) |
| Contacts | See corpus `contacts/` inventory | [passed](metal-temporal/contacts/summary.json) |
| Static | See corpus `static/` inventory | [passed](metal-temporal/static/summary.json) |
| Exterior | See corpus `exterior/` inventory | [passed](metal-temporal/exterior/summary.json) |
| Core | See corpus `core/` inventory | [passed](metal-temporal/core/summary.json) |
| Ramp | See corpus `ramp/` inventory | [passed](metal-temporal/ramp/summary.json) |
| Moving poses | See corpus `moving/` inventory | [passed](metal-temporal/moving/summary.json) |

The [native index](../issue-557/native-index.json) locates every additional
universe/construction image under `native/`; its paths are relative to `reports/`.
`corpus-traces/*.json.xz` retain full production route states. Original callback
audio files/metrics are in `audio/`; these are production PCM exports, not
speaker recordings. [Actual speaker callback telemetry](../issue-557/audio-device.json)
is a separate observation. [Overview contact sheet](overview.jpg) is a derivative
navigation aid; inspect originals for decisions.

## Remaining review

Use [#721](https://github.com/mr-exception/salimon/issues/721) and
[the maintained checklist](../../docs/macos-qualification.md). Record actual
reviewer/date, artifact hashes and accept/reject by group. Reconcile physical
scale 2 with older pending scale-1 descriptors. Empty masks, 8/255 channel
tolerance and .001 changed fraction remain proposed; repeat stability does not
approve them. Software-Vulkan/nightly originals and approvals remain separate;
no hosted/Linux campaign was claimed in this local Metal run. A baseline promotion
requires its own explicit reviewed PR.

Ordinary full-rate movement, stereo/timbre/comfort/latency, supported physical
display scales and future feature groups remain unreviewed. Windows/Linux
device checks and unavailable 120/144 Hz contexts remain unrun. Historical
rejected/failed evidence and future-group prerequisites are preserved by the
source issue and #721; none is waived.

No runtime/art changes: renderer [architecture](../../client/renderer/architecture.md)
and [invariants](../../client/renderer/invariants.md) remain authoritative.
