# Issue 468 — configurable Core-funded Data Recovery

Delivered one Fabricator-produced controller package, installed into the existing
physical Archive housing by E collection; installed item/mass ownership is exact
and durable. R starts/stops a selected damaged record. Configured Core-funded
work retains paid progress, protects an emergency floor, pauses on no surplus,
and exposes partial/recovered/unverified/irrecoverable states. Current runtime
schema 7 preserves config, exact item and work source; v1–v6 migration grants no
hardware or resources and retains existing captured jobs/recipes.

[Canonical contract](../../docs/data-recovery-module.md),
[runtime architecture](../../client/runtime/architecture.md#data-recovery-module-468),
[Fabricator architecture](../../client/fabrication/architecture.md#recovery-controller-component-468),
[ship boundary](../../client/ship/architecture.md#low-priority-data-recovery-energy-468).

## Validation

Linux x86_64, Rust 1.99.1, Python 3.12.14:

- `cargo test --workspace --locked`: passes, 793 tests; 28 opt-in tests ignored.
- `cargo test -p salimon-fabrication -p salimon-ship --locked`: passes, including
  invalid wire recipes, atomic reserve funding and transient telemetry restore.
- `cargo clippy --workspace --all-targets --locked -- -D warnings`: passes.
- `cargo build --workspace --locked`: passes.
- `cargo fmt --all -- --check`, `git diff --check`, offline documentation lint:
  pass. Five documentation-validator and four offline-importer contracts pass.
- `SALIMON_RECOVERY_EVIDENCE=<absolute-dir> cargo test -p salimon-client --locked
  recovery_ui_evidence -- --ignored`: passes; CPU RGBA/size output converted
  losslessly to the PNGs below. Text is bounded at 606×278 and 909×417, visually
  inspected. This is CPU raster output, not an OS/native screenshot.

Recovery scenarios cover failed/duplicate/batch enqueue, exact tray-to-installed
mass, one identity across save/load, depleted Core, emergency floor, abort with
no refund, source corruption/current omissions, explicit old-save recipe migration,
normal native-key translation, pause, tick continuation and real-domain
mine → carry → Fabricator → install → mine/carry → Core conversion → first record.
Deposits and route placement are explicit synthetic fixtures; physical navigation
and subjective balance were not measured. Shader parsing/validation caught an
initial reserved-word error, which was corrected before final checks. An initial
Clippy unused import was also corrected; neither remains a failed final gate.

## CPU text evidence

[Scale 2 PNG](archive-recovered-2.png) and [scale 3 PNG](archive-recovered-3.png)
show the same recovered first record and unverified source/time/location. No
coordinate, confirmed missing history or precompletion Hall UI is added.

## Remaining acceptance

- [#515](https://github.com/mr-exception/salimon/issues/515) owns native OS input,
  live component/Archive appearance, supported DPI, reference Apple M1/Metal,
  physical route, frame budget and owner review of provisional recipe/energy
  defaults/installation presentation. No live GUI/GPU capture or native budget
  approval is claimed by CPU tests. The resumed container has no usable Vulkan
  ICD/display evidence; native rendering remains unrun.
- [#470](https://github.com/mr-exception/salimon/issues/470) owns authored full
  distress/warning location leads after charting/world dependencies. This delivery
  only reconstructs uncertain local fragments; it invents no missing coordinates.
- Future universe offline machinery is #433. Current closed-app Recovery work is
  zero, and pause/death do not bank work. Stop retains paid work with no refund.

Closing #468 records this portable delivery and explicit transfer, not native,
art/balance or full narrative destination approval. Core overfill keeps the
existing waste/clipping policy; Recovery spends only surplus after ordinary loads.
