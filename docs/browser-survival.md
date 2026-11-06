# Browser survival (#787)

`client/src/survival.ts` owns deterministic 60 Hz hunger/fatigue/health; [hazards](browser-hazards.md) extend HP with exact fractional accounting.
`ResourceSession` owns admission and the schema-v8 transaction with sources,
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
validated v6–v8 save credits **only fatigue**, at 100 points over six hours, capped
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
guard preserves one whole revision. Checkpointed partial mining work is retained per source; work since the last successful checkpoint may be lost on a crash.
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
The owner approved a **10% movement floor** on 2026-11-06 for #799. Existing
components currently bottom out at 22.5%, so this floor changes no present speed.
Zero HP still denies movement; a floor never revives a terminal run. Zero fatigue
denies gathering and cockpit entry and revokes current piloting/manual throttle.
World velocity and committed assists are preserved; recovery requires fresh input.

## Approved work and action tuning (#799)

[Owner approval](https://github.com/mr-exception/salimon/issues/799#issuecomment-6679680904)
was recorded before implementation. Hunger **<20** applies a **×0.75** mining
multiplier; fatigue **≤20** applies **×0.75**. They multiply to **×0.5625**.
Baseline remains 1 kg/s. Exhaustion and zero-health bans take precedence.
The 30-minute hunger, 18-hour fatigue, six-hour offline recovery and 200 HP
maximum remain unchanged.

Each admitted 60 Hz tick samples its starting survival conditions and adds
16 / 12 / 9 integer work units at baseline / one penalty / both penalties.
**960 units** completes an extraction of up to 1,000 integer grams, limited by
source remainder, free weight and slots. The last capacity/source-limited batch
consumes the full interval; unextracted grams remain in the source. No rounding
of elapsed seconds or fractional material occurs. Residual work stays with its
source, including when switching sources, stowing, moving, cancelling or reloading.
Interruption checkpoints the exact work with clock/location/inventory/depletion.
A full inventory earns no extra work. Pause/background/offline grant no work;
reload always requires re-equipping and explicitly restarting gathering.
Schema v8 requires canonical source-bound work in (0,960); absent entries mean
zero. V1–v7 migrate with zero work and preserve all previously committed resources.
Work debit, gram transfer and survival state use the same atomic save; failed
writes restore the last whole checkpoint and freeze play. No duplicated credit.

Discrete machine actions share **250 ms / 15 active ticks** of recovery after a
successful action. `completeDiscreteAction` is the tested future adapter boundary
for Core intake, Fabricator submission and tray collection. Validate the machine
operation first, then atomically commit its result and returned survival deadline;
rejected operations consume no recovery. Paused/offline time does not advance it,
and save/reload preserves it. Exhaustion/terminal conditions deny ordinary work.
Emergency controls (including shutdown/exit/cancel) bypass the recovery and
exhaustion gate and never extend the deadline. Continuous mining, inspection,
storage transfer, movement and cockpit exit do not acquire machine recovery.
**No playable machine callers exist yet**; their future integrations must use this
boundary. This adds no machine UI, recipes, power or automatic resource grants.
Food/medkits, wounds, oxygen, multiplayer and medical treatment remain separate
content work. The [hazard domain](browser-hazards.md) retains its own source and
equipment catalog boundary.

The top-left HUD exposes native labelled meters and numerical text. Values change
at whole meter points/HP; polite announcements contain condition transitions,
not per-tick readings. It remains visible in navigation and stacks above the scene
at narrow widths. Browser tests retain outdoor, interior, return, low-condition
and terminal originals; navigation baseline approval remains unchanged.
