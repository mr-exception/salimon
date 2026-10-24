# Scout interior art direction and habitation presentation

**Status: owner-approved future art/experience target, 2026-10-23.** This is a design and future implementation contract, **not** a claim that these furnishings, customization, lighting, acoustics, aging or damage rendering currently ship. No source geometry, physics, power, survival or verified performance contract changes merely because this document exists.

Companion contracts: [approved starter Scout and modular construction](scout-and-modular-construction.md), [grounded cinematic experience direction](experience-direction.md), [low-poly visual style](visual-style.md), and [present Scout renderer/audio foundation](scout-presentation.md). Where a feature overlaps these, retain their existing authoritative mechanics, geometry and verification rules. This document refines the *art direction, tactile habitation and interior presentation*. Explicit decisions in this document supersede prior brainstorming alternatives (notably clean repairs vs visible repair patches).

## Artistic goal and starting identity

**The permanent Scout is home**: a compact, reliable, already lived-in but well-maintained expedition ship. It must feel comfortable without becoming a luxury yacht, rusty wreck, sterile laboratory or predetermined biography. Character comes from functional architecture, carefully restrained wear, real physical possessions, warm habitation contrasted with colder/industrial working spaces, responsive machinery, tactile motions and meaningful silence.

- Grounded cinematic **stylized low-poly**; broad readable volumes, deliberate clipped corners/bevels, coherent supports/hinges/service access and high-value contact details. Never equate quality with subdivision, excessive rivets, glowing stripes or uncontrolled material diversity.
- Default starter belongings are modest and generic: made bunk, secured drinking cup, simple utensils and maintenance equipment, available shelves/display space. No authored family photos, private diaries or lore props that dictate a particular occupant's history or spoil later story.
- Physical user actions make the cabin increasingly personal. No auto-generated trophies, clutter, compulsory housekeeping, dirt meter or automatic decoration progression.
- All important interactive objects have discoverable silhouettes and concise grounded prompts; key safety/functional cues are not color-only. The UI is not a separate decoration mini-game.

## Starter layout and space allocation

The existing cockpit, wide monitor, lower forward windows, Core, wall Fabricator, Archive and rear boarding ramp remain genuinely functional. The future starter requires an integrated main cabin and a *separate aft physical cargo bay*, divided by a full-height structural bulkhead and a wide closable internal door. Keep a continuous carry/walk route between ramp, cargo, processing machinery and cockpit. Do not introduce an installed Jump Drive at game start.

### Habitation fixtures

| Fixture | Design and behavior |
| --- | --- |
| Sleeping nook | Semi-private recessed/side area within main cabin, with fold-away bunk, mattress, pillow, secured/stowed bedding, local reading light, small shelf and personal locker |
| Privacy | Segmented manually sliding/folding mechanical screen, operable during blackout; does not block central passage, bunk deployment or utility access |
| Dining/work area | Small fold-away table and **fold-away padded seat with visible safety restraint**; independent deployment and natural sit/stand/free-look transitions |
| Observation | **One primary, reinforced observation window between bunk and table**, sightline from both when privacy screen is open; **manually operable protective shutter** retained through saves; glazing shows actual exterior scene rather than a fake monitor |
| Secondary glazing | Selective smaller side/aft windows only where useful and geometrically compatible with storage, privacy, structural framing and rails; preserve cockpit's functional forward/lower glazing |
| Galley | Compact preparation surface, food/water equipment, secured cup/utensils, cabinets; reuses actual Fabricator/food mechanics rather than introducing a new recipe or water simulation |
| Hygiene | Small private toilet/washbasin/mirror fixture, secured storage and task light; lightweight optional wash/use actions, **no new hygiene needs, cleaning tasks, water simulation or survival meter** |
| Personal storage | Bedside niche, real-volume locker, galley cabinets and small display shelves around table/window; heavy ore/materials belong in physical cargo room |
| Engineering/cargo | Clear Core/Fabricator/Archive service space; exposed heavier rails/tool mounts in engineering, rugged racks/restraints and reinforced floor in cargo |

Do not freeze a sketch as measured spatial truth. Validate actual dimensions, seated/lying viewpoints, player 1.8 m-scale navigation, carry clearance, headroom, frame/window intersections, hinges' swept volumes, optional installed furniture and future 3 m module interfaces. No rounding of precise existing anchors to the modular grid without measured migration.

