# Issue #368 — typed damage, healing and status execution

Implemented one deterministic damage pipeline for starvation and external/periodic
sources, fixed-order armor/resistance/amplification/caps, ordinary/treatment
healing, wound debt, conditional 60-second regeneration, tagged future sources,
and data-driven timed/persistent/exposure status rules with bounded stacking,
refresh, linger, dispel and treatment-only removal. Lethal timestamp ordering and
cause selection are explicit. [Contract](../../client/survival/architecture.md#damage-and-status-pipeline-368).

Validation on Linux, 2026-10-19:
- `cargo test -p salimon-survival --locked`: PASS, 14 integration tests.
- `cargo clippy -p salimon-survival --all-targets --locked -- -D warnings`: PASS.
- `cargo fmt --all`: applied; complete stack gates recorded after integration.

Tests cover simultaneous lethal damage/healing, starvation/heal competition,
regen thresholds, modifier arithmetic, source-specific debuffs, wound treatment,
stack caps/refresh, exact final expiry tick, idempotent exposure, radiation exit,
burn linger, offline timer freeze, split-time and fractional save continuation,
registered sources, invalid registries and corrupted active status state.

No radiation zones/fire geometry/fall triggers/armor equipment are introduced.
Native session, save composition and consumable effects remain #369–#371; this
PR supplies the portable engine and fixtures. Graphical and reference-M1 gates
are not applicable to this standalone domain layer and are not claimed.
