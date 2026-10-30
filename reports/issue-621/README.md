# Issue #621 — biological part transfer and embedded harvesting

Parent: PR #709, `issue-627-environmental-audio` at
`b6de46a31d70663ab32ac50e68b75412214e8b5b`.
Environment: cloud Linux x86_64, Rust/Cargo 1.99.1, Python 3.12.14,
production wgpu Xvfb/Mesa lavapipe, fixed 16 ms automation ticks.

## Result and source mapping

Explicit source designations bind existing raw-material yields to one immutable
physical organism/part. F grips, briefly tensions and atomically transfers that
same natural part into the existing one-object carry slot. Pre-commit focus,
pause, modal, aim or reach loss cancels intact. F drops the same object. Natural
shape, color/species material, source/part ID, mass and physical fragment handle
survive drop and save/reopen. Embedded reserves reuse the original tool,
spherical cap/rate/interrupt and physical output. Revision checks prevent stale
future-authority requests from duplicating source material.

Missing/partial geometry, static clearance, sound eligibility and personal-site
inspection consume current anatomy. Schema 29 validates publication, deltas,
shape, material, source debit, historical handles, carry and allocator before
atomic replacement; versions 1–28 migrate empty. Pending grip/audio/caches are
excluded. No regrowth, new tool, abstract inventory grant, fauna, terrain
destruction, automatic objective or new scanner timing/identification is added.

| Behavior | Production owner / evidence | Contract |
| --- | --- | --- |
| Explicit yields, revision/debit, one-object publication | world `environment/harvest.rs`, `biological_session.rs`, harvest tests | [harvesting](../../docs/biological-harvesting.md) |
| F/tool gates, current anatomy and checkpoint transaction | runtime `session/biology.rs`, focused tests, persistence migrations | same guide |
| Original tapered shape and bounded collision | `resources::NaturalShape`, runtime `app/organisms.rs`, `fragment_physics.rs`, carrying/geometry regression | [physical organisms](../../docs/physical-organisms.md) |
| Actual contact audio and removed-part rustle rejection | `scout_audio`, `environment_audio` tests and exported PCM | [environmental audio](../../docs/planetary-audio.md) |

Documentation impact: updated all affected world/runtime/persistence AI,
architecture and invariant guides, canonical resource/save contracts, root
architecture/maintenance/validation, physical and rendered organism contracts,
approved art-direction implementation note and runner guide. Save contract's
obsolete current-version header is corrected; historical migrations remain.

## Canonical world and explicit positive fixture

**Vesper remains frozen, sterile and without harvest sources.** Positive evidence
uses `environment::harvest_fixture::publish`: an explicitly fictional biosphere
on existing representative Moon geometry, with one specifically designated
biomass appendage and a different medicinal reserve. It does not claim actual
Moon life/medicine or alter observed/canonical publications. Initial position and
aim are fixtures; every subsequent grip, cancellation, mining step, drop and
reload uses production commands/controllers. No new world selector/catalog-body
adoption or physical keyboard walkthrough is claimed.

[Source context](source-context.json) records biosphere/population/catalog hashes,
population binding digest, body origin, simulation time, original anatomy, exact
source/part keys and material/mass. The fixture exercises coordinates near 1e12 m;
the renderer retains subtract-before-f32 precision. The 17.687024760319485 kg
biomass appendage retains its 0.6645750652129443 m original conservative radius;
the separate embedded reserve starts at 32.99725863918051 kg. These are effective
gameplay material densities, not scientific pharmacology or biological measurements.

## Verification commands and results

Commands source the workspace Rust/ALSA/Mesa environment. Final results and
original native artifacts are recorded below.

- `cargo test --workspace --locked --offline`
- `cargo test -p salimon-client session::biology --locked --offline`
- `cargo test -p salimon-client harvested_natural --locked --offline`
- `cargo test -p salimon-world environment::harvest --locked --offline`
- `cargo test -p salimon-client environment_audio --locked --offline`
- `cargo build --workspace --locked --offline`
- `cargo clippy --workspace --all-targets --locked --offline -- -D warnings`
- `cargo fmt --all -- --check`, `git diff --check`
- `python3 scripts/check_docs.py --base issue-627-environmental-audio`
- `python3 -m unittest discover -s scripts -p 'test_*.py'`
- `SALIMON_AUDIO_EVIDENCE=$PWD/reports/issue-621 cargo test -p salimon-client biological_contact_audio_evidence_621 --locked --offline -- --ignored`
- `scripts/salimon-test run scenarios/evidence/biological-harvest.json --binary target/debug/salimon-client --screenshot-command '["native"]' --summary --artifacts artifacts/issue-621-harvest` under Xvfb/lavapipe; equivalent biological-mining command for embedded material.

