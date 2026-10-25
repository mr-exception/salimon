# #545 — atomic room assembly groundwork (incomplete feature)

Base: `58c62ebc945487fac90f5a731de287999b5080d0` (main).
Environment: Linux x86_64, rustc 1.99.1 (58cf8f9c3 2026-10-13).
Commands source `/workspace/.salimon-tools/env.sh` first.

## Delivered

`ShipStructure::install_room(RoomAssembly)` stages all cell parts and room
membership in a private graph, publishing once after validation. One part per
cell preserves independent identity, face metadata, supplied collision and mass;
the room grouping adds no mass. Caller order need not be support order. Failures
leave topology, mass and identity availability unchanged.

Four-cell/two-level tests exercise exact serialization roundtrip, independent
face editing/damage, replay rejection, disconnected/corner-only support, late
invalid mass/collision/faces, overflow, occupied cells and retired identities.
The fixture supplies floor collision explicitly; it is not authored room geometry
or evidence of traversable decks. Metadata editing does not regenerate collision.

## Scope still outstanding in #545

This change is **not completion of #545** and does not unblock #546/#547/#549.
It is not yet called by the runtime. Structural jobs still stop just before
completion, as in #544. Remaining work includes:

- Blueprint-to-cell shell generation, doors/openings, tapered/clipped hull detail
  and actual stairs/hatches or other safe multi-deck connectors.
- Runtime final preflight/atomic commit, authoritative completed structure and
  identity allocation, mass ownership, save restoration and expansion support.
- Character/cargo traversal and collision integration, cell/partition editing,
  and production wgpu captures of the resulting playable construction loop.

No feature acceptance, renderer appearance, device input, Metal compatibility or
hardware performance is claimed. Human presentation/device/budget review remains
#561/#557/#563. Keep #545 open; do not add a closing reference to this PR.

## Checks

- `cargo test -p salimon-ship --locked structure`: initial 26 tests passed.
- `cargo test -p salimon-ship --locked`: initial 105 passed, one pre-existing ignored.
- `cargo clippy -p salimon-ship --all-targets --locked -- -D warnings`: passed.
- `cargo fmt --all -- --check`: passed.
- `python scripts/check_docs.py`: passed, zero findings.
- Initial workspace test/build attempts failed during ALSA discovery because the
  executor lacked `alsa.pc`. Downloaded and extracted Debian libasound2-dev and
  libasound2t64 into the existing task sysroot; rerun results follow below.

Retained setup failure: [initial workspace log](workspace-initial-alsa-failure.log).
Ship static analysis: [Clippy log](ship-clippy.log).

Final reruns after all source edits:

- `cargo test -p salimon-ship --locked`: 106 passed, one pre-existing ignored;
  [log](ship-tests.log).
- `cargo test --workspace --locked`: passed; [complete log](workspace-tests.log)
  preserves per-suite counts and opt-in ignored tests.
- `cargo build --workspace --locked`: passed; [log](workspace-build.log).
- `cargo clippy --workspace --all-targets --locked -- -D warnings`: passed;
  [log](workspace-clippy.log).
- `cargo fmt --all -- --check`, `python scripts/check_docs.py`, and
  `git diff --check`: passed.

Graphical scenarios were not run: this PR changes only portable graph admission
and does not implement or claim the live #545 feature. Production renderer and
playable multi-deck acceptance remain required before #545 completion.