## Furnishings, rails, personal objects and attachment

Three asset categories: **immutable ship architecture** (bulkheads, panels, fixed access); **installed physical fixtures** (bunk, galley, lighting, lockers, rails, seats, cabinets); **independently addressable belongings** (tools, cups, artifacts, mineral samples, loose cargo). Every apparent physical placement has real ownership/shape/position, not a hidden decoration inventory.

- Free placement: carry **one world object at a time**, aim and place it on a valid physical surface respecting dimensions/collision, or drop it normally. Do not replace the object with a generic decoration copy. Preserve stable ID, resource type, mass, age and damage state.
- **Securing is always explicit**: after placing an item near a compatible holder, clamp or cargo restraint, interact to secure it, with a small mechanical latch motion/sound. Releasing is another explicit action. A displayed shelf alone is not auto-secured cargo.
- Mounts are *physical attachments*, with stable parent-child relationships (ship/room -> rail/furnishing -> mount -> item). Objects remain individually retrievable and contribute mass exactly once. Attaching doesn't erase identity; blackouts do not release ordinary mechanical locks.
- Rails are standardized **small/fine installation interfaces**, independent of the ship's main 3 m structural-cell grid. **Habitation rails** are recessed into wall/panel seams, visually discoverable through locking notches/end-caps and optional covers; **engineering/cargo rails** are exposed and reinforced. Not every wall must be covered in rails.
- Compatible shelves, organizers, cup holders, display clamps and tool mounts can be installed at suitable rail positions. Constructing a *new* fixture follows the existing placeholder + physically supplied resources + automatic short build process; an empty movable accessory can be unfastened, carried and reattached at a compatible position without repaying its construction cost. Heavy/structural parts have stricter moving/safety rules.
- Cabinet, locker and bin contents are individual physical objects, never a virtual list. A closed latched cabinet is a real containing volume; when opened in zero-G, unsecured contents can drift. Don't magically protect open bins from physics.
- Folding/moving furniture refuses/interrupts an obstructed path and gives a brief contextual explanation; do not silently swallow, teleport or destroy personal objects. A room being removed or detached must handle all physically attached contents and occupancy safely.
- Persist physical transforms, room/rail/mount/item relationships, stowed/open states, shutter and screen positions, actual cargo and all component identities through save/load and lawful modular building.

## Materials, palette, cosmetic customization

One coherent manufactured family across old and newly built modules. Structural frames use **graphite** and industrial slate; habitation has warmer **ivory** composite wall panels, matte surfaces and limited **terracotta** upholstery/fabrics; **copper/ochre** gives restrained mechanical contact detail; **cyan** is predominantly active status/equipment display, not decorative neon. Authoritative swatch candidates remain in visual-style.md and existing ship authoring; extra colors and exact values require actual game-rendered approval. Use shared meshes/materials and economical per-instance parameters.

- Each physical room controls *five independent cosmetic color groups*: **walls, floor, trim, cabinetry, upholstery**. Each individual installed furniture item may also have compatible color overrides. Avoid arbitrary material-family change (e.g. mattress to chrome).
- Cosmetic recoloring, local lighting preferences and preset selection are **free and instant**. Physically replacing floor panels, wall coverings, upholstery components or installed furniture uses approved resource-fed construction rules. No cost for changing a color setting.
- Group changes are physical-room-local, not ship-wide. New constructed rooms begin with function-appropriate compatible defaults; removed partitions do **not** erase or auto-merge adjacent room color histories.
- Recoloring leaves ordinary age and actual damage intact. New physical replacement components start their own fresh age.
- Functional warning/screen colors, safety markings, mechanical supports and mount discoverability stay legible under every permitted palette, including blackout. Resource colors keep their gameplay meanings.

## Lighting and selective glazing

**Both room-level lighting and individual fixture overrides are required.**

