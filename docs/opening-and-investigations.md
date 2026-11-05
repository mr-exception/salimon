# Opening and recovered investigations (#470 / #471)

> **Corpus migration status (#791): Historical native implementation.** The current [React overhead 2D world](current-client-status.md) uses a separate **cockpit-only 3D radar**. Any older Rust, wgpu, 3D assets, first-person view or issue-delivery statements below concern the [archived native tree](https://github.com/mr-exception/salimon/tree/8f112bb6e6be9f813174efb47d706a8601930d04) rather than shipped browser features. Keep historical mechanics and balance evidence as input; reimplement in the browser with integer-gram inventory and its own versioned saves. The approved 2D hazard health maximum is 200 HP, not a migrated legacy health implementation. See the [migration register](2d-migration-register.md) for readiness and ownership.


A fresh canonical journey wakes the traveler outside the landed Scout on Vesper.
For the first 45 active seconds, the existing skippable survival tutorial adds a
brief reminder: Earth is home, the Scout is familiar, and the voyage/landing are
missing. T dismisses the whole tutorial. This text grants no coordinates, changes
no controls or resources, and does not restart on reload. Survival, gathering,
crafting, rest and independent charting remain voluntary; standing idle can end
the journey through the existing survival rules.

## Packets, locations and routes

The existing Core-funded Recovery Module retains its recipe, costs, reserve,
interruptions and irrecoverable gaps. Only **complete** reconstruction supplies a
location-bearing payload. In canonical play completion publishes one immutable
lead per packet; reopening the Archive also admits completed legacy packets.
Partial reconstruction never supplies a location. The first packet claims a
distress/accident at a fractured relay; the second claims a trap warning from a
separate remote transmitter. Senders, times, motives and truth remain unknown.
No message proves that the traveler visited either site or entered a wormhole.

`session/narrative.rs` owns two version-1 `salimon-site` identities, fixed tangent
anchors on Vesper and their minimally authored ruined equipment/clue text. The
near relay is 65 m from the landing-chart origin; the warning origin is about
14 km away. Both use the production canonical ground query for exact source and
render placement. They exist before recovery and may be found independently;
there is no quest prerequisite, NPC, functioning passage or Absenat coordinate.
These authored fictional sites have no relationship to real Gaia astrometry.

P opens the cockpit chart; M selects an entry; Enter explicitly stores a route.
A transmitted-site route retains the original source identity and coordinates,
shows **UNVERIFIED ROUTE**, and never becomes an observed scanner encounter or an
autopilot command. The chart supplies same-frame sample distance/azimuth/elevation.
Frame mismatches and future timestamps reject. Existing observed routes retain
their original rules. A packet may be ignored without affecting Earth charting.

Approach a relay on foot, aim within 3 m and press E to inspect. Ground occlusion,
surface location, pause and terminal gates apply. E closes the reader; its modal
input clears held controls. Inspection records the physical casing/plate or local
buffer separately from the inscription's unverified claims. The first inscription
suggests a passage *may* emerge near Absenat. The remote buffer repeats a warning
without proving its sender. Revisit freely; inspection text remains with its
packet in the Archive even when recovery is incomplete. There are no follow-on
damaged records, encounters or new travel abilities.

## Save and verification ownership

Schema 31 adds versioned first-inspection timestamps and omits the transient
reader. Schemas 1–30 migrate to no inspections; current missing/unknown source,
future inspection times and packet leads inconsistent with paid recovery reject
before publishing a restored session. Existing chart entries, exact source
uncertainty and voluntary route snapshots survive reload. Inspection changes enter
the normal discrete checkpoint fingerprint. Fresh journeys reset all evidence.

Runtime [architecture](https://github.com/mr-exception/salimon/blob/8f112bb6e6be9f813174efb47d706a8601930d04/client/runtime/architecture.md) and
[invariants](https://github.com/mr-exception/salimon/blob/8f112bb6e6be9f813174efb47d706a8601930d04/client/runtime/invariants.md) own composition; universe
[chart invariants](https://github.com/mr-exception/salimon/blob/8f112bb6e6be9f813174efb47d706a8601930d04/client/universe/invariants.md) own route validation.
`session/narrative/tests.rs` checks paid recovery, distinct leads, route/save
failures, aimed physical inspections, optional survival onboarding and a declared
one-hour deterministic session. The existing canonical mine/carry/Fabricator/disk
integration test remains the real-material acquisition boundary; fixture placement
is never reported as walking or flight.

Run `cargo test --workspace --locked`, workspace build/all-target Clippy/fmt,
script tests and documentation checks. Run
`python scripts/verify_narrative.py --binary target/debug/salimon-client --output artifacts/opening-evidence`
on Linux Xvfb/software Vulkan for production GPU originals at the opening, both
physical sites, both inspectable clues and both recovered packets. Its four
explicit narrative fixtures declare placement and, for recovered packets, input
materials; they are unavailable in ordinary play. Run the environmental-audio
scenario with a fresh `SALIMON_SETTINGS_FILE` and export production callback PCM
using the maintained [audio checks](planetary-audio.md). Cloud synthesis/captures
are not a physical listening trial, one hour of human play, Metal qualification
or hardware FPS measurement. Release owners remain #720–#723/#661 and #659.
