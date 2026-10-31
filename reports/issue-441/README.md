# Issue #441 — U01–U33 source/documentation consistency audit

Audited after #432–#434 on parent `3e4e7ef`, before the integrated #435 gate,
in the requested single combined PR. This report has no dependency on #435.

## Current behavior and repairs

Ordinary journeys start on frozen Vesper, retain body-local geography/resource
identities and use the approved versioned fictional two-body motion model.
Compressed six-body worlds, precision tours and representative terrain are
explicit engineering fixtures. The generic split-inertial Trajectory, drag,
moving-contact and observed ephemeris APIs have independent declared inputs and
validity; their existence does not grant observed-body playable travel.

The audit repaired an actual integration mismatch: Nacre sky disk/point lighting
and streaming observer/velocity now consume the same motion source as gravity.
Weather, organism activity and environmental audio use that physical epoch,
including offline time. Personal suit marks retain their existing active-time
metadata clock without silently migrating observations. The night evidence
camera follows the actual anti-stellar direction with evaluated 40 m clearance.
Canonical cockpit previews now clone the motion clock and run the live gravity
operator; missing/invalid motion yields no preview. Source state remains unchanged.

Corrected current root scope, owner guides, gameplay, navigation, contact, drag,
sky, streaming and trajectory contracts, maintenance map and architecture.
The approved-design per-stage implementation notes are explicitly historical;
its current status links this matrix rather than implying delivered U12–U33 work
is pending. Historical reports, golden geography and visual baselines are retained.

## Evidence matrix

Paths name implementation and test owners; renderer files include opt-in GPU
tests. This is a code/documentation mapping, not a claim that every historical
benchmark was rerun. Exact current integrated runs are recorded separately by #435.

