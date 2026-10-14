# Issue 197 — reconcile existing flight/Core documents

Updated existing owners, without a new central specification. The 15-round,
84-decision review is cross-referenced by decision number/range in the existing
spaceship gameplay document and linked to technical timing/scope ownership.

| Review decisions | Maintained owner |
| --- | --- |
| 1–41, 43–77, 79–83 | `docs/spaceship-building-and-survival.md`: inertial flight, mass, throttle, Core/intake/power, gravity, instruments, safe assistance and final crash/recovery clarifications |
| 42, 78 | `docs/technical-architecture.md`: deterministic bounded ticks, explicit backlog handling and domain boundaries |
| 9, 58, 63, 72, 84 | `docs/project-phases.md`: excluded future systems, incremental issues and existing-document-only workflow |
| Numerical/API invariants | `client/ship/{architecture,invariants}.md` and README/AI guide |
| Rendering ownership | Existing renderer guides link to ship feasibility rules instead of repeating them |
| Current systems/index/history | Existing root/client READMEs, game systems overview and Phase 0 baseline scope banner |

The final round's damped crash/grounded/recovery rules explicitly supersede
older shorthand about immediate stops or treating a crash as Landed. The
historical Phase 0 direct-speed/no-energy specification and dated evaluation
remain history, with an explicit current-scope supersession. Stale descriptions
of three monitors, deferred intake, unavailable recovery, takeoff at rest and
hull-on-ground placement are corrected.

Future thruster/landing upgrades, module loads, cabin inertial forces,
gravity/orbits and manual power management are explicitly OUT OF SCOPE. Tuning
values/configuration seams do not imply an upgrade/progression implementation.
Existing #193/#195 reports receive dated CI follow-ups with exact revision/run
links; historical results are not relabeled as latest-revision hardware evidence.

Validation:

- 128 local Markdown file targets and new owned-section anchors checked.
- Decision-range coverage checked: every number 1–84 has an owning reference.
- `git diff --check`: passed.
- Final stack `cargo test --workspace --locked` passed (single-codegen-unit container environment); formatting and Clippy passed. Python runner contracts: 58 passed.
- Documentation-only changes introduce no runtime/asset behavior. Apple M1 hardware validation remains outside these cloud tasks.
