# Repository guidelines

## Start here

1. Read the active [GitHub issue](https://github.com/mr-exception/salimon/issues),
   comments, acceptance criteria and **Blocked by** section. Resolve explicit
   blockers before dependent work. Issue metadata defines task scope; historical
   task numbers do not prescribe execution order.
2. Use [current architecture](docs/technical-architecture.md) and the
   [feature maintenance map](docs/maintenance-map.md) to find owners, sources,
   tests and scenarios. Read only the affected crate's `README.ai.md`,
   `architecture.md`, `invariants.md` and linked contracts before editing.
3. Follow [coding conventions](docs/coding-conventions.md) and select the exact
   affected-area checks from [validation](docs/validation.md). Read source before
   trusting prose, especially historical reports.

## Scope and precedence

Current implementation is a native, client-only custom Rust/winit/wgpu game.
`client/` owns runtime code; `models/` owns offline authoring; `core/` has no
backend implementation. Ship flight/landing, movement/EVA, mining, carrying and
fragment motion/contact are implemented. Resource changes survive local streaming
in the current in-memory session. Disk/backend persistence, networking, orbital
simulation and production survival/energy management remain deferred.

Explicit user instructions govern the task. Approved specifications and issue
acceptance criteria define intended behavior; current code/tests establish what
is implemented. Component invariants constrain changes unless deliberately
revised with their consumers/tests. Future proposals and historical evidence
are not active implementation instructions. If these disagree, check code and
issue decisions, repair stale documentation in the same change, and record
unresolved product/architecture decisions in [Project Q&A](docs/project-qa.md)
and the issue rather than silently inventing policy.

## Ownership and completion

Runtime composes typed domain state and native events; renderer owns GPU work
and renderer-neutral DTOs; supporting domains do not call runtime. See the
maintenance map for precise boundaries and the asset regeneration owner.
No full game engine, framework adoption or broad refactor is implied by cleanup.

Keep behavior/contracts and affected guides aligned in the same change. Use one
canonical definition per rule and link local guides to it. Preserve useful
historical evidence while labeling its date/revision and limitations.

Every completed task **must** create or update
`reports/issue-<number>/README.md` (or `reports/task-<id>/README.md` without a
GitHub issue), recording outcome, meaningful changes, checks/results, and
limitations or blockers. Put applicable screenshots/images beside it with
relative links. Follow [report conventions](docs/coding-conventions.md#task-completion-reports)
and [reports layout](reports/README.md); never add task-result folders to `docs/`.
Link the report and relevant commits/PR from the GitHub issue completion update.
Close an issue only after acceptance criteria pass; a PR awaiting merge should
link the issue for closure on merge. Push to main only when explicitly requested;
otherwise use a reviewable branch/PR for the requested workflow.