| Stage | Behavior | Implementation | Tests/evidence owner | Current contract |
| --- | --- | --- | --- | --- |
| U01 #402 | Catalog/schema | [client/universe/src/records.rs](../../client/universe/src/records.rs) | [client/universe/src/validation.rs](../../client/universe/src/validation.rs) | [client/universe/architecture.md](../../client/universe/architecture.md) |
| U02 #403 | Split coordinates/epochs | [client/universe/src/coordinates.rs](../../client/universe/src/coordinates.rs) | [client/universe/src/coordinates/tests.rs](../../client/universe/src/coordinates/tests.rs) | [client/universe/architecture.md](../../client/universe/architecture.md) |
| U03 #404 | Independent reference harness | [client/universe/src/reference.rs](../../client/universe/src/reference.rs) | [client/universe/src/reference/tests.rs](../../client/universe/src/reference/tests.rs) | [docs/universe-reference-fixtures.md](../../docs/universe-reference-fixtures.md) |
| U04 #405 | Partition codec | [client/universe/src/partitions.rs](../../client/universe/src/partitions.rs) | [client/universe/src/partitions/tests.rs](../../client/universe/src/partitions/tests.rs) | [docs/universe-storage.md](../../docs/universe-storage.md) |
| U05 #406 | Catalog save binding | [client/persistence/src/universe.rs](../../client/persistence/src/universe.rs) | [client/persistence/src/universe/tests.rs](../../client/persistence/src/universe/tests.rs) | [docs/save-contract.md](../../docs/save-contract.md) |
| U06 #407 | Frozen source metadata | [client/universe/src/sources.rs](../../client/universe/src/sources.rs) | [scripts/test_import_universe_sources.py](../../scripts/test_import_universe_sources.py) | [docs/universe-sources.md](../../docs/universe-sources.md) |
| U07 #408 | Gaia ingestion | [client/universe/src/ingestion/gaia.rs](../../client/universe/src/ingestion/gaia.rs) | [client/universe/src/ingestion/gaia/tests.rs](../../client/universe/src/ingestion/gaia/tests.rs) | [docs/universe-gaia-ingestion.md](../../docs/universe-gaia-ingestion.md) |
| U08 #409 | JPL/PS ingestion | [client/universe/src/ingestion/horizons.rs](../../client/universe/src/ingestion/horizons.rs) | [client/universe/src/ingestion/horizons/tests.rs](../../client/universe/src/ingestion/horizons/tests.rs) | [docs/universe-observed-ingestion.md](../../docs/universe-observed-ingestion.md) |
| U09 #410 | Observed missing-field inference | [client/universe/src/inference.rs](../../client/universe/src/inference.rs) | [client/universe/src/inference/tests.rs](../../client/universe/src/inference/tests.rs) | [docs/universe-inference.md](../../docs/universe-inference.md) |
| U10 #411 | Fictional builder | [client/universe/src/synthesis.rs](../../client/universe/src/synthesis.rs) | [client/universe/src/synthesis/tests.rs](../../client/universe/src/synthesis/tests.rs) | [docs/universe-fiction-builder.md](../../docs/universe-fiction-builder.md) |
| U11 #412 | Frozen awakening world | [client/universe/src/starting_system.rs](../../client/universe/src/starting_system.rs) | [client/universe/src/starting_system/tests.rs](../../client/universe/src/starting_system/tests.rs) | [docs/universe-starting-system.md](../../docs/universe-starting-system.md) |
| U12 #413 | Versioned pack publication | [client/universe/src/pack.rs](../../client/universe/src/pack.rs) | [client/universe/src/pack/tests.rs](../../client/universe/src/pack/tests.rs) | [docs/universe-pack.md](../../docs/universe-pack.md) |
| U13 #414 | Bounded reader | [client/universe/src/reader.rs](../../client/universe/src/reader.rs) | [client/universe/src/reader/tests.rs](../../client/universe/src/reader/tests.rs) | [docs/universe-reader.md](../../docs/universe-reader.md) |
| U14 #415 | Time-addressable states | [client/universe/src/ephemerides.rs](../../client/universe/src/ephemerides.rs) | [client/universe/src/ephemerides/tests.rs](../../client/universe/src/ephemerides/tests.rs) | [docs/universe-ephemerides.md](../../docs/universe-ephemerides.md) |
| U15 #416 | Dominant gravity | [client/universe/src/gravity.rs](../../client/universe/src/gravity.rs) | [client/universe/src/gravity/tests.rs](../../client/universe/src/gravity/tests.rs) | [docs/universe-gravity.md](../../docs/universe-gravity.md) |
| U16 #417 | Split-inertial trajectory | [client/ship/src/trajectory.rs](../../client/ship/src/trajectory.rs) | [client/ship/src/trajectory/tests.rs](../../client/ship/src/trajectory/tests.rs) | [docs/universe-trajectory.md](../../docs/universe-trajectory.md) |
| U17 #418 | Funded surface flight | [client/ship/src/surface/mod.rs](../../client/ship/src/surface/mod.rs) | [client/ship/src/surface/tests.rs](../../client/ship/src/surface/tests.rs) | [docs/universe-surface-flight.md](../../docs/universe-surface-flight.md) |
| U18 #419 | Atmospheric drag | [client/ship/src/atmosphere/mod.rs](../../client/ship/src/atmosphere/mod.rs) | [client/ship/src/atmosphere/tests.rs](../../client/ship/src/atmosphere/tests.rs) | [docs/universe-atmosphere.md](../../docs/universe-atmosphere.md) |
| U19 #420 | Moving continuous contact | [client/ship/src/contact/mod.rs](../../client/ship/src/contact/mod.rs) | [client/ship/src/contact/tests.rs](../../client/ship/src/contact/tests.rs) | [docs/universe-contact.md](../../docs/universe-contact.md) |
| U20 #421 | Navigation observations | [client/ship/src/prediction.rs](../../client/ship/src/prediction.rs) | [client/ship/src/preview.rs](../../client/ship/src/preview.rs) | [docs/universe-navigation.md](../../docs/universe-navigation.md) |
| U21 #422 | Predictive streaming | [client/universe/src/streaming.rs](../../client/universe/src/streaming.rs) | [client/universe/src/streaming/tests.rs](../../client/universe/src/streaming/tests.rs) | [docs/universe-streaming.md](../../docs/universe-streaming.md) |
| U22 #423 | Hybrid sky | [client/runtime/src/app/sky.rs](../../client/runtime/src/app/sky.rs) | [client/renderer/src/sky.rs](../../client/renderer/src/sky.rs) | [docs/universe-sky.md](../../docs/universe-sky.md) |
| U23 #424 | Canonical terrain LOD | [client/runtime/src/terrain_patches.rs](../../client/runtime/src/terrain_patches.rs) | [client/runtime/src/terrain_patches.rs](../../client/runtime/src/terrain_patches.rs) | [docs/universe-terrain.md](../../docs/universe-terrain.md) |
| U24 #425 | Canonical surface integration | [client/runtime/src/session/canonical.rs](../../client/runtime/src/session/canonical.rs) | [client/runtime/src/session/canonical.rs](../../client/runtime/src/session/canonical.rs) | [docs/universe-gameplay.md](../../docs/universe-gameplay.md) |
| U25 #426 | New awakening journey | [client/runtime/src/session/canonical.rs](../../client/runtime/src/session/canonical.rs) | [client/runtime/src/session/starting_supplies_tests.rs](../../client/runtime/src/session/starting_supplies_tests.rs) | [docs/universe-gameplay.md](../../docs/universe-gameplay.md) |
| U26 #427 | Ordinary-play fixture exclusion | [client/runtime/src/checkpoints.rs](../../client/runtime/src/checkpoints.rs) | [client/runtime/src/app/scene.rs](../../client/runtime/src/app/scene.rs) | [docs/universe-gameplay.md](../../docs/universe-gameplay.md) |
| U27 #428 | Contacts/history authority | [client/universe/src/scanner.rs](../../client/universe/src/scanner.rs) | [client/universe/src/scanner/tests.rs](../../client/universe/src/scanner/tests.rs) | [docs/universe-scanner.md](../../docs/universe-scanner.md) |
| U28 #429 | Five-second funded pulse | [client/runtime/src/session/scanner.rs](../../client/runtime/src/session/scanner.rs) | [client/runtime/src/session/scanner/integration_tests.rs](../../client/runtime/src/session/scanner/integration_tests.rs) | [docs/universe-scanner.md](../../docs/universe-scanner.md) |
| U29 #430 | Cockpit scanner display | [client/renderer/src/scanner_hologram.rs](../../client/renderer/src/scanner_hologram.rs) | [client/renderer/src/scanner_readout.rs](../../client/renderer/src/scanner_readout.rs) | [docs/universe-scanner.md](../../docs/universe-scanner.md) |
| U30 #431 | Scanner persistence | [client/runtime/src/session/scanner/save.rs](../../client/runtime/src/session/scanner/save.rs) | [client/universe/src/scanner/checkpoint.rs](../../client/universe/src/scanner/checkpoint.rs) | [docs/universe-scanner.md](../../docs/universe-scanner.md) |
| U31 #432 | Canonical closed trajectory | [client/ship/src/controller/celestial.rs](../../client/ship/src/controller/celestial.rs) | [client/runtime/src/session/offline/motion_tests.rs](../../client/runtime/src/session/offline/motion_tests.rs) | [docs/universe-offline-propagation.md](../../docs/universe-offline-propagation.md) |
| U32 #433 | Chronological timed work | [client/runtime/src/session/offline.rs](../../client/runtime/src/session/offline.rs) | [client/runtime/src/session/scanner/offline_tests.rs](../../client/runtime/src/session/scanner/offline_tests.rs) | [docs/universe-offline-propagation.md](../../docs/universe-offline-propagation.md) |
| U33 #434 | Verified async resume | [client/runtime/src/launch.rs](../../client/runtime/src/launch.rs) | [scenarios/evidence/offline-resume-progress.json](../../scenarios/evidence/offline-resume-progress.json) | [docs/universe-offline-propagation.md](../../docs/universe-offline-propagation.md) |


