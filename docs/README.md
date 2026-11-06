# Salimon project documents

> **Start with the 2D client:** [Code-checked browser status](current-client-status.md) and the [complete documentation migration register](2d-migration-register.md). Each legacy 3D file is classified by implementation status and future overhead-view relevance; old Rust completion notes do not mean the feature is available in the current game.

Follow the authoritative [visual style](visual-style.md) for art decisions.

GitHub issues are the task source of truth. Read each issue's **Blocked by** section before starting work. These documents capture the project specifications as migrated on 2026-09-29; historical phase wording does not override current issues.

## Approved narrative, experience and endgame designs (not yet implemented)

- [Lost journey and narrative canon](narrative-and-lost-journey.md)
- [Grounded cinematic game experience direction](experience-direction.md)
- [Scout Archive, recovery and player-known navigation](archive-recovery-and-navigation.md)
- [Absenat gate, universe chain and Hall of Passage](absenat-and-hall.md)
- [Experience initiative: traceability, dependencies and validation](experience-implementation-map.md)

These newer approved narrative contracts supersede historical Earth-start assumptions **for future implementations**; they do not imply that the current prototype already implements the alien-world opening, online Hall or endgame.

## Development acceptance

- [Issue acceptance, QA states, evidence and closure](acceptance-policy.md)
- [Navigation acceptance matrix and known-defect review example](acceptance-navigation-example.md)

## Active browser world and cockpit

- [Playable 2D foundation, QA commands and mining reuse boundary](playable-2d-foundation.md)
- [Navigation-only holographic radar contract and migration boundary](radar-navigation-prototype.md)
- [Approved cockpit transitions and future Stasis Pod-based reconnect](cockpit-navigation-and-reconnect.md)
- [React prototype setup](../client/README.md)

## Backend design (approved future, not implemented)

- [Authoritative NestJS/TypeScript backend, simulation boundaries and networking](backend-architecture.md)
- [MongoDB durable transactions, 5 s checkpoint target / 30 s strict window and crash recovery](backend-persistence-recovery.md)
- [Stasis Pod safe/unsafe logout, one-hour approved transfers, pod-loss permadeath and deferred questions](stasis-pod-and-transfer.md)

These are planning contracts only. The current browser prototype is local, not an already-running server-authoritative multiplayer game.

## Product and gameplay

- [Project Vision](project-vision.md)
- [Game Systems Overview](game-systems-overview.md)
- [World & Realism](world-and-realism.md)
- [Canonical planetary generation, multi-seed identity and approved migration policies](canonical-planet-generation.md)
- [Spatial Jump Drive, sub-light travel, shared time and Core impulse (approved future design)](jump-drive-and-interstellar-travel.md)
- [Spaceship, Building & Survival](spaceship-building-and-survival.md)
- [Weight-only inventory, mining and resource storage](resource-inventory-and-gathering.md)
- [Spacesuits, hazards, suit drawers and protection-module storage](spacesuit-and-hazards.md)
- [Multiplayer & NPC Interaction](multiplayer-and-npc-interaction.md)
- [Cockpit control and Stasis Pod-based disconnect/reconnect/death rules](cockpit-navigation-and-reconnect.md)
- [Technical Architecture & AI Maintenance](technical-architecture.md)

## Business specifications

- [Product Requirements](product-requirements.md)
- [Project Phases](project-phases.md)
- [Phase 0 — Technical Feasibility Showcase](phase-0-technical-feasibility.md)
- [Project Q&A](project-qa.md)

## Maintenance entry points

- [Feature ownership, sources, tests and scenarios](maintenance-map.md)
- [Coding conventions and completion reports](coding-conventions.md)
- [Issue completion and follow-up workflow](coding-conventions.md#issue-completion-and-follow-up)
- [Cloud and human verification workflow and labels](verification-workflow.md)
- [Validation matrix and prerequisites](validation.md)
- [Historical task reports](../reports/README.md)

## Historical repository documents

The native `models/` workspace was removed during the browser reset. The five
authoring links below explicitly target the verified historical tree, preserving
the contracts without implying these tools or assets exist in the current client.

- [3D authoring workspace and ownership](https://github.com/mr-exception/salimon/blob/8f112bb6e6be9f813174efb47d706a8601930d04/models/README.md)
- [Authored asset manifest schema and examples](https://github.com/mr-exception/salimon/blob/8f112bb6e6be9f813174efb47d706a8601930d04/models/manifest.md)
- [Shared 3D asset contracts](https://github.com/mr-exception/salimon/blob/8f112bb6e6be9f813174efb47d706a8601930d04/models/contracts.md)
- [Blender and AI authoring workflow](https://github.com/mr-exception/salimon/blob/8f112bb6e6be9f813174efb47d706a8601930d04/models/authoring.md)
- [Generic Blender export command](https://github.com/mr-exception/salimon/blob/8f112bb6e6be9f813174efb47d706a8601930d04/models/tools/README.md)

- [Phase 0 evaluation](phase-0-evaluation.md)
- [Legacy Salimon hub notes](legacy-salimon-hub.md) — older entity/buffer concept, separate from the current space game.
- [All issues](https://github.com/mr-exception/salimon/issues)
