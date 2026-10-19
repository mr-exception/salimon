# Project Q&A

> Migrated from the Salimon project documents on 2026-09-29. For current task status and dependencies, use [GitHub issues](https://github.com/mr-exception/salimon/issues).

> **Purpose:** This page is only for unresolved product, gameplay, visual, UX, or architecture questions that require a human decision before implementation continues.
## How to use this page
- Add a question here only when the intended behavior cannot be safely inferred from existing project documentation.
- Each question should explain what decision is needed, why it matters, and the main options or tradeoffs when useful.
- Once answered, remove the question from this page and incorporate the decision into the relevant project document and/or task.
- Do not keep resolved decisions, implementation notes, task history, or technical summaries here.
- When the delivering PR closes a source issue with a question still unresolved,
  link the question to its new follow-up issue under the
  [completion workflow](coding-conventions.md#issue-completion-and-follow-up).
## Open questions
### What budget should complete active 500-piece M1 signoff (#255)?

[The latest native evidence](../reports/issue-255/2026-10-17-remediation/README.md)
removes the measured redundant physics work but retains an admitted active-only
frame p95 of 24.416 ms and physics p95 of 5.741 ms. These exceed the existing
17.5 ms settled cadence guidance and provisional 5 ms physics partition; neither
limit is changed by the remediation. The benchmark guide requires an explicitly
reviewed active budget, which has not yet been approved.
Matched empty launch prefixes have frame p95 23.897 ms, so the long settled
reference and short naturally moving launch window also have different cadence
coverage. Six whole-window CPU-render relative comparisons still fail their
existing 20% check; the report retains these failures separately from absolute
component guidance.

Decide whether signoff must satisfy those existing reference values after further
profiling, or whether a separately justified active/launch-period budget should
be reviewed using matched empty-launch evidence. Track that decision and the
remaining measurements from [#255](https://github.com/mr-exception/salimon/issues/255)
in linked follow-up issues when delivering its PR; a faster solver alone is not
active cadence acceptance, and source-issue closure does not grant signoff.

On 2026-10-17 the owner requested a separately justified active-budget proposal
for PR review. The [#288 proposal and reproducible evaluation](../reports/issue-288/README.md)
provide that decision material. This request authorizes proposing thresholds;
it does not approve them or waive the historical relative failures. Keep this
question open until the owner reviews the proposal and remaining limitations.
Fresh same-state settled comparisons also fail CPU-render p95 by 22.0–24.7%
in all three pairs, while frame and physics relative checks pass. Different
moving/sleeping proportions alone therefore do not explain the relative failure;
its cause still needs attribution before unconditional signoff.

### Where should dedicated physical storage go after the cargo-room removal?

The current scout revision removes the dedicated port cargo room and its storage
volume to improve the ship silhouette. Loose fragments can still be dropped on
clear main-cabin deck space. The previous requirement for an initial dedicated
cargo room is superseded. Decide whether a later scout revision should add
integrated storage within the aerodynamic hull, use a detachable module, or defer
dedicated storage to a later ship design. This decision affects future container
placement, cargo capacity and ship expansion; it does not block the current
geometry revision or loose-fragment storage on the deck.

### What does #236 require by “save/reload”?

[Issue #268](https://github.com/mr-exception/salimon/issues/268) proposes keeping
[native verification #236](https://github.com/mr-exception/salimon/issues/236)
within the existing Phase 0 session-only boundary. Its PR is the review point
for the owner decision: approve in-session streaming unload/reload for this wave,
or request a future disk-persistence wave with separate native save/relaunch
verification. The [canonical persistence contract](../client/world/resource-contracts.md#persistence-and-reload-verification-scope-268)
records the proposed scope, evidence routes and future prerequisites.

Owner approval is pending; this PR does not claim that a decision was already
made. Keep this question open until answered, then remove it and record the
answer in the canonical contract and #236 issue scope. Current runtime behavior
remains session-only while review is pending.


Fabricator crafting is newly authorized by the [accepted five-round contracts](spaceship-building-and-survival.md#accepted-fabricator-contracts-2026-10-19-336). Implementation is staged: #336 contracts, #337 raw materials, #338 portable domain, then separate runtime/UI integration. Survival effects, disk persistence and playable progression remain deferred.