## Audit verification

Linux cloud, Rust/Cargo 1.99.1, locked debug profile. Focused checks: canonical
runtime **20 passed**, preview filter **4 passed**, moving night-camera regression
**1 passed**, affected runtime/ship all-target Clippy with `-D warnings` **passed**.
The new tests compare preview endpoints to actual celestial propagation, verify
source nonmutation, assert sky/light direction at reference/current epochs and
check terrain clearance/anti-stellar orientation at three dates. Environment
epoch tests preserve the distinction from personal active time. Workspace gates,
original production GPU captures and combined numerical/performance evidence
are deliberately run once after this audit by #435.

## Explicit limits and remaining owners

- Vesper is airless and sterile. Drag/biosphere positive fixtures are not a
  claim of atmosphere/life on this publication. No stellar-interior response
  is invented: unsupported coasting remains unverified and blocks publication.
- Existing crash/rebound/grounding and committed funded-assist policies apply;
  no new fatal-impact threshold or automatic safe landing is invented.
- TT/TDB are not mixed. Observed ephemeris validity and unknown source fields
  remain explicit; optional moving sky sources need a compatible TT epoch.
- Frozen JPL/PS selected coverage is implemented by #526. Private source usage
  is owner-authorized; release reuse/rights remain #659. Full Gaia acceptance
  remains #524, not a completed whole-Galaxy scientific/budget certification.
- Player-known charts/history survive contact expiration; scanner contacts
  expire after three hours and pulses occur once per five seconds of funded
  clock. Catalog/sky visibility grants no discovery/Earth route/Jump Drive.
- Physical OS/device/platform qualification #557, presentation/audio #561,
  hardware/M1/Metal budgets #563 and balance #660/#661 remain separately
  unverified. No source/model blocker is hidden behind those human follow-ups.

No broken path or unresolved current-model contradiction is intentionally
deferred to #435. Documentation checks validate local links and same-PR owner
coverage; the model remains explicitly limited as above.


## Shared batch acceptance

Subsequent implementation and final cloud evidence are complete in the same
combined PR: [integrated results, original media and limits](../issue-435/README.md).
That final index supersedes earlier pending-batch statements while preserving
historical focused-check revisions/counts above. No merge or hardware acceptance
is implied. The audit precedes integrated verification; this link is additional
evidence, not a dependency from #441 to #435.
