# Issue #49 — physical cargo storage

Cargo membership is derived from existing loose fragment entities and their
ship-local support anchors. The entire conservative physical bound must fit
inside the generated cargo-room bounds. Carried objects, surface objects,
cabin objects and pieces intersecting room boundaries are excluded. No separate
inventory, cached cargo state, capacity, processing or mass effects are added.
Inspection exposes `cargo.fragments` (identity/source/material/mass/volume/local
pose), `cargo.fragment_count`, room bounds and `world.fragments[].in_cargo_room`.
The cargo list scans the authoritative session entities, independent of the
player's presentation/streaming radius. Pickup removes membership immediately;
ordinary G placement on clear cargo floor restores it.

## Validation — 2026-10-01

- Locked workspace build, rustfmt check and Clippy with warnings denied passed.
- All 229 Rust tests passed, including two cargo contracts and the complete
  366-step cargo scenario through production runtime input/update logic.
- All 32 Python tests passed, including baseline/evidence route synchronization.
- The scenario mines a real deposit, transports fragments 1 and 2 across two
  surface-to-room trips, rejects picking up another object while carrying,
  leaves/re-enters the room, removes fragment 1 to the surface, and verifies
  fragment 2 stays at its ship-local anchor through takeoff and 25,000 m/s flight.
  Identity, 2 kg mass and the 19 physical session entities remain preserved.

## Native CI validation — complete

The earlier local display failure was resolved through verification of the
[successful CI run](https://github.com/mr-exception/salimon/actions/runs/36880501864)
for implementation commit `b333321d305eeb70cc42dda9c214718466f25c9c`.
All three platform jobs passed Rust/Python checks and debug/release staging.
Linux passed the required native E2E and real-X11 packaged smoke checks using
Xvfb and Mesa software Vulkan. Native macOS/Windows graphics remain unverified.

Downloaded and inspected Linux artifact `11171825888` (SHA-256
`5d8f7714ad08c113214c0bd8c618aa72ecee675c68766346e0ff43fdb20461ea`).
The baseline passed all 366 steps in 19,896 ms; the evidence variant passed all
371 steps in 28,024 ms, including every required capture. The [validation record](ci-validation.json)
preserves final authoritative states and all screenshot checkpoint states.
Full step snapshots and protocol/process logs remain in the linked CI artifact.

- [First delivery](step-174-first-cargo-fragment.png): cargo contains fragment 1.
- [Two deliveries](step-239-two-cargo-fragments.png): two physical pieces visible on the cargo floor.
- [Removal](step-311-cargo-fragment-removed.png): fragment 1 placed on the surface; fragment 2 remains in cargo.
- [Return](step-333-cargo-after-return.png): room traversal preserves fragment 2. This view faces the room wall; membership is verified by state.
- [Flight](step-371-cargo-during-flight.png): cockpit view after takeoff at 25,000 m/s; state verifies fragment 2 at its unchanged local anchor.

Screenshots are supporting visual evidence; authoritative assertions establish
identity, membership and mass. All acceptance criteria now pass and #49 can close.
The local failed-launch records below are historical evidence, not the final result.

The new baseline runs in the default CI suite. Linux CI also requires the evidence
variant, including first delivery, two pieces, removal, return and flight captures.
The shared runner preserves structured state, protocol/process logs, per-step
snapshots and available failure images under `artifacts/e2e/run-*`; CI uploads them.

```sh
python scripts/salimon-test run scenarios/physical-cargo.json
python scripts/salimon-test run scenarios/evidence/physical-cargo.json \
  --screenshot-command '["python", "scripts/capture_settled.py", "{path}"]'
```

`rust-tests.log`, `clippy.log`, `python-tests.log` and `native-startup.json` record
this session's validation. Native macOS/Windows graphics are not verified here.