- Room Work / Relax / Night / Custom presets, master On/Off, general brightness and warmth. Individual reading/table/galley/hygiene/task fixtures can be overridden while the room is enabled; the room master Off wins for customizable room lights.
- Fixture power and actual damage/disabled state override all preferences. Power restoration applies the stored valid preferences without inventing backup batteries. Critical cockpit/Core/Fabricator warnings and active status displays retain their own authoritative behavior.
- Visible lamps should drive credible low-cost localized illumination and material response. Aim for restrained contact/shadow grounding and readable dark surfaces rather than photorealistic global illumination, unrestricted bloom, real-time ray tracing or an unapproved PBR workflow.
- Primary observation window is a real reinforced hull opening with a **manual protective shutter**; opening it reveals the true outside scene. Sun/window illumination should change nearby tables, bunk and panels plausibly based on direction and shutter state. Natural exterior light may continue while Core power is lost.
- Secondary cabin glazing is selective; functional cockpit windows remain. Privacy screen partially controls sightlines, not window shutter. No fake emergency electrical lights: existing unpowered readability floor is a rendering/accessibility safeguard, not a powered system.
- Each future modular room keeps its own light configuration and installed fixture states, including after joining adjacent rooms; actual fixture energy draw needs separate authoritative approval (do not invent joule costs).

Minimum target states: sunlit landing + shutter open/closed, dark-space travel, warm resting preset with brighter reading light override, room master Off, Core blackout on a sunlit surface, deep-space blackout, power restoration, and broken fixture. Validate matching actual player viewpoints.

## Cosmetic aging, damage and **clean repairs**

Track cosmetic age and real gameplay physical condition separately.

- **Moderate localized age** increases very slowly with relevant passage of time, and **accelerates with actual component use**. Examples: hinged/shutter/locker handle wear on repeated operations; bunk/seat fabric wear through use; Fabricator operation/heat changes around vents; rail fastening contact marks; cargo/walking scuffs in traffic paths.
- Surface-specific appearance: metal edges polish, painted composites acquire subtle handling scuffs, fabric fades/wears, floor grip marks develop along routes, glass stays readable. Avoid uniform all-over scratches, grime/trash spawning, age-only mechanical failure, maintenance/cleaning chores or reduced performance purely from cosmetic age.
- Cosmetic age is bounded and saved with stable component identity. Pause and any verified offline elapsed-time rules must not create unbounded sudden aging. Use coarse deterministic counters/variants, not per-frame random scratch generation.
- **Actual damage** must come from authoritative damage/repair systems. Minor marks use conservative material variants; meaningful broken/failed parts may use authored localized **geometry variants**, e.g. broken casing or bent bracket. Real structural detachment changes physical hull identity/collision and must not be faked by visuals alone.
- **Owner-approved clean repair behavior: repairing damage restores normal geometry and removes damage visuals, without leaving permanent patches, welded reinforcements or repair marks; preserve pre-existing cosmetic age.** Physically replacing a part gives it fresh cosmetic age. Earlier patch-dependent appearance suggestions are superseded.
- Keep the first Scout maintained/used, not a wreck. No auto-damage from age. Recoloring does not heal physical condition or erase age.

## Mechanical animation and habitation use

Motion style is **grounded and tactile**, with physically intelligible hinges, slides, pivots and rails; convincing weight and soft acceleration/deceleration, quick response, firm endpoints and subtle materials/stop sounds. A heavier shutter feels heavier than a cup clamp. No need for complex physics-based multi-stage animation or idle movement on static decoration.

- Deploy/fold bunk, segmented privacy screen, seat/harness, table, locker/cabinet, protective shutter and mechanical restraints using real collision clearance and valid intermediate states.
- Sit/stand and lie/get up use short noncinematic player/camera transitions and retain free look; ordinary use does not fast-forward time. Bunk/seat restraints are visible for zero-G; no unsupported special survival bonus.
- Food handling and washing are lightweight believable animations that use existing game authority; no new hunger/health/hygiene mechanics implied.
- Manual physical mechanisms can be used during blackout; powered mechanisms respect Core power and actual component damage. Blocked motion safely stops or refuses. Interrupted/reversed animation and save/load preserve valid states, no clipping/trapped character or invalid object relocation.
- Use existing contextual prompt/control language and avoid interfering with existing one-object carry, cockpit/E, movement, Q-wheel and pause behavior; exact key binding is an implementation decision, not a new conflicting control assignment.

## Continuous sound, tactile cues, acoustic separation

Approved **restrained continuous ambience with meaningful silence**, powered/operating only where supported by actual mechanical state. Cabin differs from engineering and cargo acoustically; no random chatter/music, atmospheric EVA audio, fictional ship air-pressure behavior or always-on exaggerated drones.

