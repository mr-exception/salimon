# Planetary environmental art direction and exploration contract

Status: **owner-approved future art/gameplay direction (2026-10-24); design-only, not a claim of implemented weather, ecology, landmarks or planetary charts**. This consolidates the fifteen-round planetary environment discussion. It is the environmental presentation/interaction source of truth alongside, not a replacement for, [canonical planet generation](canonical-planet-generation.md), [universe expansion](universe-expansion-approved-design.md), [planetary/world physics](world-and-realism.md), [visual style](visual-style.md), [experience direction](experience-direction.md), [Scout design](scout-interior-art-direction.md), [navigation and Archive](archive-recovery-and-navigation.md), and [resource contracts](../client/world/resource-contracts.md). When an approved physical or save contract is more specific, that contract controls; new capabilities require their own implementation and verification.

## Intent and scope

Planet surfaces must have **plausible, persistent identity**: physical and astronomical conditions shape geology; geology and atmosphere shape climates; weather varies coherently in shared time; and rare biospheres can produce distinguishable ecological regions. A sterile planet may be almost empty but must not look like an unfinished demonstration. Seek grounded cinematic stylized low-poly silhouettes, restrained material treatment, informative environmental sound, interesting but rare exploration landmarks, and a strong contrast with the warm familiar Scout.

This is an art direction and future-gameplay specification, **not authorization for full fluid simulation, autonomous ecology, fauna, survival toxicity, terrain destruction, caves, surface bases, a new scanner mode, an autopilot or an alternate renderer**. Design proposals below require engineering scoping and native evidence before considered shipped.

## Canonical world ownership and generation

One published, versioned, hashed planet definition drives physical and gameplay truth. Its named independent, versioned domains include terrain, geology, resources, visuals, future climate/ecology and discoveries; declare explicit dependencies where geological substrates or biomes influence another domain. Generation/seed selections for fictional catalog records occur reproducibly during pack production; runtime samples the published definition on demand, not player-specific randomized planets. A renderer must never invent terrain collision, landing ground, a collidable giant organism or a gameplay deposit different from shared world authority.

Separate:
- **Immutable baseline**: planet science/identity and revision, landforms, ecological potential and biosphere decision, ecological regions, organism families and fixed meaningful individuals, resource/landmark/site identities and initial mass.
- **Time-evolving environmental state**: shared-clock astronomy, regional weather and coherent seasonal appearance. Evaluate weather for planet, location and shared universe time without running dense per-second global simulation. Gameplay-affecting weather must agree across simulation hosts; local graphics can approximate its appearance.
- **Persistent mutable deltas**: depletion/harvesting, detached objects, authoritative destruction if separately introduced, player observations, structural/site changes and any future renewable-resource events. Don't regenerate a consumed pod or move a site upon streaming.
- **Presentation**: mesh LOD, instancing, texture detail, audio voice budgets and visual micro-cover, which may vary by device without affecting gameplay identities.

Preserve existing version-1 resource/deposit identities unless migrating deliberately. Fictional-world changes after publication require explicit versioned migrations that reconcile geometry, saved positions, players, ships, site IDs, biological sources and depletion. For published real observed solid bodies, preserve established authoritative physical geography and coordinates under the real-body non-displacement policy. Installed catalog data is not itself player-discovered knowledge.

## Geological history, regions and surfaces

**Procedural scientifically grounded identities** are generated from plausible planetary properties rather than independent aesthetic palettes. Each planet can possess several coherent geological provinces (plains, impact basins, highlands, icy expanses, volcanic regions, fractured terrains, deposits) where justified; some bodies are nearly uniform. A lightweight authored-and-procedural geological *history*—impacts, volcanism, tectonic deformation where physically appropriate, weathering, erosion and deposition—explains major forms, layered materials and regional distribution. This is not a full geophysical simulation.

Large forms, regional boundaries and resource-relevant substrate come from canonical definitions and the one authoritative terrain evaluator. Include natural transitional regions, stable object/landing clearances and consistent visual/physical distances. Never add an ornamental ridge that a player or ship passes through. Caves, arches requiring overhang collision, tunnels and undercuts remain deferred where the radial heightfield and collision representation cannot support them.

