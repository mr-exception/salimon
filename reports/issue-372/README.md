# Issue #372 — persisted Journey statistics

Implemented on 2026-10-20 from main `c3c5d56ca33f36b30a61b0b683616d6222b5077c`.

The session records combined physical ship travel and character travel, exact
active survival duration, framed last valid positions, and final named location.
Death reports preserve the survival owner's typed cause. Ship-local cabin motion
prevents passenger transport/rotation from counting twice. Outside/EVA and frame
crossings use physical f64 endpoints; telemetry/camera/render origins never supply
travel. Commands/teleports/restore cannot bridge previous samples into distance.
Physical assist paths count during admitted ticks.

See the [canonical metric and migration contract](../../docs/save-contract.md#journey-statistics-372)
for ownership, sample frames, nearest-body names and the precise combined metric.
The independently moving ship and EVA player each contribute their own path.
Distance is the sum of tick chords; it is not net displacement or analytic arc length.
Active duration uses survival authority rather than legacy world elapsed time.

New full saves use schema 3. V1 physical state survives the existing fresh-survival
migration; v2 retains its distance counter and initializes empty sampling history.
V3 missing tracking source rejects transactionally. Closed time adds neither
travel nor alive time. Fresh runs reset all journey statistics.

The native host continues bounded ticks/checkpoints while occluded, minimized or
zero-sized, clearing input and suppressing presentation. Focus loss continues time.
OS suspension and renderer reconstruction retain explicit lifecycle gap exclusion.

The save worker now archives at death, not only Restart, after both terminal slots
verify. Launch recovery repairs a missing report. Version-2 summaries include
named nearest-body/space context and use `journeys/v2/run-<serial>.json`; old v1
reports remain immutable in `journeys/`. Both namespaces reserve run IDs.
Restart verifies the same archive before publishing both fresh generations.

## Validation

Environment: Linux cloud container, Rust 1.99.1, Python 3.12.14.
[Validation summaries](validation.txt) retain the exact commands and outcomes.

- `cargo test --workspace --locked`: **645 passed**, 0 failed, 22 existing opt-in ignored.
  Includes production protocol/session E2E contracts, world/ship/character, whole-save
  migration/continuation and persistence fault injection.
- `cargo build --workspace --locked`: passed.
- `cargo clippy --workspace --all-targets --locked -- -D warnings`: passed.
- `cargo fmt --all -- --check`: passed.
- `python3 -m unittest discover -s scripts -p 'test_*.py'`: **93 passed**.
- `git diff --check` and changed-document local link checks: passed.

Twelve new regressions cover solar-coordinate cabin precision and passenger motion,
doorway/frame crossings, separate EVA travel, invalid/inactive samples, real cabin
walking, inherited EVA drift, flight/offline restore/teleport/precision tour,
assisted takeoff, v2/v3 migration validation, archive location/duration/typed cause,
background/resume timing, death publication/launch repair, and immutable versioned
archives/high-water IDs. Existing lethal-boundary/freeze/restart and four-stage
publication fault regressions remain green.

## Native limitation and follow-up

A direct launch of the built binary **failed before window creation**, because
this environment has no DISPLAY/WAYLAND_DISPLAY/WAYLAND_SOCKET. See
[native launch result](native-launch-result.json) and [stderr](native-launch.log).
Native OS occlusion/minimize input delivery, graphical Journey presentation and
reference Apple M1 signoff were not run or claimed. The existing
[#376 native survival signoff](https://github.com/mr-exception/salimon/issues/376)
owns that validation; [#374](https://github.com/mr-exception/salimon/issues/374)
owns the complete Journey Ended screen. Portable functionality and gates above
are delivered here; native signoff remains pending in its existing follow-up.
