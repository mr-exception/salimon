# Game experience direction: grounded cinematic Salimon

Status: **approved experience direction; design guidance and planned work, not shipped native art signoff**. Documentation issue #458. Complements the authoritative [low-poly asset bible](visual-style.md), does not replace its geometry, palette, Blender, physical interfaces or validation rules.

## Player feeling and artistic identity

Salimon should convey solitude, enormous physically plausible distances, curiosity, vulnerability, personal persistence and mystery. The playable world can contain many living alien planets, ships, stations and civilizations, yet enormous separation makes encounters uncommon; **thousands of inhabitants do not make the universe feel crowded**. Do not fill every empty world with decorative props or scripted company. An empty horizon is meaningful when intentional, legible and contrasted with recognizable landmarks, light, terrain and sound; it is a defect when the environment reads as an unlit proof-of-concept.

**The Scout is home**: familiar, warm, physically credible, used but not messy for its own sake. Exterior environments are less predictable, more exposed, unfamiliar and often nearly silent. Contrast means the same person feels protected on returning to the cabin. Reference mood: grounded cinematic sci-fi rather than colorful arcadey space fantasy or ultra-photorealistic geometry. Existing stylized low-poly topology remains a technical and stylistic baseline; improved composition, motion, lighting and audio—not polygon proliferation—must deliver cohesion.

## Pillars and measurable review expectations

1. **Functional credibility:** the Core, Fabricator, cockpit, engine, boarding ramp, cargo, scanner and Archive each have physical place, mechanics, access, and state-readable feedback; avoid freestanding HUD puzzles.
2. **Atmosphere and composition:** intentional camera sightlines, warm/cool zoning, hierarchy from dominant silhouettes to usable controls, near/far contrasts and subdued exposure around luminous Core and monitors. A Blender beauty render cannot certify native wgpu behavior; verify actual shader capabilities. Contrast and information must survive blackout.
3. **Reactive machines:** animated states for startup/shutdown, thrust, doors/landing gear, extraction, feeding Core, Fabricator processing, disabled/loading/failed equipment. All derive from authoritative domain snapshots, not independent renderer simulation.
4. **Sound and silence:** distinguish engine vibration, hull resonance, interior footfalls, planet/EVA vacuum behavior, Fabricator, Core and alert states; spatial attenuation, accessible alternatives, mix controls. Do not layer constant music or chatter over purposeful silence. Survival audio #383 is a related existing issue, not an invitation to duplicate it.
5. **UI coherence:** typographic scale, focus, context prompts, cockpit/physical displays, error and status states form a shared language with restrained accents and accessible non-color-only signals. No Hall endgame spoiler.
6. **Environmental identity:** planetary skies, geological forms, terrain color/lighting and environmental sound should differ by location, with credible rarity of recognizable wrecks, traces, anomalous signals and landmarks. Distinctive planets do not require NPCs at every stop.
7. **Mystery as evidence:** clues are encountered through physical places and imperfect records, without unsupported definitive claims.

## First reference vertical slice: Scout interior and every onboard interaction

The initial review target is the complete Scout, not a new planet biome or an isolated beauty shot:
- Seated and standing cockpit: wide monitor, lower windows and navigation readability; pilot exit/return; powered, powerless and landed/in-flight states.
- Core: framed physical monitor, live power/conversion telemetry, fragment intake, charging and blackout feedback.
- Fabricator: onboard input, supported/locked recipes, processing, pause/blocked state, output tray and collection.
- Aft door and walkable boarding ramp, internal lanes, loose physical cargo, contextual hand actions.
- Archive/communications terminal and eventual recovery equipment, integrated like machinery rather than a floating game-menu screen.
- Scanner hologram when future #430 lands, without changing its five-second pulse rule.
- Accessible context prompts, consistent interaction UI, predictable transitions and real-time sounds/animation.
- Clear separation between a single whole-scene art target and independent crate/code responsibilities.

**Experiential reference loop:** leave warm cabin → survey strange horizon → mine a resource → carry it through the ramp → physically feed Core/Fabricator → observe believable audio/visual feedback → examine recovered log → return to cockpit and leave. This is a validation journey, **not a compulsory scripted quest**.

## Acceptance and review

Maintain repeatable authored/reference cameras for cockpit, Core, Fabricator, door/ramp, cargo, exterior and transitions; record conditions and revision. Compare gameplay captures, not just offline Blender previews. Include 1080p/readability, scene memory and draw cost, GPU/CPU frame timing, LOD, focus handling, lighting when Core dies, accessibility settings and quiet periods. Performance target and actual reference Apple M1/Metal measurements must be distinguished from CI/Linux results. Integrate with existing #273–279, #294/#299, #308/#314, #342–348, #451 and proposed #463–466. Work involving the future alien homeworld references #412/#424–426.

## Anti-goals

No random emissive strips, endless information HUDs, ungrounded NPC density, fake weather everywhere, shader features presumed implemented, external game engine, forced narrative campaign, renaming of physical assets just for style, or premature Absenat spoilers.

## Future quantum Jump Drive staging — approved mechanic, not a shipped art asset

The [Spatial Jump Drive travel contract](jump-drive-and-interstellar-travel.md) requires a **quantum-themed loading environment containing the same ship and onboard occupants**, shown **after** an explicitly engine-off, owner-authorized, sufficiently funded and destination-validated jump. The staging should feel like deliberate extraordinary engineering rather than a generic spinner or guaranteed lore wormhole; it remains a **presentation of a single transactional relocation**, not a second world/time scale, alternate multiplayer timeline or proof of the Absenat mystery. Preserve readable ship interfaces, power/jump progress/error information, accessible effects and the low-poly grounded visual bible; avoid inventing shader capabilities or finalized effect palettes without native signoff. The normal seamless space-to-surface flight art direction is unchanged and does not inherit a loading transition. Actual art/mockup and wgpu/M1 validation belong to future implementation, not this documentation-only update.
