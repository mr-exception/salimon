# Browser survival (#787)

`client/src/survival.ts` owns integer, deterministic 60 Hz hunger/fatigue/health.
`ResourceSession` owns admission and the schema-v6 transaction with sources,
player inventory, count definitions, ship stores and safe standing location.
`TopDownView` drives the same owner outdoors, indoors and while the radar is open.
The radar's own ship clock never advances survival a second time.

New characters have **200 HP** and full hunger/fatigue. This supersedes the
historical native 100 HP maximum. No supplies, items or world resources are granted.
A point is 38,880 integer units: hunger loses 36 units per tick (30 minutes
100→0), fatigue loses one (18 hours). At zero hunger, each subsequent 600 ticks
removes one HP. Passive recovery adds one HP per continuous 3,600 eligible ticks
with hunger ≥70 and fatigue ≥50; breaking either bound clears the interval.
Damage precedes healing; terminal zero HP freezes time and gathering.

Production admits fixed steps from foreground RAF elapsed time with the existing
250 ms frame cap. Pause [P] stops both world/survival and navigation; hidden pages
stop active clocks, release input and checkpoint. Ordinary inspection/storage
menus keep time active. Focus, resume and save grant no recovery. Reopening a
validated v6 save credits **only fatigue**, at 100 points over six hours, capped
at full. Credit is rounded down to an integer meter unit (less than 0.000026
point), committed before use, and protected by a monotonically increasing UTC
watermark. Reopening at the same UTC or clock rollback grants nothing again.
No offline hunger, starvation or regeneration is simulated. As with the historical
contract, time lost since the last successful checkpoint can count as closed time
following a crash; no active hunger or health changes are inferred for that gap.

Schema v1–v5 introduces fresh meters once at its next successful save, preserving
mass, capacities, stores and location. Unsupported/corrupt saves are retained and
play fails closed; there is no reset button for them. Autosave is once per second,
on movement stop, mining/transfer/item transaction, pause/background/pagehide,
and immediately at death. Synchronous replacement plus the Web Lock and stale-byte
guard preserves one whole revision. Uncommitted partial mining is discarded.
A failed write freezes play and restores the last committed meter snapshot; no
resource or meter mutation is published by that failed transaction.

At zero HP, **Start fresh character · keep world resources** is the explicit
reset. It restores the character's meters only, retains location, all inventory,
ship stores and source depletion, and never replaces an unsupported historic save.
This is a local prototype run boundary, not multiplayer resurrection/death authority.

## Restrictions and deliberate follow-ups

The historical survival contract's established movement thresholds are reused:
hunger ≥50 / [20,50) / <20 yields 100% / 90% / 75%; fatigue >50 /
(20,50] / (0,20] / 0 yields 100% / 85% / 65% / 30%. Multiply components.
The resulting minimum is 22.5%, above the historical provisional 10% floor.
Zero fatigue denies gathering and cockpit entry, revokes current piloting and
manual throttle, and requires fresh cockpit entry/input after recovery. World
velocity and committed assists are preserved. Zero HP denies movement and work.

Deferred tuning follow-up [#799](https://github.com/mr-exception/salimon/issues/799): the historical **provisional** 75% mining work-rate
multipliers and 250 ms discrete-action recovery are not approved 2D balancing.
Mining retains the current 1 kg/s until owner approval of those values and their
composition. HUD warns about established movement/exhaustion effects only.
Future machine/carrying/Core action restrictions belong with those production
systems; no such playable actions exist yet. Food/medkits, consumption, wounds,
hazards, oxygen, radiation, multiplayer and medical treatment are separate issues
(#788 and later content work); this change does not introduce their catalogs.

The top-left HUD exposes native labelled meters and numerical text. Values change
at whole meter points/HP; polite announcements contain condition transitions,
not per-tick readings. It remains visible in navigation and stacks above the scene
at narrow widths. Browser tests retain outdoor, interior, return, low-condition
and terminal originals; navigation baseline approval remains unchanged.
