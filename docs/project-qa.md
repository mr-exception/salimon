# Project Q&A

> Migrated from the Salimon project documents on 2026-09-29. For current task status and dependencies, use [GitHub issues](https://github.com/mr-exception/salimon/issues).

> **Purpose:** This page is only for unresolved product, gameplay, visual, UX, or architecture questions that require a human decision before implementation continues.

> **Resolved 2026-10-22:** Starter Scout cargo/storage is no longer an open question. The approved future design includes a **dedicated aft cargo bay, full-height partition, wide closable door and physical bins**, distinct from the current authored Scout implementation. See [approved Scout and modular construction contract](scout-and-modular-construction.md) and issues #538–#540.
## How to use this page
- Add a question here only when the intended behavior cannot be safely inferred from existing project documentation.
- Each question should explain what decision is needed, why it matters, and the main options or tradeoffs when useful.
- Once answered, remove the question from this page and incorporate the decision into the relevant project document and/or task.
- Do not keep resolved decisions, implementation notes, task history, or technical summaries here.
- When the delivering PR closes a source issue with a question still unresolved,
  link the question to its new follow-up issue under the
  [completion workflow](coding-conventions.md#issue-completion-and-follow-up).
## Open questions
### What budget should complete legacy active 500-piece M1 signoff (#563)?

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
recorded failures without an attributed cause. [#563](https://github.com/mr-exception/salimon/issues/563)
owns this decision and any required attribution after the
[#479 reverified decision package](../reports/issue-479/README.md). Keep this
question open until explicit review resolves it; source closure grants no signoff.
Current-cap/mixed-sphere overload is independently tracked in #455.

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

The 2026-10-21 [#314 delivery](../reports/issue-314/README.md) contains scale-2 Apple M1 candidates, frozen-repeat checks and defect-injection evidence. Human image/mask/tolerance approval and a separate baseline promotion PR remain required; the scale-1 pending descriptor must be reconciled deliberately. True normal-play interpolation at available 60/75 Hz is also unrun: fixed-step E2E disables it, and external native input delivery failed. [#561](https://github.com/mr-exception/salimon/issues/561) owns human corpus decisions/review, with #557 physical platform checks and #568 cloud automation. Earlier #477/#453 findings remain dated evidence. Source issue closure grants no appearance or interpolation acceptance.

Scout #463 lighting is a bounded presentation proposal: #483/#566 own cloud capture/regression/timing evidence, #561 human image/readability review and #563 hardware budget decisions under the [verification workflow](verification-workflow.md). Its cool blackout floor is visibility fill, not emergency electrical energy. Native #451/#477 captures from older shaders do not approve this change.

Scout #465 cloud synthesis/routing/playback lifecycle and matched audio-on/off
measurements remain [#485](https://github.com/mr-exception/salimon/issues/485);
human timbre/mix/listening review is #561, physical OS playback/device checks
#557 and hardware budget decisions #563.
Portable synthesis is bounded but cannot grant listening or M1 performance
acceptance; the minimal shared OS playback seam may need a continuous backend.

## Survival difficulty budget review (#384 / #496)

Portable balance evidence is [documented](survival-balance.md). Proposed comfortable
360-second and break-even 720-second one-ration budgets await explicit owner
approval in [#660](https://github.com/mr-exception/salimon/issues/660) after #496
evidence delivery. Physical routes/subjective native timing remain unmeasured;
current non-Earth profiles have no biomass. Approved meter durations are unchanged.

## Engine handling after impulse migration (#493 / #498)

The approved 1 N·s/J and 100/50 MW imply 1000/500 m/s² dry acceleration.
The 20m/s landing gate, stationary assist handoff and 60GJ capacity remain.
Portable tests reconcile numerical behavior; native handling/readability review
remains #498. Jump Drive N=20% capacity/progression balance stays future, unchanged.


## Replacement corpus and production source decisions (#537)

On 2026-10-27 the owner authorized proceeding with the verified #537 preparation
in PR #658 and verifying personally later. This accepts delivery with deferred
review, not unrun checks or new source permissions. [#659](https://github.com/mr-exception/salimon/issues/659)
owns source-specific full terms, exact credits/permissions and full production
snapshot approval; #660 owns measured survival/recovery routes and balance
choices; #661 owns future full-experience/backend acceptance. #561 retains human
image/mask/tolerance approval and independent hosted software-Vulkan corpus
review, with #557 physical platform experiments and #563 hardware qualification.
The owner still permits private development only; production eligibility remains
blocked on #659. Closing #537 grants no appearance or production-data approval.

## Approved construction economy (#543, owner decision 2026-10-24)

The owner explicitly answered “Approve the proposed catalog” in this chat on
2026-10-24. The catalog is now `approved_economy:true`; #543 may accept physical
materials. Timed construction still belongs to #544.
The authored `client/runtime/catalogs/blueprints.json` is the single source:
stool 1 kg Iron / 10 s; partition 5 kg / 20 s; internal door 5 kg / 30 s;
hull cell 20 kg / 120 s; deck cell 15 kg / 90 s; two-deck room 40 kg / 300 s;
utility module 3 kg / 60 s. These are approved gameplay parameters, not a measured
acquisition-route or subjective play-balance result.
Fine-grid pitch is per blueprint (currently 0.5 m); the structural lattice stays
3 m. B02 does not authorize occupied-room demolition or structural completion.


## Core upgrade balance — owner direction 2026-10-24 (#548)

The owner answered the unresolved-values question with “guess them yourself” and asked for balanced costs so play is neither too grindy nor too easy. This explicitly authorizes the reviewable data catalog: 90/120/180 GJ, incremental iron/silicate costs 20/10, 30/10, 40/20 kg. Rationale, exact source/transaction semantics and limits are in [Core upgrades](core-upgrades.md). These are selected gameplay parameters, not empirical material measurements, a measured playtest, or blanket approval of the unrelated construction economy.
