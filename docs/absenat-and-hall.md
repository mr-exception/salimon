# Absenat, cosmological threshold and Hall of Passage

Status: **approved endgame narrative; an internal backend-independent spatial prototype is implemented in #472. Completion, accounts, inscriptions and returning access remain future work**. See [prototype scope and verification](hall-prototype.md). Documentation issues #460–461. Some implementation details (distance discontinuities, moderation specifics, final message length) remain future contracts rather than invented canon. See [lost journey](narrative-and-lost-journey.md), [experience](experience-direction.md) and [roadmap](experience-implementation-map.md).

## Absenat and chained universes

Absenat is a remote endgame destination in this universe. It contains a gate giving access to the **boundary where the end of a preceding universe meets the beginning of ours**. Cosmological eras form a possible chain: the end of one is the beginning of another. This is speculative science fiction inspired by cyclic/bounce/Big Bang cosmology, **not** a scientific claim that a literal Big Bang happened at one point in our existing space. The Big Bang refers to expansion of space from an early hot dense state; the gate is fictional access to a boundary, not an ordinary galactic coordinate of that event.

Keep uncertain: whether the transition can be traversed in ordinary terms, exact number of universes, source/nature of the Core or entity, purpose of wormhole, whether traveler passed a wormhole, and identity/intent of warning sender. Do not conflate the wormhole (possible shortcut) with the Absenat gate (endgame destination). Reaching/passing the Absenat gate ends the active journey; backend completion is verified before permanent account rewards.

## Hall geometry and artistic meaning

Beyond the gate, an **apparently endless, circular, monumental Hall of Passage**, a physical presentation of the starts of the universe chain. Its **center contains the gate and an intensely bright, blazing, dense/hot primordial Core-like presence**, symbolizing the Big Bang origin. It is a presentation, not literal containment of the historical Big Bang. Geometry and details convey massive scale, awe and silence, with signs arranged in widening **concentric rings** around the center. No visible exterior wall, no ordinary MMO/social-feed layout. The central Core is visually powerful but player-friendly/readable and performant; keep a safe traversable ceremonial floor and camera composition.

Ring 1 contains 40 positions; ring 2 contains 60; ring 3 80; ring 4 100; generally `capacity(r)=40+20*(r-1)`, with stable adequate circumference spacing and increasing radius. Usernames and short final messages appear on physical inscribed signs/monuments, with the same art language and readable proximity. Far rings can use streaming, LOD and distant silhouettes. Architecture appears boundless rather than requiring actual infinite meshes.

## Rank by traveled distance

For all verified completed journeys, rank travelers by **greatest cumulative physical travel distance** achieved on a successfully finished run (not straight-line spawn-to-end displacement or arrival timestamp). Farther-traveled player → closer-to-center ring. Exact accumulation across wormholes/teleports/discontinuous coordinate shifts **must be specified before implementation** to avoid awarding fictional distance; tie breakers and identical user/account policies also need deterministic backend definitions.

Rankings are **dynamic**, so old signs may occupy different rings after new completions. **Each hall visit uses a stable snapshot:** no reshuffling while a visitor walks or reads; entering again can fetch an updated ranking. Ring position is not a permanent entitlement, but username and inscription persist.

## First-arrival ceremony

Each newly completing player sees **one temporary blank sign in the first inner circle**, even if not top-40 by distance. The reserved ceremonial position is a **visitor-local presentation overlay** and does not eject another player's ranked sign from the canonical shared snapshot. The player approaches the blank sign and submits their final message, associated with account/username. Only when completion and submission are durably accepted does the game end/unlock Hall revisit; on reload or failure retry idempotently. After ceremony, canonical ranked placement follows cumulative distance. Avoid a conspicuous leaderboard UI or rank labels dominating inscriptions.

## Permanent access and spoiler protection

After first successful completion, the main menu **gains** `Visit the Hall of Absenat`. This option remains unlocked across future journeys and death. From menu, players can enter the Hall directly, explore/read refreshed ranking and their own latest sign, with **no additional inscription submission on revisit**. Hall revisit is separate from active save/resume and does not consume a new character run.

**Absolute spoiler rule:** Before first verified completion, do not render or expose Hall menu action at all—not even disabled/greyed out, empty slot, tooltip, tutorial, achievement label, loading hint, accessibility label, metadata preview or any other discoverable UI. Developers can exercise explicitly gated internal fixtures, not shipping menus. Test both pre- and post-completion UI.

## Backend and cross-universe memorial

The shared Hall is future work **only after backend/account support**: a global durable collection of inscriptions across the shared chain-of-universes lore, tied to verified completion and player identity; moderation, safety and abuse prevention required. Player's final one-time message and username, verified best completed distance, ranking placement and permanent Hall unlock must survive new games/death. One visible sign per account is the proposed policy; repeated successful journeys should preserve the original inscription while eligible best distance updates ranking. Reconsider only via explicit owner contract, not ad hoc UI behavior.

Backend should provide atomic idempotent completion+inscription/unlock, consistent versioned visit snapshots, deterministic tie-breaking, pagination and scalable ring queries, moderation, account rename/display policy, outage/reconnect handling and privacy rules. Do not pretend offline local data is a global public memorial. Content limits and moderation rules are implementation-level choices pending review.

## Lore tone

The hall is a quiet convergence of those who have completed the near-impossible journey; inscriptions are traces symbolically carried across universe beginnings. Avoid forced NPC crowds, chat feeds, loud music, spoiler cinematics before gate passage or detailed exposition stating who constructed the Hall. The central object must be distinct from ordinary ship Cores in scale while preserving thematic resemblance.
