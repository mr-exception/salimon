# Strict top-down illustrated 2D direction

**Approved refinement (2026-11-26):** [Browser art direction](browser-art-direction.md) defines semi-realistic illustration, character layers, scale, detail hierarchy, contextual highlights and the previously unresolved low-wall/adaptive occlusion convention. [Production](art-production-guide.md) owns asset specifications; [renderer evaluation](browser-rendering-architecture.md) is conditional, not a migration claim.


**Status: owner-approved art direction (2026-11-04), delivered as the default
browser world.** The Scout cutaway, fixed landing area, outdoor walking, source
inspection/gathering, doorway frame handoff and panoramic cockpit are playable.
See [current scope and source owners](current-client-status.md). The isolated
`/?scene=top-down` development fixture remains inspection-only; it does not
represent the full default `/` gameplay entry.

## View and illustration

- **Universal ship-up rule:** the active spaceship is always vertical in the 2D view, bow pointing to the top of the screen, indoors and outdoors. Rotate the scene basis consistently; preserve physical headings, world anchors and collision geometry. W/A/S/D follows screen directions.
- **Strict overhead top-down**, not isometric or three-quarter. Ground positions, collision footprints, collection ranges and rendering alignment must refer to the same top-down world space.
- **Stylized illustrated 2D** sprites and environmental layers, with strong silhouettes, controlled textures, readable materials and restrained highlights/shadows. No pixel-art requirement or mandatory decorative 3D meshes.
- Avoid visual clutter and decorative geometry that hides important objects or changes gameplay collision. Ensure legibility at normal gameplay zoom.
- Walls, doors, furniture and tall objects need a consistent cutaway or occlusion treatment so the character never disappears behind them. Exact fading and wall-height conventions remain to be designed.

## Scout, planets and characters

- Retain the existing approved Scout identity: cozy, personal but well maintained, with graphite/slate machinery, warmer ivory habitation, ochre/terracotta accents and cyan functional indicators. Preserve layout intent, privacy screen, observation window/shutter, physical furniture, localized aging and clean repairs.
- Preserve deterministic canonical geology, climate, weather, ecology, rare biological signatures and landmarks on planets. Empty worlds must remain visually interesting. Present these in illustrated overhead layers, not arbitrary independent AI-generated background pictures.
- Characters and tools use coherent overhead silhouettes and motion. Distinct states include walking, mining/harvesting, machine interaction and future zero-G. Raw resource gathering does not attach visible carried fragment sprites.
- Raw resources and count-based items share [one 10 kg / 12-slot weight-based inventory](resource-inventory-and-gathering.md), with no volume cap. Tierless modular suits and their storage/hazard systems are [future design](spacesuit-and-hazards.md); installed fixtures stay separate until interactive.

## 3D navigation consistency

- **Strict 2D is the default world view.** The production cockpit opens **360° panoramic observation** at the **cockpit desk**, with independent inertial camera, compact instruments and shared physical ship location. The local player remains seated at the desk in 2D; multiplayer presence remains future work. See [cockpit states and multiplayer reconnect](cockpit-navigation-and-reconnect.md).
- The isolated `/?scene=radar` engineering fixture retains faceted contact proxies and a sparse galactic lattice; it is not the production cockpit.
- Share instrument typography, functional accent colors, visual warning language, restrained animation and material sensibility between navigation and 2D exploration. The production cockpit uses optical sky and scanner-gated contacts without legacy heading/velocity rays. The single five-second scanner pulse remains unchanged.

The former 3D low-poly art bible and first-person interior/environment camera contracts are legacy implementation references. Their mood, narrative and world-identity decisions remain useful, but incompatible camera, mesh and fragment-specific presentation rules are superseded.

## Shared HUD art direction (#827)

The owner-approved [Expedition Interface guide](hud-ui-style-guide.md) selects **hybrid industrial equipment surfaces + restrained projected instrumentation**, with an **adaptive compact** 2D survival/inventory/context HUD. The production 2D HUD now applies this direction through compact vitals/capacity and on-demand inventory/help. Gameplay mechanics remain unchanged. Retain all authoritative survival, weight/slot, mining, resource and interaction values while consolidating visual primitives and preserving the full viewport.

## World viewport

The playable 2D world fills the available browser viewport. Survival, inventory,
interaction/status feedback and key hints sit on translucent HUD panels over the
scene; decorative titles, borders and footer bars must not reserve world space.
Preserve overhead proportions by expanding the visible world to the window's
aspect ratio, with the followed character clear of the HUD. See the
[viewport/input contract and verification](playable-2d-foundation.md#viewport-and-hud-803).