**Selectively active geology** is possible where physically justified: persistent vents, plumes, eruptions, cryovolcanic activity or tremors. Active sites have stable identities and positions; their intensity and effects can change with shared time. Do not automatically deform published canonical terrain or invent atmospheric effects on airless worlds. No implied environmental health or ship-damage rules without separate gameplay approval.

Surface presentation follows substrate and climate: broad silhouette, controlled facet sizes, geological layers, mineral palettes, limited detailed wear or deposits, dust, frost and plausible regional changes. Favor coherent large-scale fields over noisy ground textures and arbitrary saturated biome boundaries.

## Atmosphere, climate, lighting and weather

Distinguish **climate**, which determines geographically and seasonally plausible conditions, from **weather**, which changes at a time and place. Inputs include atmospheric pressure/composition, available condensables, stellar spectrum and flux, rotation and inclination, latitude, elevation, surface volatiles and geological context. No atmosphere means no ordinary sky haze, airborne dust, clouds, precipitation, wind, or wind-driven vegetation.

Use a **lightweight coherent regional weather model**, not computational fluid dynamics. Weather fronts, cloud masses, dust storms and precipitation must align between orbital view, descent and the ground; conditions must not flip abruptly when tiles stream or camera LOD changes. Cloud forms, color and opacity depend on chemistry and actual conditions; airborne particulate dust is not the same as condensed cloud. Seasonal/regional changes should make causal sense. If weather can affect gameplay, share authoritative values and time across offline resume and future multiplayer.

Astronomically informed illumination follows the local star(s), celestial geometry, atmosphere and occlusion. Sky scattering, sunsets, day/night exposures, eclipse shadows, cloud attenuation and storm light should be plausible and smooth where supported. Avoid universal blue skies, omnipresent fog, glowing night terrain or fixed artistic daylight contrary to the local system. Shader fidelity is explicitly subject to production wgpu feasibility; scalable approximations are acceptable if grounded.

**Surface liquid/hydrology scope is unresolved.** No approval is implied for oceans, standing liquid bodies, coastlines with gameplay water collision, swimming, rivers, liquid flow or a fluid simulation. Existing climate/erosion history and atmospheric weather design may reference volatiles and precipitation, but fluid-bearing worlds need a separate physical/rendering/gameplay decision before liquids are implemented.

## Rare biospheres and ecological regions

**Habitable does not mean inhabited.** On fictional worlds, environmental suitability sets potential, but a separate *canonical fixed biosphere decision* determines actual presence; no runtime per-visit coin flip. Real-world astronomical observations must not be represented as proven alien life. Preserve enormous lifeless and empty regions; no guaranteed planet-wide vegetation even when microscopic life exists.

A biosphere may contain hidden/microbial, sparse macro-organism, developed surface, liquid-associated or subsurface life, possibly with complex communities. Visible forests are not mandatory. Surface ecology follows geology, elevation, exposure, environmental chemistry, available light/liquids, latitude and climate, with gradual transitions unless the environment has a real discontinuity. Ecological baseline and biological family identity are deterministic and stable; weather and season change appearance rather than rewriting individual identities.

Generate a coherent planetary **biological design grammar**, then related species families, local populations and close-range visual instances. Structural families include mats/crusts, upright columns/branching forms, flexible sheets/filaments, unusual colonies and rare enormous specimens. Body plans reflect environment and support constraints; no universal Earth trees/green leaves. Planet identity creates recognizable related colors, structure and shapes. Species and individuals add bounded visual variation, not arbitrary palette randomization.

**Biological appearance: scientifically motivated diversity** of pigment and structural coloration, matte, waxy, mineralized, reflective, iridescent and modestly translucent materials. Pigment choices are informed by chemistry, stellar illumination and biological history without simplistic star-color rules. Bioluminescence is occasional, localized and anatomically/biologically motivated, not an all-purpose blue neon overlay. Emissive presentation may be visible without expensive world lighting; bloom-off readability and native budgets matter.

