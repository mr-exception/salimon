# #541 — Portable 3D structural foundation

## Outcome

Implemented `salimon_ship::structure`, an independent authoritative domain
boundary for the approved 3 m ship-local building grid. It is not yet composed
into `GameplaySession`, the current Scout asset or native presentation. The
current playable flight, physical cargo, Core fueling/recovery and player save
schema are unchanged.

## Changes and ownership

- `client/ship/src/structure/grid.rs`: typed X/Y/Z cells, six cardinal neighbors,
  checked integer boundaries, precise explicit `f64` origin and position/envelope
  queries. No authored anchor rounding or mandatory fine-snap spacing.
- `structure/mod.rs`: sparse stable-ID parts, frame/full/tapered/corner geometry
  hints, shared face ownership, floor/ceiling/doorway/opening metadata, multi-cell
  room aggregates and equipment footprints. Supporting structure, room grouping,
  equipment occupancy and future traversal/power/pressure are distinct.
- Connected admission and safe deliberate removal; transactional damage partitions
  into Core-attached state and explicit detached components. Unsupported equipment
  and invalidated room metadata are returned to the future salvage owner. IDs
  remain stable and retired parent IDs cannot be reused.
- Baseline `IncludedInDryHull` versus `AddedKilograms` mass is explicit. Added
  assemblies/equipment emit disjoint `MassObjectId::Structure` entries exactly
  once; rooms/faces do not add mass or duplicate existing cargo/legacy equipment.
- `structure/persistence.rs`: independent schema 1 saves authoritative data and
  retired IDs; validated restore rebuilds indexes without assuming ID/topological
  order. Whole-game schema 7 and existing ship/controller saves are unchanged.
- Owned ship README/AI guide/architecture/invariants and root routing, approved
  design, save and validation guides describe the implemented boundary and later
  integration accurately.

Canonical ownership: [ship architecture](../../client/ship/architecture.md#portable-structural-foundation-541),
[ship invariants](../../client/ship/invariants.md#portable-structure-541),
[approved design](../../docs/scout-and-modular-construction.md).

## Verification

Environment: Linux cloud; Rust/Cargo 1.99.1, Python 3.12.14. Commands run from
repository root after sourcing `/workspace/.salimon-tools/env.sh`, with shared
`CARGO_TARGET_DIR=/workspace/salimon/target`. Locked dependencies were restored
with `cargo fetch --locked`; subsequent checks used `--offline`.

- `cargo test -p salimon-ship --locked --offline structure::tests`: **11 passed**.
  Covers all six attachments vs edges/corners, negative/extreme/malformed
  coordinates, explicit origin precision, 55-cell multi-deck footprint, room and
  equipment ownership, shared face/floor/ceiling changes, normal bridge removal,
  damage transfers/mass conservation, duplicate IDs, atomic invalid inputs,
  baseline versus added mass, domain envelope round-trip and corrupted saves.
  An independent Manhattan-closure oracle checks every connected subset of a
  2×2×2 grid and more than 200 normal/damage cut cases.
- `cargo test --workspace --locked --offline`: **860 passed, 30 existing opt-in tests ignored**. This run preceded the final test-only exhaustive cut oracle; all 11 structural tests and the full ship suite passed again afterward.
- `cargo test -p salimon-ship --locked --offline`: **84 passed, one existing opt-in ignored**, at final source/test revision.
- `cargo build --workspace --locked --offline`: **passed**.
- `cargo clippy --workspace --all-targets --locked --offline -- -D warnings`:
  **passed** at final source/test revision.
- `cargo fmt --all -- --check`: **passed**.
- `python3 scripts/check_docs.py`: **passed**, zero failures/historical debt;
  report included; same-PR base check also passed.
- `git diff --check`: **passed**.

The first implementation compile exposed one stale field reference during the
mass-contract refactor; it was fixed before passing checks. No failed check is
represented as a pass. Existing opt-in/ignored GPU/reference/performance checks
are not newly executed by the portable workspace suite.

## Limits and existing follow-on owners

The foundation has no platform/GPU dependency or live player flow, so visual
screenshots and reference-machine performance are not acceptance evidence for
this change. No native/Metal/M1/human approval is claimed.

No unfinished #541 acceptance is transferred. Existing later issues retain
placement/preview admission (#542), physical material/resource and mass transfer
(#543), work progress (#544), generated room geometry (#545), flight-readiness
policy (#546), module installs (#547), physical damage/salvage (#549) and session/
disk construction persistence (#550). Envelope bounds are conservative metadata,
not final nonrectangular mesh collision admission. Equipment footprints currently
reserve whole cells; finer furniture snapping is not approved by this issue.
Detachment returns durable source data but does not spawn/simulate a wreck or
change the starter dry-mass model on its own.
