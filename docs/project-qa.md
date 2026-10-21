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
### Which dark-cabin condition should Core contrast verification use (#451)?

The [#299 native evidence](../reports/issue-299/README.md) verifies the powered
and blank Core face under current fixed warm interior lighting. Outage blanks
the display but does not dim the cabin. Decide whether a separate dim-light
validation fixture is intended or whether the requirement should use existing
lighting. [#451](https://github.com/mr-exception/salimon/issues/451) owns the
remaining native condition and evidence; no dark-cabin pass is claimed.

### What budget should complete legacy active 500-piece M1 signoff (#479)?

The [2026-10-21 native evidence](../reports/issue-255/2026-10-21/README.md)
retains 150 completed moving intervals across ten fresh drops: frame p95
23.446 ms, physics p95 5.932 ms, no intervals over 50 ms and a 1.064 ratio to
matched empty launch prefixes. All numerical checks in the existing
[#288 proposal](../reports/issue-288/README.md) pass; its status remains
`proposed_pending_owner_review`. Requesting a proposal did not approve its limits.
The active results still exceed settled 17.5 ms guidance and the provisional
5 ms physics partition; neither is silently replaced.

Owner review must decide whether the proposed active/launch-period limits and
remaining historical evidence justify acceptance or whether more profiling is
required. Fresh exact historical binary pairs pass the 20% relative checks, but
the six prior whole-window and three prior same-state CPU-render failures remain
recorded failures without an attributed cause. [#479](https://github.com/mr-exception/salimon/issues/479)
owns this decision and any required attribution after #255 delivery. Keep this
question open until explicit review resolves it; source closure grants no signoff.
Current-cap/mixed-sphere overload is independently tracked in #455.

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

## Survival restriction tuning (#369, 2026-10-19)

The approved movement factors and exact continuous threshold intervals are
implemented in the [restriction contract](../client/survival/architecture.md#movement-and-interaction-restrictions-369).
Unspecified numeric interaction/minimum-speed settings are explicit configurable
provisional tuning: minimum 10%, low-hunger and low-fatigue interaction rates
75% each (multiplicative), 250 ms base recovery for discrete nonessential actions.
Normal rate adds no recovery; emergency doors/food/medicine/menus bypass it.
These defaults are exposed for review and later #384 balance work, rather than
being immutable gameplay requirements. Exhaustion selects zero manual thrust
without cancelling an already committed assist or braking inertial motion;
recovery requires fresh cockpit entry/input. Whole-save activation remains #371.

## Consumable recovery tuning (#370, 2026-10-19)

Basic `food-ration` uses the expansion's +40 instant hunger default. The existing
`medkit` receives explicit provisional, configurable data of +2.5 ordinary health
per active second for 10 seconds (+25 total), cap one stack; subsequent legitimate
kits refresh duration without multiplying stacks or resetting partial cadence.
This is implementation tuning for review/#384 balancing, not a prior approved
exact efficacy or medical-lore requirement. Definitions can select treatment
healing for later items; the starter kit does not silently erase treatment-only
wounds. Full meters allow consumption and wasted excess. Recipes, ingredients,
production time/watts/mass and unlocks remain unchanged; advanced foods are #378.
The [canonical recovery contract](../client/survival/architecture.md#consumable-recovery-definitions-370)
owns validation, identity conflicts, exact continuation and offline freeze.

### When can the native Metal candidates become approved baselines?

The 2026-10-21 [#314 delivery](../reports/issue-314/README.md) contains scale-2 Apple M1 candidates, frozen-repeat checks and defect-injection evidence. Human image/mask/tolerance approval and a separate baseline promotion PR remain required; the scale-1 pending descriptor must be reconciled deliberately. True normal-play interpolation at available 60/75 Hz is also unrun: fixed-step E2E disables it, and external native input delivery failed. [#477](https://github.com/mr-exception/salimon/issues/477) owns these remaining decisions/checks, coordinating #453. Source issue closure grants no appearance or interpolation acceptance.

Scout #463 lighting is a bounded presentation proposal pending image/readability and matched frame-budget review in [#483](https://github.com/mr-exception/salimon/issues/483). Its cool blackout floor is visibility fill, not emergency electrical energy. Native #451/#477 captures from older shaders do not approve this change.