**Lightweight physical ecology**: cosmetic ground cover needn't be individually collidable; large fixed organisms and interactive parts have stable identities and meaningful simplified collision. Physically important biology participates in safe landing, walking and object placement, including authored opening-site clearances. Large anchored specimens can act as real landmarks. Wind/fluid motion uses material-specific flexibility and local environmental conditions; movement that physically blocks or pushes actors needs synchronized collision logic, while small appendage animation can remain cosmetic. No implicit reproduction, population simulation, continuous growth, behavioral fauna or automatic regrowth.

## Biological resource interaction and persistence

Use existing `organic-biomass` and `medicinal-compounds` material identities initially; the presence of these catalog keys **never proves living organisms, edibility or medical safety**. Species may yield neither, one or both through explicitly defined separate material-bearing parts. Do not assume every organism is harvestable.

Two interaction paths:
1. **Direct detachment of designated reachable parts** (pods, growths and safe-to-detach appendages): with an empty hand, F grips and separates the particular part and leaves the player holding **that same physical object**. An equipped mining tool is stowed as appropriate to ordinary carrying rules. While holding another item F retains drop behavior, not a second pickup. Direct detachment is unavailable through obstacles/outside believable reach, or for excessive/structurally integral growths. Use short physically grounded reach/grip/resistance/separation/hold animation; interruption before the committed separation leaves the source unchanged.
2. **Existing mining tool** for embedded, non-detachable biomass or mineralized biological deposits: use existing held-left-mouse spherical extraction, configured rate/cap, aim/interrupt handling, physical fragments and resource accounting, not a second competing extraction system.

A discrete part has stable parent/part identity, attach point, material and original/remaining mass. Its detachment is atomic: source state changes and one uniquely identified held physical object is produced, with no invisible inventory transfer or duplicate mass. Hold/drop obeys one physical object at a time; released part retains identity, mass, pose, and suitable approximate physical geometry. Persist per-part missing/depleted visual appearance through stream/reload/save and future shared state; no unapproved regrowth. Mining of embedded materials likewise preserves remaining mass and spherical fragment conservation. Presentation/LOD changes may not visually restore a removed prominent growth or alter harvestable mass.

Species-specific feedback is restrained: elastic tension/tear, rigid fracture, hard socket separation, exposed tissues or depleted surfaces, localized particles and material-dependent sound with valid transmission paths. No gory universal effect or gigantic light flash.

## Natural landmarks, exploration and observations

**Procedural landmarks with rare authored signatures.** Typical geology naturally creates ordinary landscape texture. Uncommon notable features and extremely rare signature locations (geological structures, giant organisms, mixed geothermal/ecological formations) use deterministic geology/ecology/site IDs and controlled authored exceptions where valuable. There is no minimum landmark quota per planet, no regular player-centered placement, no required treasure or reward, and no impossible underground/overhanging geometry added beyond the supported physics.

A player may **sight**, **deliberately mark**, **inspect** and **record** an observation. Sighting does not auto-populate a POI map; inspection need not mean definitive scientific identification. Marked but unverified information retains its provenance and uncertainty. Arbitrary player-interest locations (a quiet valley or good parking/landing site) must be markable without being predefined canonical landmarks. Signature sites retain stable canonical IDs; player-created marks have their own stable observation IDs.

**Field marks + physical Scout chart (chosen B).** The suit has a compact affordance to mark current estimated position or distant viewing direction/area with optional note and timestamp. No full handheld magical world map, equipment-slot replacement, extra manual scanner or forced HUD quest marker. The Scout chart provides detailed review, naming, editing, observation history, estimated uncertainty and selection of a known location for *manual* navigation. It does not grant an autopilot.