- Source types: quiet real-powered ventilation and machinery, Core/Fabricator, appropriate engine structure sounds, physical door/ramp, footsteps differing by floor material, bunk/table/seat, privacy screen, observation shutter, locker, clamps and cargo restraints.
- Weight and event synchronization matter: light latch clicks only when actually fastened, heavy shutter and cargo bulkhead have more substantial travel/end-stop sounds; sounds track interruption/reversal, not merely key press.
- **Lightweight physical sound separation:** distance/direction always apply; a **closed full-height internal cargo door significantly attenuates** sound from the other room, reopening restores it; closed **sleeping privacy screen gives small softening** only. Do not require ray-based acoustics, detailed reflections or a room-reverb solver. New modular rooms may add simple physical boundaries.
- Core blackout shuts down powered audible machinery/ventilation, but manually operated hinges/latches remain capable of physical noise; no fake backup ventilation or lingering phantom hum. Outside vacuum/EVA stays governed by real implemented environmental model.
- Keep listener/source positions, master mute, ship volume, accessibility visual equivalents and lifecycle controls. Deliver a continuous playback/mixing boundary; present half-second OS/WAV process chunks are only a foundation, not proof of gap-free ambience. Listen to actual original playback and measure response/memory/frame costs.

## Technical and implementation boundary

The **current** Scout remains a Blender-authored combined GLB loaded as a static shell plus glass path with a few tagged dynamic features. The checked-in export report measures **6,493/6,500 triangles; 123/124 primitives; 13/13 materials; 549,732/557,056 GLB bytes**. New furnishings, window modifications, installed objects, damage variants and modular construction require an intentionally migrated asset architecture and *measured* CPU/GPU/memory/draw budgets, not silently pushing more geometry into this nearly full GLB.

- Keep source Blender files editable and the hull/cockpit/wide monitor/door spatial contracts tested. Independently rendered reusable fixtures, physical item identities, room material overrides, light fixtures and damage variants should be instanced/batched where reasonable.
- Current shader's warm fill, cold blackout visibility floor and emissive machine feedback are real foundations, but **no physically accurate sun-through-window lighting, shadows, ordinary ship PBR metallic/roughness/texture sampling or full indirect light** is currently established. Authoring Cycles reference images doesn't certify production wgpu.
- Current synthesized audio chunks may have response latency or gaps. A continuous backend, physical source cues and partition-aware attenuation require implementation and human listening.
- Maintain deterministic authoritative interaction/physics/power states, safe save-schema evolution, true one-item carrying and mass accounting, no new unapproved damage or water/pressure/survival systems, and unchanged future construction/refund/Core-upgrade contracts.
- Evaluate genuine playable performance in furnished scenes and with many loose objects; cloud software-Vulkan capture is helpful but is not actual platform/Metal frame-budget signoff.

## Reference scene and acceptance contract

Extend the existing reproducible cockpit, Core, Fabricator and aft-door cameras in scenarios/visual/scout-cameras.json with **habitation entrance**, **lying in bunk**, **seated at window-side table**, **galley/hygiene**, **cargo threshold** and **customized fixture close-up**. Captures are actual production renderer images; do not auto-promote human art approvals from image comparisons.

For representative fixed viewpoint and scene state pairs at **1280x800** and **1920x1080**, inspect:
1. Powered sunlit planet, window shutter open vs closed; powered dark-space, Work/Relax/Night and individual override.
2. Blackout on sunlit ground vs dark-space blackout, physical instrument blanking, restored lighting preference, actual manual shutter and privacy-screen movement.
3. Manufacturing consistency and readability: graphite frames, ivory/slate material distinctions, copper/terracotta accents, same materials on starter and player-constructed fittings; readable physical UI and safe collision/carry paths.
4. Fresh vs moderately aged, physically damaged vs **cleanly repaired**, physically replaced, room recolored with wear retained, independent adjacent-room finishes.
5. Fixed and moving furniture transforms, full obstruction, sit/lie/look, secured vs loose cargo, zero-G, saves/reloads and passage from rear ramp to cockpit.
6. Original sound recordings for cabin/engineering/cargo, closed/open cargo door, sleep screen, lightweight mount, heavy restraint and powered/blackout quiet; mute, no stale audio cues.
7. Whole-scene CPU/GPU/frame-tail and memory/draw accounting for base Scout, furnished/expanded cabin and realistic multiple loose-item stress; record actual backend/hardware context. Human image/audio acceptance and hardware-performance qualification are independent of cloud functionality checks.

