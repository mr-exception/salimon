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

## Cloud completion audit — 2026-10-21

The original engine was already merged through PR #394 into the current main
ancestry. This follow-up fixes one reproduced remaining acceptance gap: the
regeneration accumulator survived an eligibility loss ending exactly at an
advance boundary, or periodic food at that boundary. Immediate feeding could then
reuse credit from the interrupted interval. Reset now occurs at the ineligible
timestamp before food/healing publication. Inclusive thresholds remain unchanged.

Two new deterministic regressions failed on main, then passed after the fix.
They cover immediate feeding, checkpoint roundtrip, split elapsed admission and
periodic food at the first ineligible nanosecond, using validated 216-second
hunger tuning to expose partial regeneration credit. No default tuning changes.

Linux validation: `cargo test -p salimon-survival --locked` passes all 29
integration tests; `cargo clippy -p salimon-survival --all-targets --locked --
-D warnings` passes. `cargo fmt --all` applied. No GUI/M1 checks apply to this
portable domain fix. Existing 2026-10-19 evidence above remains historical.
