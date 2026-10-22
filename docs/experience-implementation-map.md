# Experience and narrative implementation/ownership map

Status: **documentation planning only; implementation issue descriptions do not imply delivery**. Documentation issue #462. This matrix aligns the narrative, art, navigation and Absenat requirements with existing issues and newly planned features. Existing issue state may change; always inspect blockers and latest PRs before implementing.

| Contract | Design source | Existing infrastructure/reconciliation | Development issue |
| --- | --- | --- | --- |
| Cinematic Scout composition/lighting | [Experience](experience-direction.md) | #273–279 low-poly style, #294/#299 Core frame, #308/#314/#451 QA | #463 |
| Believable connected machine animation/VFX | Experience | #342–348 Fabricator, ship/Core/door interactions | #464 |
| Spatial sound, quiet loneliness | Experience | #383 survival sound/feedback | #465 |
| Diegetic readable UI across onboard machinery | Experience | #343/#346 crafting UI, #430 scanner display | #466 |
| Lost life and physical persistent Archive | [Narrative](narrative-and-lost-journey.md), [Archive](archive-recovery-and-navigation.md) | #363–366 canonical save | #467 |
| Recovery module / energy motivation | Archive | Existing Core, #342–348 Fabricator, mining/cargo | #468 |
| Player-known unknown Earth, discovery chart | Archive | #402–406/#413–414 catalog, #421 navigation, #428–431 scanner | #469 |
| Distress accident + separate warning origins | Narrative | #412/#424–426 homeworld/terrain integration | #470 |
| Planet awakening and holistic onboard/exterior opening | Narrative and Experience | #380 tutorial, #412/#424–426 alien start | #471 |
| Absenat gate, centered blazing primordial Core, ring geometry | [Absenat](absenat-and-hall.md) | #422–424 rendering/streaming | #472 |
| Completion, blank sign, permanent menu unlock with no prior spoiler | Absenat | Future backend required | #473 |
| Global inscriptions, distance rankings, stable snapshot service | Absenat | #372 distance, #406 identity, future account/backend | #474 |
| Return visit, streaming signs, own latest placement | Absenat | Future backend and Hall | #475 |
| Playthrough, spoiler, native frame-time verification | All | #279/#308/#314, #435–439 M1, #440–442 docs CI | #476 |

## Recommended dependency-order planning

1. **Documentation now, no blockers:** #457–462, approved design distinctions established. Do not depend on implementation delivery for docs.
2. **Can start independently:** #463 Scout lighting/reference and #465 audio base. Work with existing art/runtime interfaces, not duplicate prior low-poly migration.
3. **Onboard presentation:** #464 after #463 and Fabricator work, #466 after #463 and relevant existing physical UI, #467 after persistence/UI, #468 after Archive/Fabricator.
4. **Real-scale discovery:** #469 after universe catalog, scanner and navigation foundations; #470 after recovery, chart and planetary destinations; #471 after alien homeworld, Scout and Archive slice.
5. **Later gated Absenat:** #472 can be renderer-only prototype after streaming/visual groundwork and must remain internal precompletion. #474 requires **real backend identity, authoritative journey completion and ranking**. #473 requires Hall and backend. #475 integrates immutable visit snapshots and revisits. #476 validates the complete journey at corresponding maturity; the Scout portion can be tested earlier.

## Grounded acceptance and consistency checks

- Evidence distinguishes observed telemetry, sender assertion, player hypothesis and unknown historical truth; **wormhole entry is unresolved**, not established.
- A fresh traveler cannot chart Earth merely from canonical universe data, a known name or an old archive entry.
- Reconstructed distress and warning messages can motivate locations without fake autopilot or mandatory linear progression.
- No lore leak of the Hall through main menu, disabled items, achievements or accessibility precompletion.
- Great-circle/cumulative path accounting, discontinuities, sign placement, ties, moderation and persistent unlock require independent implementation contracts before final tests.
- Visual/audio acceptance includes in-engine 1080p views, powered/unpowered states, contact feedback, low clutter, CPU/GPU cost; Linux/cloud evidence is not Apple M1/Metal signoff.
- Avoid overwriting full technical approved universe contracts in #402–442 merely to add story. A later documentation sync PR updates owned files as systems actually ship.

## Versioned verification delivery (#476)

[Experience validation](experience-validation.md) and the
[26-invariant matrix](../scenarios/portable/experience-v1.json) define current
portable Scout coverage and native/backend protocols. Twelve exact Scout test
mappings are runnable now; fourteen remaining rows explicitly retain their
opening/navigation/story/endgame blockers. Portable passes are not full E06,
M1/audio/appearance approval or a backend fallback implementation.