**Suit location accuracy: assisted with uncertainty (chosen B).** Record position estimates derived from ship-relative/inertial movement and genuine available celestial/environmental fixes, not exact authoritative engine coordinates masquerading as sensor readings or universal GPS. Fixes may improve estimates; dead reckoning may drift; instrument failures and missing fixes degrade precision honestly. A sightline to an unknown distant feature records direction and approximate region, not an invented exact planet position. Display quality such as well constrained / estimated / poorly constrained / unavailable with non-color-only affordances. Precise numerical models, input shortcut and equipment power behavior require navigation implementation design, not art-bible guesses.

A stationary surface coordinate and observation anchor are planet-body-fixed, with reference frame/version, recorded epoch where relevant, source provenance, confidence and stable identity. Keep the immutable universe pack separate from the player's discovered knowledge and chart; no Earth/Solar System route, Absenat secret or landmark automatically exposed from offline package data. The **existing single omnidirectional five-second scanner pulse** remains unchanged: no automatic surface POI scan, active/passive mode or manual pulse. Live scanner contact expiry is distinct from durable observations/landmark history.

Save, reopen, region eviction, future shared-state synchronization and approved world revisions preserve observation and site identity. Player deletion/editing of a bookmark does not delete the canonical landmark; future multiplayer does not automatically merge personal charts or grant hidden knowledge.

## Acoustic identity and restrained ambient presentation

Sound must follow actual transmission paths and condition, not a generic soundtrack per biome. Airless exterior: no airborne wind, precipitation, wildlife or unexplained room-tone; suit machinery and structure-borne/contact vibrations may be audible through valid coupling. In atmosphere: locally supported wind, precipitation, dust, moving material and biology, spatially attenuated and obstruction-aware where feasible. Geological events and biological parts sound only from plausible sources, distance and media. Flexible tissues rustle; rigid plates may knock; submerged phenomena use their medium if explicitly implemented. Do not fake fauna calls without fauna.

Tie ambient response to the same weather/environment state driving particles, illumination and vegetation animation. Weather visible on the horizon is not automatically audible at the player's location. Preserve meaningful quiet and silence; no always-on ambient music, particle loop or random ominous chatter. Respect existing Scout interior door/privacy acoustic decisions and suit/exterior transitions; use restrained material-specific harvesting feedback. Support independent audio mix/accessibility and visual alternatives for important warnings.

## Reference scenes, verification and exclusions

Provide repeatable source revision, canonical planet ID, location/frame, simulation timestamp, camera route, weather condition, player/save state and graphics/audio settings for all comparisons:
1. **Vesper airless day/night:** correct physically dark vacuum sky, material/mineral identity, long-range landmark continuity, no wind or airborne atmospheric sound/effects.
2. **Geology from orbit to ground:** coherent geological provinces, strata, terrain/physics contact, safe landing and landmark preservation over LOD/streaming.
3. **Atmospheric day/night + storm:** cloud/illumination/weather continuity from orbit to landing, seasonal/regional behavior and matching sound.
4. **Rare living world:** related low-poly biological families, credible distribution/collision, pigments, transparency/iridescence and plausible restrained luminescence with bloom disabled.
5. **Harvest + reload:** F grip and physical handoff, carrying restriction, interruption, atomic mass conservation, partially mined biological resource and unchanged depletion after resume and region eviction.
6. **Landmark observation:** unknown sighting -> approximate field mark -> Scout chart -> close inspection -> persistent verified history, with uncertainty, malformed data and safe world-definition migration.
7. **Acoustics:** airless quiet, valid physical contact, atmospheric spatial sound, local geological activity, harsh weather and Scout door transition; accessible equivalent for meaningful cues.
8. **Ordinary empty world:** interesting credible terrain without forced organisms, landmarks, atmosphere or constant sound.

Automate reproducibility, numerical identities, resource conservation, reload/golden vectors, weather synchronization, culling and crash behavior in portable tests/production renderer where available. Native testing must include actual perceptual scene/audio review on Apple M1/Metal reference hardware; collect real median/p95/max frame time, CPU/GPU/memory/draw costs and missed 60 FPS target intervals. Do not invent triangle/material/draw thresholds, declare unimplemented shaders supported, or treat cloud software-Vulkan/Blender captures as M1 signoff. Improve through adaptive clouds, instancing, terrain/organism LOD, caching and density without changing gameplay truth.

