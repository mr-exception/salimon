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

Native graphical E2E remains **blocked in this workspace**. The native runner
failed before readiness because no display was available. Attempting Xvfb also
failed to open Unix listening sockets; TCP display connection failed. No gameplay
assertion or screenshot was reached, and no visual success is claimed. The issue
must remain open until the native baseline and evidence scenarios pass on a
working display. Build/contract success does not establish rendered validation.

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
