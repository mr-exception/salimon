# Coding conventions

These rules apply to Rust runtime/domain code, Python tooling, shaders and assets.
Use the [maintenance map](maintenance-map.md) before changing a boundary and the
[validation matrix](validation.md) for exact checks.

## Responsibility and contracts

Keep internal modules and functions focused on one responsibility. Split when
independent responsibilities obscure ownership or require unrelated context;
there is no arbitrary line-count limit. Large inline test modules alone do not
justify a rewrite. The source map identifies the focused character and ship modules,
and runtime `app.rs` orchestration/private helpers.

Public contracts use typed commands, snapshots and validated domain values.
Runtime owns composition, platform input and cross-domain sequencing; domains
own their rules and state. Physical-object gravity, contact response and substeps belong in `salimon-physics`;
runtime supplies environment/frame/session adapters. See the
[ownership decision](technical-architecture.md#physical-object-simulation-decision-114).
Renderer owns GPU resources and renderer-neutral
DTOs, never character/ship/world dependencies. Keep fields private when mutation
must preserve invariants; use the narrowest useful visibility (`pub(crate)` for
internal crate interfaces). Read affected Cargo dependencies before changing
contracts. Do not add an event bus, ECS, WIT host or physics engine merely to
match a future proposal.

## Units, frames and determinism

Use meters and seconds (and explicit SI mass/volume) in contracts. Name scalar
values with their unit when ambiguous and document world, body-local, ship-local
or camera-relative frames on vectors/transforms. CPU absolute positions stay
`f64`; subtract camera origin in `f64` before GPU `f32` conversion. Follow the
[coordinate contract](../client/world/coordinate-strategy.md) and
[asset axes](../models/contracts.md), including Blender/runtime conversion.

Use `salimon-math` for compatible `f64` component arithmetic in migrated
consumers. Keep domain normalization thresholds/fallbacks and frame conversion
with their owner. Do not replace `hypot`, overflow-safe renderer normalization,
resource validation or quaternion policy with a superficially similar helper.
The [shared math decision](technical-architecture.md#shared-math-decision-115)
records compatible operations and intentional duplication.

Keep identities stable across streaming, pickup and drop. Preserve explicit
catalog and simulation iteration order; do not rely on hash-map iteration for
deterministic results. State tolerances with their unit and reason (e.g. floating
point spacing at distant anchors) rather than hiding errors with broad epsilons.
Fixed-step automation must use production controllers and gates.

## Rust and failures

Use Rust 2024, rustfmt defaults, workspace metadata/lints, stable Rust 1.89+,
and committed `Cargo.lock`. Normal verification uses `--locked`. Apply formatting
with `cargo fmt --all`; do not hand-align against rustfmt. `unsafe_code` is denied
and all Clippy warnings fail the quality gate. Dependency changes require a
deliberate lockfile update and relevant validation.

Return typed, actionable errors at fallible input, asset, platform and process
boundaries; include the failing operation, asset/path or broken contract.
Handle recoverable surface errors at the runtime boundary. Do not swallow
failures or turn failed capture/export into success. Panic/unwrap/expect is for
proven internal invariants or test assertions, not invalid user/file input;
explain a non-obvious invariant in the expect message or nearby comment.
Comments explain constraints, units, invariants and decisions rather than
repeating the code. Avoid speculative abstractions and unrelated cleanup.

## Python and shaders

Python build/runner scripts support 3.10+; tooling checks use Python 3.12.
PR CI only builds Rust source; run applicable tooling checks locally. Keep standard-library
build/run tooling independent of Blender/model dependencies. Model tooling's
third-party requirements live in `models/tools/requirements.txt`. Use explicit
paths rooted in the repository where the CLI supports them, small functions,
clear exceptions/nonzero exit codes, and bounded subprocess execution. Preserve
prior valid artifacts when export/build staging fails. Add tooling contracts
using the existing `unittest` layout; no new formatter or framework is mandated.

WGSL lives with renderer pipelines. Change shader bindings, vertex/uniform
layouts, alignment/padding and their Rust upload definitions together. Keep
reverse-Z clear/comparison/projection conventions consistent. Document a layout
or coordinate assumption where the CPU/shader contract is established. Shader
appearance needs actual renderer captures in addition to CPU-side tests,
with human appearance review under the [verification workflow](verification-workflow.md).
Cloud software Vulkan is suitable for routine rendered regression checks.

## Authored and generated files

Editable new assets live in `models/assets/`; runtime exports in `client/assets/`.
Read asset manifests, preservation/category contracts, licensing and consumers.
Do not hand-edit generated GLB/glTF/bin, export reports, spatial sidecars or
`client/character/src/spatial_contracts.rs`. For the scout use
`python models/assets/ships/salimon-scout/export.py --blender /path/to/blender`,
then its validator; generic asset CLI commands do not register the scout adapter.
See [authoring](../models/authoring.md) and
[scout source/regeneration](../client/assets/ship/README.md#source-and-regeneration).
Commit source, deliberate contract changes, exports, generated layouts and
consumer/tests/docs together. Blender is optional for normal runtime builds.

## Tests and documentation

Place Rust unit/regression tests with the owning module; cross-domain composition
contracts belong in runtime, not renderer. Python tooling tests live in
`models/tests/` or `scripts/test_*.py`. Use declarative baseline/evidence scenarios
for native integration; protect domain bugs with a focused regression where
practical rather than relying solely on screenshots. Tests should prove a
contract, edge case or failure mode. Documentation-only cleanup needs link/path
and evidence-integrity checks, not new tests mirroring prose.

Update affected canonical guides with behavior/contract changes. Crate guides
link here for common rules instead of duplicating them. Historical reports
record what was validated at their revision and do not override current guides.

## Issue completion and follow-up

Use the [two-label verification workflow](verification-workflow.md).
`auto-verification` means AI executes cloud checks and creates/updates a PR and
issue. `human-verification` means an issue gives ordered review/capture steps;
the person returns evidence to Codex for assessment and an issue update. Codex
opens a PR when repository documentation, evidence, code or baselines change;
only a complete evidence-only human decision requiring no repository change may
close directly, with reviewer/date, provenance, outcomes and evidence retained
in the issue. This is the sole exception to the report-file/PR rules below.

Deliver an issue's completed work in the current PR and close the source issue
on merge, using a closing reference such as `Closes #123` in the PR description.
If any point remains unresolved, create a separate follow-up issue for that
remaining work rather than keeping the entire source issue open. Create and
link the follow-up before the source issue closes so the work stays tracked.

For each follow-up:

- Give it a focused title, the exact unresolved requirement or decision,
  acceptance criteria, relevant evidence and any explicit blockers.
- Link back to the source issue, delivering PR and completion report. Link the
  follow-up from the source issue's completion update, PR and report as well.
- State which original acceptance criteria passed and which were transferred.
  Preserve failed, skipped and unrun checks and their reasons. Closing the
  source issue records delivery and transfer of remaining scope; it does not
  turn those checks into passes, approve a pending decision or waive a limit.

Update affected dependency links so work blocked by an unresolved point follows
the new issue rather than treating the closed source issue as a resolved blocker.
Keep unresolved product/architecture questions in [Project Q&A](project-qa.md)
linked to the follow-up until a decision is made. Preserve historical reports
as evidence at their recorded revision; current workflow belongs here.

For example, a performance PR can close its source issue after delivering an
optimization while a linked follow-up owns a remaining timing regression or
budget decision. The PR/report must still identify the failed gate or pending
approval explicitly.

## Task completion reports

Except for the evidence-only human closure above, every completed task must write/update `reports/issue-<number>/README.md`, or
`reports/task-<id>/README.md` for a task without an issue. Record:

- Outcome and meaningful changes, including the affected ownership/contracts.
- Exact validation commands and results, relevant environment/tool versions,
  skipped checks and the reason; distinguish not run, failed and passed.
- Limitations and remaining blockers, with links to the follow-up issues that
  own unresolved work under the [completion workflow](#issue-completion-and-follow-up).
- Applicable result screenshots/images beside the report, linked relatively;
  include structured state/logs when they establish behavior. Do not invent
  screenshots for a documentation-only task.

Link the report from the GitHub issue completion update alongside commits/PR.
Future task-result reports/images must not go in `docs/`; it holds maintained
specifications and development guides. Reusable asset previews remain with the
asset. Keep disposable build/E2E output in ignored `artifacts/`, promoting only
useful evidence into the task report. Redact unrelated desktop/private content
before committing screenshots. See [reports layout](../reports/README.md).

## Universe documentation impact (#440)

The [U35 maintenance gate](documentation-maintenance.md) requires a Documentation
impact section, same-PR guide updates or precise justified exceptions, owning
architecture/invariants links and actual path/link checks for every U01–U34 PR.
Authors complete it before readiness; reviewers require changes for gaps. Keep
approved requirements, implementation, historical fixtures and native evidence
distinct. Templates implement the checklist; automated CI lint is #442 and the
final consistency audit is #441, neither a reason to postpone documentation.
