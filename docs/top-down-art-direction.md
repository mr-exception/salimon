# Strict top-down illustrated 2D direction

**Status: owner-approved art direction (2026-11-04).** Not yet a playable 2D implementation; the existing browser client is a separate 3D radar prototype.

## View and illustration

- **Strict overhead top-down**, not isometric or three-quarter. Ground positions, collision footprints, collection ranges and rendering alignment must refer to the same top-down world space.
- **Stylized illustrated 2D** sprites and environmental layers, with strong silhouettes, controlled textures, readable materials and restrained highlights/shadows. No pixel-art requirement or mandatory decorative 3D meshes.
- Avoid visual clutter and decorative geometry that hides important objects or changes gameplay collision. Ensure legibility at normal gameplay zoom.
- Walls, doors, furniture and tall objects need a consistent cutaway or occlusion treatment so the character never disappears behind them. Exact fading and wall-height conventions remain to be designed.

## Scout, planets and characters

- Retain the existing approved Scout identity: cozy, personal but well maintained, with graphite/slate machinery, warmer ivory habitation, ochre/terracotta accents and cyan functional indicators. Preserve layout intent, privacy screen, observation window/shutter, physical furniture, localized aging and clean repairs.
- Preserve deterministic canonical geology, climate, weather, ecology, rare biological signatures and landmarks on planets. Empty worlds must remain visually interesting. Present these in illustrated overhead layers, not arbitrary independent AI-generated background pictures.
- Characters and tools use coherent overhead silhouettes and motion. Distinct states include walking, mining/harvesting, machine interaction and future zero-G. Raw resource gathering does not attach visible carried fragment sprites.
- Raw materials use [the 10 kg / 10 L resource inventory](resource-inventory-and-gathering.md). Keep installed fixtures and personal non-resource objects separate until their top-down interactions are specified.

## 3D navigation consistency

- The navigation mode stays a **simple orthographic 3D holographic radar** with faceted contact proxies and a sparse galactic lattice; it is not a realistic 3D spaceship scene.
- Share instrument typography, functional accent colors, visual warning language, restrained animation and material sensibility between navigation and 2D exploration. The navigation heading/velocity rays and scanner rules remain unchanged.

The former 3D low-poly art bible and first-person interior/environment camera contracts are legacy implementation references. Their mood, narrative and world-identity decisions remain useful, but incompatible camera, mesh and fragment-specific presentation rules are superseded.
