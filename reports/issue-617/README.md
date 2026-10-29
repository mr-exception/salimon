# Issue #617 — canonical active geological source interfaces

Parent: non-draft PR #689, branch `issue-615-regional-weather`. Its #615 climate
implementation, merged #612 geology and #611 design provide the required code.
The parent's subsequent ship-guide-only CI correction is retained in ancestry.
This child does not duplicate climate implementation in its PR diff.

World now owns explicit mechanism/heat/duty inputs, bounded deterministic
candidate placement, stable terrain-bound source IDs and continuous shared-TT
activity/plume/thermal/acoustic events. Zero heat produces no sites; unsuitable
geology fails closed. Site radii are emitter bounds, never new physical colliders.
Vacuum uses ballistic emission and contact-only coupling, atmospheric sources
consume the actual climate wind, and seismic sources have no plume.

Strict codec/compatibility checks bind the complete upstream climate/geology and
reconstruct sources without replay, terrain changes or duplicate IDs after
stream/restore. Vesper's original publication stays unchanged and dormant.
The [canonical contract](../../docs/active-geology.md), owning world guides,
architecture, maintenance map, validation/root routing and approved direction
are updated in the same PR.

## Validation

Debian 13 Linux x86_64, Rust/Cargo 1.99.1. Commands source the existing
`/workspace/.salimon-tools/env.sh` development/toolchain environment repaired by
#615; no additional setup failures occurred.

- `cargo test -p salimon-world --locked environment::activity`: six focused tests passed.
- `cargo test -p salimon-world -p salimon-ship --locked`: **242 passed, zero failures,
  three existing opt-in ignored**. Covers physical-surface agreement, bounded
  unique spatial selection, exact restore, periodic and offline/partition queries,
  vacuum/atmosphere separation, geological suitability, malformed data and migration.
- `cargo clippy --workspace --all-targets --locked -- -D warnings`: passed.
- `cargo build --workspace --locked`: passed.
- `cargo fmt --all -- --check`: passed.
- `python3 scripts/check_docs.py --base issue-615-regional-weather`: passed after commit, zero failures.

[Raw affected tests](tests.log), [Clippy](clippy.log), [workspace build](build.log).
The full unchanged runtime/tooling baseline is retained in the [parent's report](../issue-615/README.md):
1,152 workspace tests and 134 tooling tests passed there. This child reruns the
changed world owner and its ship consumer plus workspace compile/lint; it does
not claim the unchanged full runtime matrix was rerun on this revision.

## Scope and limits

These are production portable source/selection interfaces, not a new terrain,
hazard, resource or particle/audio system. No runtime renderer, collision producer
or audio mixer changes occur, so graphical capture is not applicable. Source
presentation/audio consumers retain their owners (#627 for environmental sound).
No inferred active geology on real planets, permanent terrain deformation,
physical sound-in-vacuum, resource creation or ship/Core energy changes are added.
Human subjective/physical/hardware criteria remain separately unverified under
#561/#557/#563; no native/M1/Metal claim follows from cloud tests.
