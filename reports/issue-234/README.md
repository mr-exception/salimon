# Issue 234 — spherical release, carry and Core integration

One bounded mass/density radius now drives visual/contact dimensions, pickup
rays, carry/terrain clearance, drop containment, prompt placement and benchmark
support. `side_meters` means sphere diameter. Legacy ID-parity hull selection
is removed; original GLBs remain for explicit historical geometry comparisons.
Sphere attachment includes current camera orientation; release keeps the same
reserved identity and mass and falls under ordinary gravity. Preview mass never
participates in loose physics/pickup. Mining remains free; Core intake/conversion
keeps ship-owned yields, power policy and mechanical blackout access.

Portable integrated test covers all three resources at 10 kg and 0.25 kg:
held extraction and preview exclusion → release → production F pickup/carry/drop
→ ship anchoring → reacquisition → mechanical E Core intake → energy restoration
and stable-ID cleanup → stale deposit restoration without regeneration.
Ray tests reject old cube corners and preserve rotation independence. Carry
clearance tests include the new 10 kg cap. Fixture size expectations are derived
from spherical volume; transfer resting checkpoints use measured current poses
and retain the existing 2 cm tolerance.

Compatibility: Phase 0 has no supported disk save format. Existing in-memory
session IDs/source/material/mass/poses survive streaming. All session fragments
render and collide as spheres regardless of prior shard parity. Unknown material
keys still fail validation. No speculative save parser or backend is introduced.

Validation and environment details follow in the PR. Native M1 signoff remains
#236. No native rendering/performance is claimed by portable CPU execution.

## Validation

Linux x86-64, Rust 1.89.0. Workspace test compilation and runtime, character,
physics, renderer, ship, math and diagnostics tests passed. The world test
executable initially lost its execute bit in the cloud filesystem; after
restoring that bit, all world and integration tests passed. A resumed-session
rerun of the retained executables confirms the complete **474-test** base stack
(11 opt-in ignores), including 146 runtime tests and 40 world unit tests.
Commands: `cargo +1.89.0 test --workspace --locked`, followed by direct test
executable reruns in `target/debug/deps/`. Local artifact workarounds used
`CARGO_INCREMENTAL=0`, single codegen units and no debug symbols; repository
build settings were not changed. Workspace Clippy with `-D warnings`, rustfmt
and diff checks passed. Graphical/native checks were not run.
