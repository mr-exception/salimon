# Project Q&A

> Migrated from the Salimon project documents on 2026-09-29. For current task status and dependencies, use [GitHub issues](https://github.com/mr-exception/salimon/issues).

> **Purpose:** This page is only for unresolved product, gameplay, visual, UX, or architecture questions that require a human decision before implementation continues.
## How to use this page
- Add a question here only when the intended behavior cannot be safely inferred from existing project documentation.
- Each question should explain what decision is needed, why it matters, and the main options or tradeoffs when useful.
- Once answered, remove the question from this page and incorporate the decision into the relevant project document and/or task.
- Do not keep resolved decisions, implementation notes, task history, or technical summaries here.
## Open questions
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

[Native verification #236](https://github.com/mr-exception/salimon/issues/236)
mentions save/reload, while the accepted #234 integration and current
[resource compatibility contract](../client/world/resource-contracts.md) support
only in-memory streaming restoration, with no disk format or backend. Native
streaming unload/reload can be tested now; cross-process restoration cannot pass
without a separately designed persistence feature. Confirm whether to narrow
#236 to session streaming or schedule disk persistence and its native acceptance
in a future wave. Track the decision in
[issue #268](https://github.com/mr-exception/salimon/issues/268); this does not
change current resource lifetime or authorize a speculative save implementation.