### Reference playable loop

Enter through aft ramp carrying one world object -> secure in cargo -> close bulkhead (hear separation) -> visit Core/Fabricator/Archive -> enter habitation -> deploy table/seat -> view true exterior through window -> change room/fixture light -> place and explicitly secure a found sample on a rail shelf -> operate shutter/privacy screen/bunk -> test real blackout and restoration -> verify age/damage/clean repair variants -> save/reload and recheck IDs, attachment relationships, appearance, fixture states and physical access.

## Work tracking and non-duplication

This document records owner decisions; implementation follows **separate development issues**, not this documentation PR.

**Existing foundation**: [#538](https://github.com/mr-exception/salimon/issues/538) starter hull and cabin layout; [#539](https://github.com/mr-exception/salimon/issues/539) cargo; [#540](https://github.com/mr-exception/salimon/issues/540) base habitation; [#541](https://github.com/mr-exception/salimon/issues/541) 3D grid foundation (closed); [#542-#545](https://github.com/mr-exception/salimon/issues/542) construction/rooms; [#549](https://github.com/mr-exception/salimon/issues/549) real damage/salvage; [#550](https://github.com/mr-exception/salimon/issues/550) construction persistence. Existing presentation/verification: [#551](https://github.com/mr-exception/salimon/issues/551), [#561](https://github.com/mr-exception/salimon/issues/561), [#563](https://github.com/mr-exception/salimon/issues/563) and [#502](https://github.com/mr-exception/salimon/issues/502).

**New targeted development** (not duplicates of those owners):

| Issue | Responsibility |
| --- | --- |
| [#575](https://github.com/mr-exception/salimon/issues/575) | Independent reusable runtime furnishings/material/transform/damage-variant architecture and measured budgets |
| [#576](https://github.com/mr-exception/salimon/issues/576) | Free object placement, explicit securing/releasing and physical rail mounts |
| [#577](https://github.com/mr-exception/salimon/issues/577) | Tactile habitation furniture animations, sitting/lying and safe transitions |
| [#579](https://github.com/mr-exception/salimon/issues/579) | Room finish groups and individual furniture cosmetic customization |
| [#580](https://github.com/mr-exception/salimon/issues/580) | Fixture/preset lighting and true shutter-aware natural light |
| [#581](https://github.com/mr-exception/salimon/issues/581) | Usage-driven localized age, real damage visuals and clean repair appearance |
| [#582](https://github.com/mr-exception/salimon/issues/582) | Seamless continuous restrained Scout audio backend |
| [#583](https://github.com/mr-exception/salimon/issues/583) | Tactile sound events and lightweight cargo-door/privacy-screen acoustic boundaries |

Human signoff remains required when a reviewer is asked to approve actual art or sound: [#561](https://github.com/mr-exception/salimon/issues/561). Performance thresholds are governed by [#563](https://github.com/mr-exception/salimon/issues/563), not implied by this design approval. The palette values are proposals pending actual render approval.


## Expanded cabin mechanisms (#577)

The optional schema-12 cabin source, manual reversible fixtures, supported postures, visible restraints and native controls are implemented in [the maintained cabin contract](cabin-mechanisms.md). Historical schema-10 habitation remains the migration baseline; lighting preferences (#580), sound (#583), human comfort (#561) and hardware budgets (#563) retain their separate owners.


## Persistent cosmetic groups (#579)

Five room groups and exact furniture overrides now use the [bounded finish contract](room-finishes.md). Schema 13 preserves histories independently of divider topology; physical material replacement, future constructed room geometry (#545) and actual damage (#549) retain their owners. Shared instance tint factors are now bounded to [0,4] and cosmetic RGB is clamped before lighting; warning/functional surfaces retain authored values.


## Room and fixture lighting (#580)

The [maintained lighting contract](room-lighting.md) describes implemented presets, independent overrides, effective bus/condition gates, shutter/Sun contribution and schema 16. Actual future damage/room creation remain #549/#545; human approval #561 and hardware budgets #563.
