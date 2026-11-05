# 2D documentation migration register

**Audit on 2026-11-05:** 106 existing Markdown files in `docs/` were read and classified; **107 rows including this new register** (GUIDE: 12, DESIGN: 22, NATIVE: 61, ARCHIVE: 6, PLAYABLE: 6). Current reference: `main` after merged PRs #789 and #790. Root/client guides, GitHub workflows/templates and archived issue reports were checked as routing or evidence, not rewritten as current implementation.

**How to interpret status:** **PLAYABLE** means a current browser code owner/contract (individual future subsections remain future); **GUIDE** is active routing/policy; **DESIGN** is approved or proposed future gameplay, **not shipped**; **NATIVE** is revision-specific Rust/3D implementation evidence; **ARCHIVE** is evaluation/qualification or earlier discarded designs. A historical issue closing does not demonstrate browser functionality.

## Contract precedence and approved 2D interpretation

1. **Current capability:** [browser implementation status](current-client-status.md), actual `client/src` implementation and its production tests. Never infer active functionality from a removed Rust module.
2. **Future behavior:** latest owner-approved [2D art](top-down-art-direction.md), [cockpit/reconnect](cockpit-navigation-and-reconnect.md), [weight-only inventory](resource-inventory-and-gathering.md), [suits and hazards](spacesuit-and-hazards.md), [planet environment](planetary-environment-art-direction.md) and [narrative](narrative-and-lost-journey.md). Where rules are unresolved, keep explicit TBDs.
3. **Historical contracts:** native Rust source, models, 3D screen/camera, fragment, original build, reports and benchmarks in the [pinned pre-2D tree](https://github.com/mr-exception/salimon/tree/8f112bb6e6be9f813174efb47d706a8601930d04) provide evidence **for that revision**. Preserve useful numbers/identity; do not import superseded assumptions into current browser development.
4. **Verification:** cloud browser/TS tests and captures are the routine gate; physical device, M1/Metal and subjective release review retain separate ownership.

| Historical 3D concept | Current 2D treatment |
| --- | --- |
| First-person player, room/terrain meshes, skybox | **Strict overhead illustrated 2D** world. Layered sprites, persistent IDs, actual 2D collision footprints/interaction anchors and intentional tall-object occlusion. Canonical world can still be physically 3D. |
| 3D game-wide navigation | **3D holographic orthographic radar only at cockpit desk**. The sample radar does not physically relocate the authored 2D landing site. |
| Mineable physical fragments and two volume/weight limits | Direct raw-resource credit in **integer grams**. Initial inventory: **10 kg and 12 slots**, raw type stacking, per-item count stacks/weights, **no volume**. Source debit equals credit. |
| Cargo racks and fabrication/fuel intakes | Future 2D module interactions and exact transfers; **50 kg Resource Container**, separate **50 kg Fabricator/Core raw buffers** are approved, **not playable storage/machines**. |
| First-person Scout interior, physical modules | Continuous visible **top-down Scout cutaway**, ship-local frames, fixture footprints and concise contextual UI; preserve cozy habitation identity and build constraints when porting. |
| Physical suits, hazards and upgrades | Future **tierless** basic suit with two module slots, drawer tiers **4/8/12**, module-store tiers **12/16/24**, **200 HP** character health, independent exposure and **1,000 HP** absorption per module. Exact gear masses/recipes remain TBD. |
| Native localized room/module hull damage | Current 2D design uses **ship-wide damage or total ship destruction**. Partial-room destruction and detached compartment logic deferred; offline death only after authoritative whole-ship loss. |
| Native Gaia, terrain, gravity, streaming | Retain reproducible canonical data, IDs and physics as future port inputs; authoritative overhead surface sprites/colliders and cockpit radar proxies need independent browser/backend integration. |
| Native whole-session save or network state | Browser-local `ResourceSession` **v4** gram ledger (after #783/#784) is **not** native schema 29+, multiplayer authority or cloud sync. |
| Rust QA and native screenshot evidence | Historical; use Node/TypeScript and real browser Playwright capture for new feature delivery. Native hardware signoff is not a default browser blocker. |

## Development sequence (not a new gameplay approval)

The persistent source/store domain is delivered by stacked #783 (now merged); the next bounded gameplay adapters are [#784 raw Resource Container](https://github.com/mr-exception/salimon/issues/784), [#785 storage UI](https://github.com/mr-exception/salimon/issues/785), [#786 count-item UI](https://github.com/mr-exception/salimon/issues/786), [#787 survival](https://github.com/mr-exception/salimon/issues/787) and [#788 hazards](https://github.com/mr-exception/salimon/issues/788). Real canonical planetary navigation, Core/Fabricator, wearable gear, full multiplayer and Absenat require further scoped work or outstanding product decisions. No original Rust issue is automatically reopened or treated as already ported.

## Complete file-by-file classification

One row for every `docs/*.md` file in this revision. `reports/` is kept as historical reproducibility evidence, not rewritten or reclassified as active source. Archived-source links inside guides are pinned to verified pre-reset paths when missing from the current tree.

| Document | Status | 2D adaptation / next action |
| --- | --- | --- |
| [`2d-migration-register.md`](2d-migration-register.md) | GUIDE | This register is the file-by-file migration index; keep it synchronized with source changes. |
| [`README.md`](README.md) | GUIDE | Start here; route active 2D specs before archived 3D references. |
| [`absenat-and-hall.md`](absenat-and-hall.md) | DESIGN | Keep secret endgame design; previous Hall test was native only. |
| [`acceptance-navigation-example.md`](acceptance-navigation-example.md) | GUIDE | Historical contract or approved intent; do not claim code migration without new issue and browser tests. |
| [`acceptance-policy.md`](acceptance-policy.md) | GUIDE | Observable cloud/browser acceptance, historical native qualification separate. |
| [`active-geology.md`](active-geology.md) | NATIVE | Archived native environment. Translate published identities, effects and sources into illustrated overhead layers and 2D footprints. |
| [`architecture-baseline.md`](architecture-baseline.md) | ARCHIVE | Historical contract or approved intent; do not claim code migration without new issue and browser tests. |
| [`archive-recovery-and-navigation.md`](archive-recovery-and-navigation.md) | DESIGN | Historical contract or approved intent; do not claim code migration without new issue and browser tests. |
| [`biological-harvesting.md`](biological-harvesting.md) | NATIVE | Archived native environment. Translate published identities, effects and sources into illustrated overhead layers and 2D footprints. |
| [`biological-signature-sites.md`](biological-signature-sites.md) | NATIVE | Archived native environment. Translate published identities, effects and sources into illustrated overhead layers and 2D footprints. |
| [`cabin-mechanisms.md`](cabin-mechanisms.md) | NATIVE | Archived native Scout module. Preserve functional design; use cutaway, ship-local interaction footprints and gram-based transfers in 2D. |
| [`canonical-planet-generation.md`](canonical-planet-generation.md) | DESIGN | Historical contract or approved intent; do not claim code migration without new issue and browser tests. |
| [`cockpit-navigation-and-reconnect.md`](cockpit-navigation-and-reconnect.md) | PLAYABLE | Delivered local chair transitions; multiplayer reconnect/offline death still future. |
| [`coding-conventions.md`](coding-conventions.md) | GUIDE | TypeScript/React domain conventions should govern active edits; Rust sections archival. |
| [`construction-damage.md`](construction-damage.md) | NATIVE | Archived native Scout module. Preserve functional design; use cutaway, ship-local interaction footprints and gram-based transfers in 2D. |
| [`construction-flight-safety.md`](construction-flight-safety.md) | NATIVE | Archived native Scout module. Preserve functional design; use cutaway, ship-local interaction footprints and gram-based transfers in 2D. |
| [`construction-materials.md`](construction-materials.md) | NATIVE | Archived native Scout module. Preserve functional design; use cutaway, ship-local interaction footprints and gram-based transfers in 2D. |
| [`construction-offline.md`](construction-offline.md) | NATIVE | Archived native Scout module. Preserve functional design; use cutaway, ship-local interaction footprints and gram-based transfers in 2D. |
| [`construction-progress.md`](construction-progress.md) | NATIVE | Archived native Scout module. Preserve functional design; use cutaway, ship-local interaction footprints and gram-based transfers in 2D. |
| [`core-upgrades.md`](core-upgrades.md) | NATIVE | Archived native Scout module. Preserve functional design; use cutaway, ship-local interaction footprints and gram-based transfers in 2D. |
| [`current-client-status.md`](current-client-status.md) | PLAYABLE | Source-checked active capability map; visible 2D world is not full game. |
| [`data-recovery-module.md`](data-recovery-module.md) | NATIVE | Archived native gameplay/state; requires a distinct browser session implementation and acceptance. |
| [`documentation-maintenance.md`](documentation-maintenance.md) | GUIDE | Same-PR 2D documentation impact, old universe tooling archived. |
| [`experience-direction.md`](experience-direction.md) | DESIGN | Preserve atmosphere/cinematography goals through 2D shape, color and UI. |
| [`experience-implementation-map.md`](experience-implementation-map.md) | DESIGN | Historical contract or approved intent; do not claim code migration without new issue and browser tests. |
| [`experience-validation.md`](experience-validation.md) | DESIGN | Historical contract or approved intent; do not claim code migration without new issue and browser tests. |
| [`game-systems-overview.md`](game-systems-overview.md) | DESIGN | Product systems overview; native implementation claims must be checked against browser code. |
| [`geological-materials.md`](geological-materials.md) | NATIVE | Archived native environment. Translate published identities, effects and sources into illustrated overhead layers and 2D footprints. |
| [`hall-prototype.md`](hall-prototype.md) | NATIVE | Archived native gameplay/state; requires a distinct browser session implementation and acceptance. |
| [`independent-game-qa.md`](independent-game-qa.md) | GUIDE | Historical contract or approved intent; do not claim code migration without new issue and browser tests. |
| [`jump-drive-and-interstellar-travel.md`](jump-drive-and-interstellar-travel.md) | DESIGN | Approved future Jump Drive/impulse, not browser-delivered. |
| [`legacy-salimon-hub.md`](legacy-salimon-hub.md) | ARCHIVE | Historical contract or approved intent; do not claim code migration without new issue and browser tests. |
| [`macos-qualification.md`](macos-qualification.md) | ARCHIVE | Historical contract or approved intent; do not claim code migration without new issue and browser tests. |
| [`maintenance-map.md`](maintenance-map.md) | GUIDE | Browser modules first; historical crates are not present. |
| [`modular-construction.md`](modular-construction.md) | NATIVE | Archived native Scout module. Preserve functional design; use cutaway, ship-local interaction footprints and gram-based transfers in 2D. |
| [`modular-interior-fixtures.md`](modular-interior-fixtures.md) | NATIVE | Archived native Scout module. Preserve functional design; use cutaway, ship-local interaction footprints and gram-based transfers in 2D. |
| [`multiplayer-and-npc-interaction.md`](multiplayer-and-npc-interaction.md) | DESIGN | Future server authority; browser-local Web Lock is not multiplayer. |
| [`narrative-and-lost-journey.md`](narrative-and-lost-journey.md) | DESIGN | Narrative approved; no production browser lost-journey opening yet. |
| [`non-core-modules.md`](non-core-modules.md) | NATIVE | Archived native Scout module. Preserve functional design; use cutaway, ship-local interaction footprints and gram-based transfers in 2D. |
| [`observed-source-coverage.md`](observed-source-coverage.md) | NATIVE | Archived native universe/catalog/physics implementation. Keep deterministic identities and mathematical contracts for later browser integration. |
| [`opening-and-investigations.md`](opening-and-investigations.md) | NATIVE | Archived native gameplay/state; requires a distinct browser session implementation and acceptance. |
| [`periodic-ci.md`](periodic-ci.md) | ARCHIVE | Historical contract or approved intent; do not claim code migration without new issue and browser tests. |
| [`phase-0-evaluation.md`](phase-0-evaluation.md) | ARCHIVE | Historical contract or approved intent; do not claim code migration without new issue and browser tests. |
| [`phase-0-technical-feasibility.md`](phase-0-technical-feasibility.md) | ARCHIVE | Historical contract or approved intent; do not claim code migration without new issue and browser tests. |
| [`physical-mounts.md`](physical-mounts.md) | NATIVE | Archived native Scout module. Preserve functional design; use cutaway, ship-local interaction footprints and gram-based transfers in 2D. |
| [`physical-organisms.md`](physical-organisms.md) | NATIVE | Archived native environment. Translate published identities, effects and sources into illustrated overhead layers and 2D footprints. |
| [`planetary-atmosphere.md`](planetary-atmosphere.md) | NATIVE | Archived native environment. Translate published identities, effects and sources into illustrated overhead layers and 2D footprints. |
| [`planetary-audio.md`](planetary-audio.md) | NATIVE | Archived native environment. Translate published identities, effects and sources into illustrated overhead layers and 2D footprints. |
| [`planetary-chart.md`](planetary-chart.md) | NATIVE | Archived native environment. Translate published identities, effects and sources into illustrated overhead layers and 2D footprints. |
| [`planetary-climate.md`](planetary-climate.md) | NATIVE | Archived native environment. Translate published identities, effects and sources into illustrated overhead layers and 2D footprints. |
| [`planetary-ecology.md`](planetary-ecology.md) | NATIVE | Archived native environment. Translate published identities, effects and sources into illustrated overhead layers and 2D footprints. |
| [`planetary-environment-art-direction.md`](planetary-environment-art-direction.md) | DESIGN | Approved deterministic geology/weather/ecology/landmarks; adapt to overhead illustrated layers. |
| [`planetary-geology.md`](planetary-geology.md) | NATIVE | Archived native environment. Translate published identities, effects and sources into illustrated overhead layers and 2D footprints. |
| [`planetary-landmarks.md`](planetary-landmarks.md) | NATIVE | Archived native environment. Translate published identities, effects and sources into illustrated overhead layers and 2D footprints. |
| [`planetary-organisms.md`](planetary-organisms.md) | NATIVE | Archived native environment. Translate published identities, effects and sources into illustrated overhead layers and 2D footprints. |
| [`planetary-weather-visuals.md`](planetary-weather-visuals.md) | NATIVE | Archived native environment. Translate published identities, effects and sources into illustrated overhead layers and 2D footprints. |
| [`playable-2d-foundation.md`](playable-2d-foundation.md) | PLAYABLE | Delivered 2D movement, Scout/cockpit handoff, persistent raw mining and local saves. |
| [`product-requirements.md`](product-requirements.md) | DESIGN | Evaluate active browser against future product requirements; no implied native port. |
| [`project-phases.md`](project-phases.md) | GUIDE | Active 2D delivery roadmap; older Phase 0 native results are historical. |
| [`project-qa.md`](project-qa.md) | DESIGN | Historical contract or approved intent; do not claim code migration without new issue and browser tests. |
| [`project-vision.md`](project-vision.md) | DESIGN | Overhead world with cockpit-only 3D radar is the active visual contract. |
| [`radar-navigation-prototype.md`](radar-navigation-prototype.md) | PLAYABLE | Delivered 3D holographic cockpit instrument, not a global player view. |
| [`resource-inventory-and-gathering.md`](resource-inventory-and-gathering.md) | PLAYABLE | 10 kg / 12 slots, integer grams and sample mining; schema-v4 raw ledger and identified 50 kg Scout container domain (#784); transfer UI follows in #785. |
| [`room-finishes.md`](room-finishes.md) | NATIVE | Archived native Scout module. Preserve functional design; use cutaway, ship-local interaction footprints and gram-based transfers in 2D. |
| [`room-lighting.md`](room-lighting.md) | NATIVE | Archived native Scout module. Preserve functional design; use cutaway, ship-local interaction footprints and gram-based transfers in 2D. |
| [`save-contract.md`](save-contract.md) | NATIVE | Historical Rust schema 29+; browser v3 local gram ledger is distinct. |
| [`scout-acoustics.md`](scout-acoustics.md) | NATIVE | Archived native Scout module. Preserve functional design; use cutaway, ship-local interaction footprints and gram-based transfers in 2D. |
| [`scout-and-modular-construction.md`](scout-and-modular-construction.md) | DESIGN | Approved modular structure intent, not a browser construction system. |
| [`scout-interior-art-direction.md`](scout-interior-art-direction.md) | DESIGN | Preserve lived-in Scout; translate furniture/door/window presence into 2D cutaway art and interaction anchors. |
| [`scout-presentation.md`](scout-presentation.md) | NATIVE | Archived native Scout module. Preserve functional design; use cutaway, ship-local interaction footprints and gram-based transfers in 2D. |
| [`spaceship-building-and-survival.md`](spaceship-building-and-survival.md) | DESIGN | Preserve approved ship rules; current 2D damage is ship-wide, raw resources have no volume, machine systems deferred. |
| [`spacesuit-and-hazards.md`](spacesuit-and-hazards.md) | DESIGN | Approved tierless suits, 200 HP exposure model and drawer/module slots; unimplemented. |
| [`suit-observations.md`](suit-observations.md) | NATIVE | Historical contract or approved intent; do not claim code migration without new issue and browser tests. |
| [`surface-age.md`](surface-age.md) | NATIVE | Archived native Scout module. Preserve functional design; use cutaway, ship-local interaction footprints and gram-based transfers in 2D. |
| [`survival-balance.md`](survival-balance.md) | NATIVE | Archived native gameplay/state; requires a distinct browser session implementation and acceptance. |
| [`technical-architecture.md`](technical-architecture.md) | GUIDE | Active browser owners first; Rust crate owners archival. |
| [`terrain-validation.md`](terrain-validation.md) | NATIVE | Archived native environment. Translate published identities, effects and sources into illustrated overhead layers and 2D footprints. |
| [`top-down-art-direction.md`](top-down-art-direction.md) | PLAYABLE | Authoritative overhead illustration, collision footprints, avatar visibility; radar 3D only. |
| [`universe-atmosphere.md`](universe-atmosphere.md) | NATIVE | Archived native universe/catalog/physics implementation. Keep deterministic identities and mathematical contracts for later browser integration. |
| [`universe-contact.md`](universe-contact.md) | NATIVE | Archived native universe/catalog/physics implementation. Keep deterministic identities and mathematical contracts for later browser integration. |
| [`universe-ephemerides.md`](universe-ephemerides.md) | NATIVE | Archived native universe/catalog/physics implementation. Keep deterministic identities and mathematical contracts for later browser integration. |
| [`universe-expansion-approved-design.md`](universe-expansion-approved-design.md) | DESIGN | Retain real-scale deterministic universe and physics; no playable canonical browser integration yet. |
| [`universe-fiction-builder.md`](universe-fiction-builder.md) | NATIVE | Archived native universe/catalog/physics implementation. Keep deterministic identities and mathematical contracts for later browser integration. |
| [`universe-flight-verification.md`](universe-flight-verification.md) | NATIVE | Archived native universe/catalog/physics implementation. Keep deterministic identities and mathematical contracts for later browser integration. |
| [`universe-gaia-ingestion.md`](universe-gaia-ingestion.md) | NATIVE | Archived native universe/catalog/physics implementation. Keep deterministic identities and mathematical contracts for later browser integration. |
| [`universe-gameplay.md`](universe-gameplay.md) | NATIVE | Archived native universe/catalog/physics implementation. Keep deterministic identities and mathematical contracts for later browser integration. |
| [`universe-gravity.md`](universe-gravity.md) | NATIVE | Archived native universe/catalog/physics implementation. Keep deterministic identities and mathematical contracts for later browser integration. |
| [`universe-inference.md`](universe-inference.md) | NATIVE | Archived native universe/catalog/physics implementation. Keep deterministic identities and mathematical contracts for later browser integration. |
| [`universe-navigation.md`](universe-navigation.md) | NATIVE | Archived native universe/catalog/physics implementation. Keep deterministic identities and mathematical contracts for later browser integration. |
| [`universe-observed-ingestion.md`](universe-observed-ingestion.md) | NATIVE | Archived native universe/catalog/physics implementation. Keep deterministic identities and mathematical contracts for later browser integration. |
| [`universe-offline-propagation.md`](universe-offline-propagation.md) | NATIVE | Archived native universe/catalog/physics implementation. Keep deterministic identities and mathematical contracts for later browser integration. |
| [`universe-pack.md`](universe-pack.md) | NATIVE | Archived native universe/catalog/physics implementation. Keep deterministic identities and mathematical contracts for later browser integration. |
| [`universe-reader.md`](universe-reader.md) | NATIVE | Archived native universe/catalog/physics implementation. Keep deterministic identities and mathematical contracts for later browser integration. |
| [`universe-reference-fixtures.md`](universe-reference-fixtures.md) | NATIVE | Archived native universe/catalog/physics implementation. Keep deterministic identities and mathematical contracts for later browser integration. |
| [`universe-scanner.md`](universe-scanner.md) | DESIGN | Approved discovery contract; current scanner uses sample radar not full native catalog. |
| [`universe-sky.md`](universe-sky.md) | NATIVE | Archived native universe/catalog/physics implementation. Keep deterministic identities and mathematical contracts for later browser integration. |
| [`universe-sources.md`](universe-sources.md) | NATIVE | Archived native universe/catalog/physics implementation. Keep deterministic identities and mathematical contracts for later browser integration. |
| [`universe-starting-system.md`](universe-starting-system.md) | NATIVE | Archived native universe/catalog/physics implementation. Keep deterministic identities and mathematical contracts for later browser integration. |
| [`universe-storage.md`](universe-storage.md) | NATIVE | Archived native universe/catalog/physics implementation. Keep deterministic identities and mathematical contracts for later browser integration. |
| [`universe-streaming.md`](universe-streaming.md) | NATIVE | Archived native universe/catalog/physics implementation. Keep deterministic identities and mathematical contracts for later browser integration. |
| [`universe-surface-flight.md`](universe-surface-flight.md) | NATIVE | Archived native universe/catalog/physics implementation. Keep deterministic identities and mathematical contracts for later browser integration. |
| [`universe-terrain.md`](universe-terrain.md) | NATIVE | Archived native universe/catalog/physics implementation. Keep deterministic identities and mathematical contracts for later browser integration. |
| [`universe-trajectory.md`](universe-trajectory.md) | NATIVE | Archived native universe/catalog/physics implementation. Keep deterministic identities and mathematical contracts for later browser integration. |
| [`validation.md`](validation.md) | GUIDE | Node tests, build, Playwright; Cargo/native suites archival. |
| [`verification-workflow.md`](verification-workflow.md) | GUIDE | Browser tests current; Rust/wgpu and M1 claims historical/release only. |
| [`visual-style.md`](visual-style.md) | DESIGN | Mood/colors survive; native low-poly GLB and first-person asset requirements are historical. |
| [`world-and-realism.md`](world-and-realism.md) | DESIGN | 2D camera does not flatten world physics or eliminate canonical 3D universe. |

## Maintaining these contracts

When a 2D issue actually ships, update the affected owner guide, this register row, [current browser status](current-client-status.md) and its evidence report **in the same PR**. Preserve revision-specific Rust test results and approved mechanics. Do not rewrite a documented number, fabricate a catalog/recipe or promote a visual baseline just to declare migration complete.
