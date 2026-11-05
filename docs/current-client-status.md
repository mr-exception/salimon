# Current browser implementation and contract routing

The [full 2D migration register](2d-migration-register.md) classifies every documentation file, maps historical 3D concepts onto active 2D design, and identifies the unfinished browser adapters.

Checked against `client/src` on 2026-11-05. The default `/` is the playable
React/TypeScript 2D Scout/planet world. The 3D radar opens at its cockpit; the
illustrative radar ship does not relocate the authored 2D landing site.

Start with the [playable journey](playable-2d-foundation.md),
[client setup and verification](../client/README.md), and
[active architecture owners](technical-architecture.md#active-browser-world-and-navigation).
The [docs index](README.md) includes both approved designs and historical contracts;
an issue closure or native implementation note does not establish browser delivery.

## Code-checked reconciliation

All source paths below are relative to `client/src/`. “Approved future” records
requirements, not playable behavior. “Decision pending” identifies missing rules.

| System | Browser-delivered evidence | Approved future / decision pending / legacy boundary |
| --- | --- | --- |
| 2D navigation and cockpit | `topDownRuntime.ts`, `topDownTraversal.ts`, `cockpit.ts`, `TopDownView.tsx`: continuous walking, collision, doorway frames, owner chair, radar entry/exit; `navigationRuntime.ts` continues simulation | Local power fixture is not Core power. No flight relocation of the landing site. [Cockpit/reconnect contract](cockpit-navigation-and-reconnect.md) separates future authority |
| Mining | `resourceSession.ts`, `planetarySite.ts`: two fixed sources, up to 1,000 g per completed second, interruptions and depletion | Full catalog, tools and balancing pending; native physical fragments are legacy-only |
| Inventory | `resourceInventory.ts`: 10,000 g / 12 shared slots, one raw stack per type, count-item mass/stack validation; `resourceSession.ts`: v6 ledger/item-definition/survival save, lossless v1–v5 migration | Generic item transactions and unified count/mass/slot UI are delivered; production item content, equipment UI and upgrades remain pending. [Weight-only contract](resource-inventory-and-gathering.md) supersedes volume limits |
| Ship storage | `scoutResourceContainer.ts` declares the 50 kg raw-only `scout:storage`; `resourceContainerInteraction.ts` + `ResourceContainerPanel.tsx` provide E/Use, exact-gram review/confirmation and capacity/save feedback | Atomic domain transactions and schema-v5 saves persist both endpoints. Focus/Escape/reach/scene cancellation preserves commits; no container slot cap, count-item content or machine buffers |
| Suits | No suit/equipment/hazard runtime in `client/src` | [Approved tierless suits and hazard design](spacesuit-and-hazards.md): initial basic suit is a requirement, not a shipped grant; exact masses, recipes and switching rules pending |
| Hazards and health | `survival.ts` provides 200 HP and terminal local-run state; no exposure/module-durability runtime | Preserve **200 HP** base maximum and the approved exposure/protection formulas; implementation and remaining tick/category rules pending |
| Core | Visible inspection fixture; radar energy is unlimited (`simulation.ts`) | Continuous mass-based conversion is approved future; yields, watts, priority and batch bookkeeping pending. Native conversion defaults are not browser balance |
| Fabricator | Visible inspection fixture | Approved queue/power/tray design remains future browser integration; recipes and count-item content are not implemented |
| Survival | [Browser survival](browser-survival.md): active hunger/fatigue, starvation, passive regen, offline fatigue, persisted HUD and explicit terminal reset | Historical Rust survival implementations are legacy-only; suit/hazard requirements govern future 2D work |
| Multiplayer | `cockpit.ts` is explicitly local; `TopDownView.tsx` uses a Web Lock for one local save writer | Server authority, presence, reconnect and offline death are future; no backend sync/anti-cheat claim |
| Universe | `sampleUniverse.ts`, `scanner.ts`, `contact.ts`: illustrative radar catalog, scanning, conservative contact; `planetarySite.ts`: one fixed authored site | Canonical generation, scientific catalogs, travel/opening narrative and full universe integration remain future browser work; native catalog claims are historical |
| Ship damage | No 2D damage/destruction runtime | [Approved 2D damage](cockpit-navigation-and-reconnect.md#current-ship-damage-scope-simplification) is **ship-wide**. Localized room/module damage is legacy/deferred, never silently restored |

## Legacy contracts and supersession

The native Rust source, models and tools are absent from the active checkout.
The [pinned historical tree](https://github.com/mr-exception/salimon/tree/8f112bb6e6be9f813174efb47d706a8601930d04)
retains them. [Maintenance map](maintenance-map.md) routes active browser owners
first and labels the old crate map; the historical sections of the
[root README](../README.md#historical-native-reference),
[architecture](technical-architecture.md#historical-native-architecture) and
[phases](project-phases.md#historical-native-post-baseline-prototype) remain accessible.
Historical mechanics, numbers and reports are preserved as revision-specific records.
Current weight-only inventory, ship-wide damage and 200 HP hazard design take
precedence over incompatible native/earlier designs. No approved number changes here.

## Browser/cloud validation

Use Node >=22.18, `cd client`, `npm ci`, `npm test`, `npm run build`,
`npx playwright install --with-deps chromium`, and `npm run test:browser`.
The [Navigation client workflow](../.github/workflows/navigation-client.yml) also
runs negative mutations, repeat visual comparisons and baseline approval.
Use affected suites during iteration; honor each issue's mandatory matrix.
Documentation-only changes require links/semantic checks; browser/runtime/E2E
checks are N/A when behavior is unchanged. Native/M1/Metal checks do not gate
routine 2D cloud delivery. Synthetic input and software rendering establish only
their actual context, not physical hardware performance.

Review code, criteria and actual evidence in the active implementation chat;
no independent reviewer or retired automated QA gate is required. Preserve failed
and unrun checks. Human retains baseline approval and merge authority.
