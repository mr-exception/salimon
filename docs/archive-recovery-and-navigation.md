# Scout Archive, data recovery and discovered navigation

> **Future planetary field observations (2026-10-24):** [Planetary environmental art direction](planetary-environment-art-direction.md) adds proposed lightweight suit location/bearing marks with assisted, uncertain positioning, and review/editing through a physical Scout planetary chart. These extend **player-known observation** rather than canonical catalog, star-chart discovery or the existing automatic five-second scanner. No perfect planet GPS, automatic landmark POI reveal, new scan mode or autopilot is approved.


Status: **Archive reader #467 and local Recovery Module #468 implemented; destination leads/charting remain future #469–470**. Documentation issue #459. Related: [lost journey](narrative-and-lost-journey.md), [approved universe](universe-expansion-approved-design.md), [Absenat](absenat-and-hall.md).

## Physical Ship Archive

The Scout contains an inspectable, diegetic archive/communications terminal. It shows ordered local ship events, voyage entries, sender/source and timestamp **when known**, routine communications and recovered data. Initial intact records include Earth orbital departure, Mars transit, unrelated daily notes/maintenance and leaving Solar System territory. Later records are damaged; unreadability may be due to corruption, cryptographic encoding or missing segments—**do not call all damage “encryption.”** Recovered bits must retain provenance, confidence, original record identity and permanent read access. No player-known Earth coordinates merely from historic mentions.

Explicit states: intact/readable; damaged with recoverable metadata; actively recovering; partially reconstructed; recovered with uncertainties; irrecoverable; unverified content. Users can review old recovered logs later. Differentiate untrusted incoming messages from verified navigation/telemetry. UI must not falsely prove wormhole transit.

## Recovery Module and energy motivation

An onboard module is installed/activated or restored through physical Fabricator/ship interactions (exact authored mounting/recipe to be decided during implementation). It performs reconstruction, integrity repair and decryption where appropriate, drawing **large, configurable Core energy**. This creates the player's first voluntary motivation to mine, carry resources back aboard, power Core and engage Fabricator without an arbitrary fetch quest. Do not hardcode unapproved duration, recipe mass or energy. Protect emergency propulsion/landing reserves from unsatisfiable softlock and show estimated progress/power limits. Specify behavior for insufficient ingredients, interrupted Core draw, queue/priority, app closed/offline restoration, failure, save/load and accessibility. Reuse existing Core conversion/energy and crafting authority; avoid a second fictional power economy.

First recovered message supplies a distress/accident planet location; second supplies an origin for a distant hostile/trap warning. Investigation can uncover **clues suggesting** a wormhole shortcut near Absenat, without confirming its truth. Further messages not yet authored. Recovered records should motivate travel and leave both return-Earth and Absenat goals open.

## Knowledge model and route charting

Separate canonical universe catalog (immutable scientific identity/ephemeris) from **player-known celestial chart** and confirmed navigation targets. The traveler knows Earth as birthplace/home, not its coordinates or Solar System route. Absenat also lacks known coordinates. Never expose hidden canonical coordinates or marker just because an object is present in offline pack.

Suggested states: unknown → observed contact → identified/inferred → source-supplied lead → independently verified/chartable. An unverified coordinate can point the player toward exploration without being misrepresented as fully known or safe. Track observation source, timestamp, catalog identity, frame/epoch, uncertainty and saved route history. Use existing single five-second scanner pulse model, contact expiry/historical discovery, future #428–431; scanner contact freshness and durable navigational discoveries should have deliberately distinct lifetimes. Validate navigation transformations in the new real-scale coordinate and moving-body model; no free galaxy-wide destination search.

Finding Earth is a valid independent long-term investigation through observation, recovered history, star matching and possible external evidence; no requirement to follow the wormhole messages. Player can freely choose either direction or neither.

## Persistence and boundaries

Versioned records, module state, progress and discovered route identities must round-trip in canonical save under #363–366, #406, #431; offline progression rules must explicitly respect #432–434. Never mix UI rendering ownership with simulation authority. No required backend during opening prototype. No Hall UI or spoiler references pre-completion.

## Validation

Test arrival with no Earth route, inspecting ordinary and damaged messages, interrupted recovery, full mine-carry-Core-recover loop, recovering two distinct location leads, navigation uncertainty, save/load/close/reopen, existing scanner pulse/contact expiry, and alternate choice to chart Earth directly. Keep historical event claims and recovered evidence separate.

## Jump Drive integration with discovery (future design only)

The [Spatial Jump Drive](jump-drive-and-interstellar-travel.md) is a **physical Core-powered owner-operated displacement module**, not the narrative's alleged wormhole. The player may set **six light-distance fields** (years/months/days/hours/minutes/seconds) to request a jump along the ship's current forward direction; this neither supplies free galactic search nor reveals unknown Earth/Solar System/Absenat coordinates. Canonical hidden catalog and moving-body data may be checked for **destination collision safety without granting player knowledge**; show navigation/uncertainty warnings through existing instruments only when supported by actual charted evidence. The jump leaves inertial velocity unchanged; approach/orbit/braking remain player responsibilities.

The quantum-themed transition is **loading**, not proof of travel through the unresolved wormhole or another chronology; every player remains on the same universal 1:1 real-time clock. The engine must be explicitly **off**, not just zero throttle. First-module N is **20%** of the remaining difference to c per light-second priced as equivalent Core impulse (see full formula in the canonical future design); no jump code is delivered by these docs.


## Implemented local reader (#467)

The separate Scout wall terminal now exposes five intact ordinary voyage/routine
records and two damaged packets with unknown contents. Source and unknown time
are explicit; a name or historical departure never grants a destination. Durable
versioned identities/read selection round-trip through schema 6. Recovery states and Core-funded work are delivered by #468. Destination leads
and player charting remain #469–470. Native appearance/input signoff remains separate. See the
[implementation boundary](../client/runtime/architecture.md#scout-archive-467).

## Local Recovery implementation (#468)

[Canonical module contract](data-recovery-module.md) records configurable recipe/
energy defaults, one-component installation into the Archive, emergency reserve,
low-priority funded work, R controls and schema-7 migration. Recovered local
fragments stay unverified; full location-bearing story payloads remain #470.
Native physical/visual/budget and provisional tuning review remains #515.
