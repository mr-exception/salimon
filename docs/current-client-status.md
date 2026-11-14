# Current browser implementation and contract routing

> **Shared UI foundation (#826), compact 2D HUD (#827) and panoramic styling (#828):** the [Expedition Interface HUD/equipment style guide](hud-ui-style-guide.md) specifies the shared industrial/projected visual system. Scoped tokens and typed primitives now have a development-only light/dark gallery (`client/src/ui`, `tests/ui.html`); the production 2D HUD now consumes its tokens, meters, progress, inventory utility and help dialog. Production panoramic instruments now consume the shared control, utility-panel and dialog skins, with opaque readouts and warning summaries inside help; compact survival/help/pause remain available in the cockpit. The storage dialog uses the shared industrial shell (#829), and cross-view focus/feedback uses shared priority and Escape owners (#830). The deterministic gallery and cross-view screenshot matrix (#831) extend existing Playwright coverage; see the [verification guide](../client/README.md#cross-view-expedition-interface-visual-regression-831). See [#828 evidence](../reports/issue-828/README.md). No new gameplay mechanics follow from the style approval.

> **Panoramic cockpit (#815):** Default `/` opens the shared grounded panoramic observation scene at the Scout desk. The isolated `/?scene=radar` keeps the orthographic free-flight engineering fixture. See [the delivered integration](../client/README.md#cockpit-gameplay-766).

The [full 2D migration register](2d-migration-register.md) classifies every documentation file, maps historical 3D concepts onto active 2D design, and identifies the unfinished browser adapters.

Checked against `client/src` on 2026-11-09. The default `/` is the playable
React/TypeScript 2D Scout/planet world. The 3D panoramic scene opens at its cockpit and shares the grounded Scout / Vesper surface
anchor with the 2D world. Authorized pilots can close the aft door and start an uncancellable six-second takeoff, then steer and use signed thrust in the same panoramic scene (#841). Landing remains future work.

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
| 2D navigation and cockpit | `topDownRuntime.ts`, `topDownTraversal.ts`, `cockpit.ts`, `TopDownView.tsx`: continuous walking, collision, doorway frames, owner chair, panoramic entry/exit, session-local observation persistence; `scoutLocation.ts` binds world frames and radar physical state; `navigationRuntime.ts` continues time/scans while grounded | Local power fixture is not Core power. Explicit lifecycle endpoints and ship/scanner saves are delivered (#838); airborne interior detaches the exterior. The aft boarding door is a real E/mouse interaction with a saved v10 open/closed state, matching collision/art and flight lock (#839). Launch requires a live authorized powered seat, a validated grounded site, all occupants aboard, a closed door and no conflicting work/maneuver. The deterministic uncancellable 360-tick takeoff trajectory is delivered (#840), including thrust/mass feasibility, swept admission and validated save/resume; production Take off admission/progress and flying-only controls are wired (#841). The world owns the only clock, including held cockpit steering; exit to the detached 2D interior preserves flight and selected throttle. [Cockpit/reconnect contract](cockpit-navigation-and-reconnect.md) separates future authority |
| Mining | `resourceSession.ts`, `planetarySite.ts`: two fixed sources, up to 1,000 g per completed work interval, approved survival work-rate penalties and persisted partial work | Full catalog, tools and balancing pending; native physical fragments are legacy-only |
| Inventory | `resourceInventory.ts`: 10,000 g / 12 shared slots, one raw stack per type, count-item mass/stack validation; `resourceSession.ts`: v10 ledger/item-definition/survival/hazard/Scout save, lossless v1–v7 migration | Generic item transactions and unified count/mass/slot UI are delivered; production item content, equipment UI and upgrades remain pending. [Weight-only contract](resource-inventory-and-gathering.md) supersedes volume limits |
| Ship storage | `scoutResourceContainer.ts` declares the 50 kg raw-only `scout:storage`; `resourceContainerInteraction.ts` + `ResourceContainerPanel.tsx` provide E/Use, exact-gram review/confirmation and capacity/save feedback | Atomic domain transactions and schema-v10 saves persist both endpoints. Focus/Escape/reach/scene cancellation preserves commits; no container slot cap, count-item content or machine buffers |
| Suits | No suit/equipment UI or production gear catalog in `client/src` | [Approved tierless suits and hazard design](spacesuit-and-hazards.md): initial basic suit is a requirement, not a shipped grant; exact masses, recipes and switching rules pending |
| Hazards and health | `survival.ts` provides 200 HP and terminal local-run state; `hazards.ts` adds exact temperature, radiation/poison exposure and 1,000 HP module budgets via schema-v10 atomic saves | [Browser hazard domain](browser-hazards.md) has no production source/module defaults; planetary sampling, equipment and unspecified categories remain pending |
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
The [PR Navigation client workflow](../.github/workflows/navigation-client.yml)
runs domain tests, build/typecheck and four focused browser smoke cases. The
[daily full workflow](../.github/workflows/navigation-client-full.yml) preserves
every browser project, negative mutation and repeated visual comparison in five
parallel groups. Scheduled/manual failures create or update a bug per failing
group, with revision and run/artifact links; smoke green is not full-suite acceptance. The owner removed the
human baseline-approval job in PR #834; deployed-version visual review and
follow-up issues replace that gate.
Use affected suites during iteration; honor each issue's mandatory matrix.
Documentation-only changes require links/semantic checks; browser/runtime/E2E
checks are N/A when behavior is unchanged. Native/M1/Metal checks do not gate
routine 2D cloud delivery. Synthetic input and software rendering establish only
their actual context, not physical hardware performance.

Review code, criteria and actual evidence in the active implementation chat;
no independent reviewer or retired automated QA gate is required. Preserve failed
and unrun checks. The owner retains deployed-version visual review and merge authority.

The #810 perspective camera foundation is available in development fixture
`/tests/observation.html`: inertial full-sphere observation, ship-relative origin,
75°–0.1° optical zoom and session-local snapshots. It is now composed with the physical sky/HUD in the production cockpit by #815.
See the [camera API and checks](../client/README.md#perspective-observation-camera-foundation-810).

### Physical optical sky foundation (#811)

A reusable scanner-independent `client/src/OpticalSky.tsx` now provides physical
angular sphere rendering, diffuse terminators, unresolved point spread and a
sparse deterministic starfield. `opticalSky.ts` owns split-origin subtraction and
stable near-surface ray geometry. The development-only optical fixture exercises
75°/0.1° free-space views and Vesper's shared `scoutLocation.ts` grounded anchor,
including real globe/horizon occlusion. No discovery names or actions are emitted.
Production now composes this sky with #812 scanner overlays and #815 cockpit lifecycle. See the
[client optical guide](../client/README.md#physical-optical-sky-foundation-811)
and [local evidence](../reports/issue-811/README.md).

### Perspective sensor overlay foundation (#812)

The separate `/tests/perspective.html` development fixture composes the optical
scene/camera with scanner-authorized markers and a searchable, paginated contact
browser. Physical line-of-sight tests suppress occluded/behind-camera/off-screen
marker hit targets; those scanned IDs stay in the browser. Live guards remove
stale selection actions on range loss. Production composes this overlay through #815; the standalone orthographic fixture remains isolated.
See the [overlay API](../client/README.md#perspective-sensor-overlay-foundation-812)
and [local evidence](../reports/issue-812/README.md); remote acceptance is pending.


### One-time perspective contact observation (#813)

The reusable perspective overlay now offers scanner-guarded Look at (camera)
and Set direction (ship), with separate cancellation and live flight gates.
Look at captures direction once, preserves zoom/ship motion, and deterministically
handles opposite directions. Shared grounded Scout permits other detected bodies
while identifying Vesper and disabling self-body actions. Scanner loss removes
commands without a selectable ghost; foreground optical geometry remains opaque.
The development-only `/tests/look-at.html` verifies flying and shared grounded
composition; #815 also integrates this into the default grounded cockpit.
See [API/help](../client/README.md#one-time-contact-observation-813) and
[local evidence](../reports/issue-813/README.md); CI awaits the user's report.