**Outside this approval:** simulated oceans/rivers/liquid navigation; hydrology and fluid-body physics; autonomous fauna or animal calls; toxic-air/suit-oxygen effects, full food/medicinal chemistry; continuous ecosystem regeneration, full structural/tree destruction, landscape deformation, player-built planetary bases, caves/overhangs where unsupported, new radar mode, global omniscient map, mandatory missions. Each requires its own approved design and dependencies.

## Delivery dependencies and documentation ownership

Start with canonical terrain/geology/region contracts, then physically supported surface materials/atmosphere and independently coherent climate/weather. Build biosphere selection/organism grammars on those regions; integrate collision and harvesting through existing resource/carrying contracts. Add landmark site placements and visual observation on canonical positions; suit field marks/Scout planetary chart extend—not replace—existing player-known chart and scanner history. Finish with combined cloud tests and **separate** native visual, acoustic and performance signoff.

Check existing #424 (terrain LOD), #425 (landing/mining), #419 (atmospheric drag), #423 (astronomical sky), #428–431 (scanner/history), #469 (player-known chart), #406 (universe saves), #561 (human presentation review), and #563 (hardware budgets) for integration; these are not claims that this new environmental scope is implemented. Implementation PRs must update the source-of-truth and any superseded docs, plus test evidence, not just ship visuals.


## Implemented geology boundary (#612)

[Terrain-bound geology](planetary-geology.md) now supplies versioned deterministic
province, history, substrate and layer observations without changing published
terrain or resources. This additive implementation does not implement weather,
active sites, biospheres or new terrain migrations. Surface presentation is #613.


## Implemented surface presentation (#613)

[Geological materials](geological-materials.md) now map versioned geology into
neutral exposure/weathering response and filtered layers in production terrain.
Vesper remains unchanged. This does not implement climate, active sites, biology
or physical terrain migrations.


## Implemented regional climate boundary (#615)

[Regional climate](planetary-climate.md) supplies versioned geology-bound inputs
and deterministic shared-TT weather metadata. Atmospheric rendering remains #616;
active sites, biospheres and environmental audio retain their own scopes.


## Implemented active source boundary (#617)

[Active geological sources](active-geology.md) expose immutable suitable sites
and bounded shared-TT plume/thermal/acoustic metadata without altering terrain,
resources or hazards. Existing bodies gain no activity automatically; source
presentation consumers and environmental sound retain their own owners.


## Implemented biosphere boundary (#618)

[Rare biosphere publications](planetary-ecology.md) separate fixed existence from
suitability, with continuous geology/climate-bound ecological regions, independent
family/patch identity and strict catalog/save references. Observed bodies gain no
confirmed life and Vesper stays sterile. Organism meshes/physical instances and
harvesting remain #619–#621; this metadata does not introduce them or hydrology.


## Optical atmosphere implementation (#614)

See [planetary optical atmosphere](planetary-atmosphere.md) for source/test ownership,
physical inputs, bounded depth-aware scattering and exact fixture/native limits.
Clouds/weather presentation remains #616; human/hardware qualification is separate.


## Coherent weather presentation (#616)

[Implemented weather visuals](planetary-weather-visuals.md) route authoritative
shared-time samples through runtime lighting and fixed production renderer
fields. Acoustic integration remains #627 and hardware/human qualification separate.


## Organism family presentation (#619)

[Procedural organism families](planetary-organisms.md) map the world grammar,
stable cosmetic surface tiles, runtime adapter and bounded renderer/test owners.
Physical colliders/landing safety are #620; biological material response is #622.


## Physical organisms and clearance (#620)

[Physical organism placement](physical-organisms.md) owns stable source/part
bindings, root/exclusion rules, common physics sweeps and live character/ship
clearance. Vesper remains sterile; mutable harvesting/removal remains #621.
