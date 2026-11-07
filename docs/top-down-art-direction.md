# Strict top-down illustrated 2D direction

**Status: owner-approved art direction (2026-11-04), delivered as the default
browser world.** The Scout cutaway, fixed landing area, outdoor walking, source
inspection/gathering, doorway frame handoff and cockpit radar are playable.
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

- **Strict 2D is the default world view.** The navigation mode remains a **simple orthographic 3D holographic radar** opened by interacting with the **cockpit desk**, not a global view toggle. Other connected players see the active pilot seated at the desk in 2D. See [cockpit states and multiplayer reconnect](cockpit-navigation-and-reconnect.md).
- The radar retains faceted contact proxies and a sparse galactic lattice; it is not a realistic 3D spaceship scene.
- Share instrument typography, functional accent colors, visual warning language, restrained animation and material sensibility between navigation and 2D exploration. The navigation heading/velocity rays and scanner rules remain unchanged.

The former 3D low-poly art bible and first-person interior/environment camera contracts are legacy implementation references. Their mood, narrative and world-identity decisions remain useful, but incompatible camera, mesh and fragment-specific presentation rules are superseded.

## Shared HUD art direction (future implementation)

The owner-approved [Expedition Interface guide](hud-ui-style-guide.md) selects **hybrid industrial equipment surfaces + restrained projected instrumentation**, with an **adaptive compact** 2D survival/inventory/context HUD. This is a proposed presentation refinement, not a change to currently playable panels or their gameplay mechanics. Retain all authoritative survival, weight/slot, mining, resource and interaction values while consolidating visual primitives and preserving the full viewport.

## World viewport

The playable 2D world fills the available browser viewport. Survival, inventory,
interaction/status feedback and key hints sit on translucent HUD panels over the
scene; decorative titles, borders and footer bars must not reserve world space.
Preserve overhead proportions by expanding the visible world to the window's
aspect ratio, with the followed character clear of the HUD. See the
[viewport/input contract and verification](playable-2d-foundation.md#viewport-and-hud-803).