Full workspace rerun: **1,241 tests passed, 0 failed, 51 opt-in tests ignored**.
Final focused biology checks: **6 passed**, including later modal-priority and
immediate restored-collision admission. World source checks: 3 passed. Original
natural-geometry mapping: 1 passed. Environmental checks: 6 passed (1 opt-in
export ignored). Actual separation PCM export: 1 passed. Python: 136 passed.
Build, Clippy warnings denied, format and diff checks passed. Opt-in hardware
and benchmark cases are not counted as passes.

The final old-save tick comparison adds only the authorized empty biological
publication and schema version, preserving every other frozen source byte.
Tests cover stale revisions, oversize/occupied source rejection, exact mass within
existing 1e-9 conservation checks, caps, interruption/reopening, missing anatomy,
malformed cross-owner source/shape/handle/debit and unchanged live state on error.

## Listening evidence and release limits

[Actual separation WAV](actual-separation.wav) is original production continuous
callback PCM from 125 real 16 ms gameplay observations (2 s / 96,000 stereo frames
at 48 kHz), including the fixture's actual weather/material context and successful
separation contact. [Metrics](audio-metrics.json) are shared cloud CPU/peak/storage
measurements, not a device recording, hardware latency or subjective listening
approval. Reload is checked not to replay the contact. #561 owns physical
listening/tuning; #557/#563 retain physical platform/input and M1/Metal performance.

Natural loose-object/terrain support is a conservative circumsphere; static
anatomy and held clearance use transformed tapered-part bounds. This preserves
shape visually without promising exact convex tissue contact or soft-body motion.
Provisional natural carry limits are 0.75 m radius / 20 kg. Existing mineral
geometry/caps remain unchanged. Future multiplayer is not implemented; expected
revision checks define the single-authority transaction boundary.

## Corrections and failed attempts

Initial compile checks caught test-only serialization assumptions (world has no
serde_json dependency), a moved helper module path, argument grouping and typed
reticle mapping. Fixed before passing verification. The geometry regression
caught a conservative carried-body clearance issue at separation; final held
admission checks transformed anatomy, clear end pose and sight path, with explicit
2.8 m fixture observer clearance. It now verifies exact original shape at the
physical hand pose and absence of an attached clone.

The first full run found an older byte-for-byte tick fixture that omitted the
new empty journal. Its expectation now adds only that authorized field/version;
the rerun passed. An initial native embedded route requested 1,000 frames in one
command, beyond the protocol's existing 600 limit; corrected. A subsequent
capture inspection hit the adapter's fixed five-second request timeout while
the full suite was busy. That failed attempt is not a pass; final native results
are recorded separately. No numerical or physical tolerance was widened.

## Final production native evidence

- **Harvesting: 44/44 steps passed**, 194.300 s: attached source, visible grip,
  pause cancellation, original appendage in hand, missing part, drop and reopen.
- **Embedded mining: 30/30 steps passed**, 76.456 s: actual equipped tool, partial
  source geometry, interruption/reopen, 10 kg capped physical output and persistent
  partial state. Final physical masses are 0.1279999999999859, 10.0 and
  2.7839999999996934 kg (IDs 1/2/3), totaling the actual 12.91199999999968 kg debit.
  Remaining source fraction is 0.6086947663989232. Full embedded depletion/cap
  repetition is covered by the controlled world lifecycle regression, not claimed
  as a single uninterrupted native ray through physical output.

Original 1920×1080 wgpu images: [attached](harvest-step-006-attached-biomass-part.png),
[grip](harvest-step-013-gripping-source.png),
[same original part in hand](harvest-step-029-same-natural-part-in-hand.png),
[released object](harvest-step-041-released-natural-part.png),
[embedded reserve](mining-step-006-attached-medicinal-reserve.png),
[partial](mining-step-017-partial-depletion.png),
[cap output and persistent partial source](mining-step-026-cap-release-and-partial-source.png).
Final originals were visually inspected. Complete compressed result records retain every step/state and performance
observation. Compact command traces omit repetitive read-only inspect polling;
[provenance](evidence-provenance.json) retains original protocol digests and sizes.
Duplicated stdout response streams remain disposable workspace artifacts.

The isolated native rerun completed all capture operations within the existing
request timeout. Its last cap assertion initially referred to ID 1, the earlier
0.128 kg interrupted output; the correct full capped object is ID 2. The final
route checks that original identity and its exact cap. This was an evidence
index correction, not a capacity or numerical tolerance change.

Documentation gate: 0 failures, 0 retained historical findings. Final diff/format
checks passed. Full workspace logs, final focused checks and command outputs
are retained compressed beside this report. The six focused checks reran after the final carry-restriction and immediate
restored-collider guards.
