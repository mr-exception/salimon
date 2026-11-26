## Gear Bench and shared consumer order (#918)

The preinstalled `scout:gear-bench` at (2.2, −0.1 m) uses the existing shared
industrial console and context-menu/auto-approach adapter. Its shallow wall-mounted
footprint and matching art preserve the starboard crate-to-cockpit passage. Its independent
`scout:gear-bench:input` stores 50,000 g including active reservations. Recipes
and 2,000 W work use `gearBench.ts`; the approved table is in
[the survival contract](spacesuit-and-hazards.md#gear-crafting-bench-918-and-repair-919).
One active job or one output blocks another start. Claims create one fresh full-HP
identity in inventory; rejected capacity/revision/save checks keep the tray intact.

H01's approved order supersedes E03/E05's historical draw-first session ordering:
on each sole tick convert once, pay Fabricator first, then Gear Bench, each from
the remaining reserve. Both receipts and energy commit once. The pure Core API
still supports its original draw-before-conversion operation; the session composes
conversion with deterministic consumer allocation. No offline progress/cancel.
Schema v19 migrates earlier saves with empty bench/buffer. Per-recipe completed
receipts account for consumed raw inputs; tray/carried gear carries only declared
output mass. Input minus output is byproduct-free consumed material, including
T2 (1,000 g) and T3 (2,500 g) loss; no duplicated raw payload.

Focused coverage: `gearBench.test.ts`, affected Fabricator/Core/session tests,
and production `tests/gear-bench.spec.ts`. Validation records belong to the PR;
original captures and raw state stay in ignored `client/test-results/`/CI artifacts.

# Browser Core + Fabricator vertical slice — decision contract (E01 / #873, E03 / #875)

**Status: Core processing, E05 Fabricator domain and E06 reachable console implemented (#878).** The E02 Core-only values listed immediately below were approved by the owner; issue #874 implements the durable Core buffer/state boundary. The E05 values below were approved separately; E07 marker deployment is approved and implemented. A docs PR alone does not authorize the remaining runtime features.

> **E02 Core-only approval (2026-11-22, #874):** The owner approved the proposed independent `scout:core:fuel` 50,000 g buffer, 2,000,000 J reserve cap, 500,000 J initial reserve for both new and v1–v13 migrated expeditions, biomass 2,000 J/g, mineral 500 J/g, 10,000 W maximum generation, and job draw before conversion in a future single-tick machine loop. Issue #874 ships only the durable fuel account, exact transfers and v14 Core state; it does **not** convert fuel, add UI, gate propulsion or settle Fabricator product/use decisions. The other E01 proposals and TBDs remain pending their respective implementation approvals. The historical v14 accumulator was strictly zero. E03 migrates it to exact 1/60-joule accounting in v15.

> **E05 approval (2026-11-23, #877, active implementation chat):** The owner approved 1,000 g mineral → one 1,000 g field-marker kit (stack limit 1), 20 seconds at 5,000 W (100,000 J), one-item tray, no cancellation after start, and no offline progress. E07 implements the separately approved deployment rules below. The independent input ID is `scout:fabricator:input`; reserved grams still occupy its 50,000 g capacity.

## Ground truth / ownership

This contract applies only to the active React/TypeScript single-player browser. The current `resourceSession.ts` publishes **schema v17** at `salimon:expedition:v1`, retains `landing-resources:v2`, and registers the trusted `field-marker-kit:v1` count item. The active sources expose `mineral` and `biomass` (the original authored source quantities are 60,000 g and 15,000 g). `resourceInventory.ts` validates integer-gram raw quantities and count items; `resourceLedger.ts` owns revision-bound, conserved raw transfers; `scoutResourceContainer.ts` provides the existing 50,000 g `scout:storage` store. The Core has a reachable console; the reachable Fabricator console provides input transfers, start, paid status and output claim.

All machines belong to `ship:scout` and are physically accessed at the existing `scout:core` anchor (1.9, 1.8 m) and `scout:fabricator` anchor (-1.85, 3.9 m) in `scoutLayout.ts`; use current nearby selection, same-scene reach, E/Use, keyboard/modal/Escape and focus ownership. They are **not remotely operable**. Both use the existing 1.9 m center-distance reach from `scoutLayout.ts` and `interactionInReach`; shared collision and production walking routes determine access. E08 exercises reachable positions rather than teleporting to the anchors.

## Proposed minimum economy — approval matrix

Every value labelled **P** is a concrete *proposal*, not a shipped or owner-approved value. Existing **A** rows are already established contracts. All time is simulated foreground time unless specified.

| Decision | Candidate value / exact units | Status | Reason / edges |
| --- | --- | --- | --- |
| Player carrying | 10,000 g, 12 slots; raw resource one slot per type | A | Weight and slots are independently checked. No volume. |
| Existing raw store | `scout:storage`: 50,000 g raw only | A | Do not reuse it as either machine buffer. |
| Core buffer | `scout:core:fuel`: 50,000 g, raw only, no slot cap | A | Independent 50 kg Core buffer. |
| Fabricator input | `scout:fabricator:input`: 50,000 g, raw only, no slot cap | E05 | Separate from Core and storage. |
| Starter fuel eligibility | `biomass` 2,000 J/g; `mineral` 500 J/g | A | Both existing source types can fuel a first run. |
| Core peak generation | 10,000 W (= 10,000 J/s) | A | Caps Core generation. |
| Core energy reserve | 2,000,000 J maximum; 500,000 J initial for fresh and migrated expeditions | A | A nonempty start will support initial fabrication. |
| Core conversion | Prefer higher J/g, then ascending stable resource ID; process as integer-gram discrete debits against a continuous energy budget | E03 | Deterministic across reload, ticks and unordered maps. No whole-kg rounding. |
| Core rounding | Each elapsed 1/60 s step accrues exactly 10,000/60 J in **integer sixtieths of a joule**; consume 1 g only if the whole gram's yield fits remaining reserve **and** accrued generation budget; carry fractional budget | E03 | No fractional-gram material creation or lost fractional ticks. Cap budget to one gram of the highest eligible yield when reserve is full, and never mint free joules. |
| Reserve-full behavior | Pause conversion when no complete eligible gram fits the energy headroom; resume after draw | E03 | No input debit if energy cannot be credited atomically. |
| Core idle draw | 0 W; no background drains or battery self-discharge | E03 | Makes a bounded and testable prototype. |
| Fabricator job | `field-marker-kit:v1`: debit/reserve 1,000 g `mineral`, produce one count item of 1,000 g, stack limit 1 | E05 / E07 | One real mass-bearing item and a single job. E07 consumes the kit into a persistent outdoor marker; existing notebook observations stay free. |
| Fabricator processing | 20 simulated seconds at 5,000 W, total 100,000 J | E05 | Duration * power equals energy cost; no free completion. |
| Power allocation | Draw the whole consumer tick from reserve first, then convert fuel to replenish reserve; other machine draws are 0 W | E03 | One global tick, no alternating-order double spend; if available budget below requested draw, job makes **no progress** this step and no energy is debited for that step. |
| Tray | 1 completed job, 1 item, 1,000 g total; one active job maximum | E05 | Full tray blocks starting another job until claim. Never auto-credit player inventory. |
| Claim | On deliberate E/Use claim, validate 1 free count-item slot and 1,000 g free carried mass; atomic tray → inventory transfer | E05 | On full slots/mass keep output in tray with separate reason. No loss, duplicate or implicit discard. |
| Offline | No Core conversion or Fabricator progress while tab closed/unloaded; clock starts from restored saved tick, no wall-clock catch-up | E05 | Consistent with explicit foreground machine loop; offline fatigue remains governed by survival rules. |
| Flight | **Keep `simulation.ts` unlimited-energy propulsion** (100/50 MW prototype thrust law) | A / exclusion | Core energy MUST NOT gate thrust, turns, takeoff/landing, cockpit power or scanner. Separate approval needed. |

### Atomic conversion and reservation semantics

1. A player transfer to either buffer uses the existing expected-revision, strictly positive integer gram, all-or-nothing pattern; unavailable grams, over-capacity, invalid IDs, save failure and stale revisions leave **all** endpoints, energy, tick and revision unchanged.
2. The Fabricator reserves the exact 1,000 g of mineral **on Start**. Reservation is represented as a distinct, immutable job-input account, removed from *withdrawable* input grams but still contributing to ship mass until completion. A Start failure (including full tray or insufficient input) changes nothing. **One job at a time; no queue beyond active + completed tray** for E01.
3. On completion, the reserved 1,000 g becomes the 1,000 g count item on the tray atomically. Input mass must not be counted once in both reservation and tray. There are no modeled byproducts or mass loss in this candidate recipe. Claims change only the tray/inventory location, not the total ship payload when aboard.
4. The Core has no user-set consumption reservation or batch queue. At each simulated step choose the highest-yield **eligible** material, tie-break lexicographically by stable ID; debit only a complete gram in the same state transition that credits exact energy. A gram in the conversion operation is no longer withdrawable. No hidden fractional material is consumed, and generation never exceeds the exact watt/energy limits. Recompute eligibility as energy headroom and buffer contents change.
5. Explicit withdraw actions target **unreserved, unconverted** grams only; reject a request that includes Fabricator reserved grams or Core already converted grams. No machine-to-machine transfer in E01; route through player inventory after explicit retrieval/deposit. Reject simultaneous Start/withdraw/claim races via one revision.
6. During work the Fabricator holds reserved grams, accrued progress ticks and spent joules. A power loss *pauses* job progress without resetting reservation or cumulative energy spent; restored power resumes. Closing panel, moving away, leaving ship, focus loss or tab unload cancels the **interaction UI**, not the already committed job. A terminal ship state cannot silently duplicate committed machine contents.
7. **E05:** no cancel command once started; leave the job resumable. Failed start/progress/claim writes publish no partial transaction. E07 records each deployed kit separately from free notebook marks, and validates carried + deployed kit count against claimed outputs.

### Tick, power and persistence invariants

- Work only from the session's **single authoritative simulated tick** (60 ticks/s). Do not use a second `setInterval`, `Date.now`, or independent RAF time source for generation or crafting. Make batched and individual ticks equivalent; no offline wall-clock conversion. Use exact integer gram and exact sixtieth-joule and integer progress fields; reject NaN, negative and unsafe integers.
- Process each tick by committing an input/debit and output/credit **together** to one local session snapshot. Fund the whole requested consumer tick from the pre-tick reserve first; if insufficient, the job pauses without a partial paid tick. Then convert fuel to replenish reserve, subject to cap. Choose and convert fuel only within the cap; no energy should be credited then lost to a full reserve. E03 implements the approved job-draw-before-conversion order. Verify full-reserve behavior through deterministic replay.
- The browser's current resource ledger validator equates initial raw resource mass with raw source + player + store mass. **It must be extended, not bypassed**: include all machine raw accounts, reserved jobs, tray items and manufactured count items. Define raw-material-to-item conversion as a typed, explicitly authorized mass-balanced transformation instead of pretending the raw resource still exists. Record per-resource transformed totals or an equivalent immutable transaction journal sufficient to reject replay/fabricated mass. Generated geologic deposits retain their own source identity/depletion accounting.
- The current `resourceLedgerMass` returns player mass and raw ship-store mass. Ship mass must count Core input, Fabricator input, reserved job and tray **exactly once**, regardless of UI visibility, with the player payload counted only aboard. Mass converted to Core energy leaves the ship as fuel mass (no automatic return of consumed grams). Manufacturing conserves mass by construction for the candidate recipe.
- Schema **v14** keeps the existing localStorage key and catalog version. It validates the trusted Core store and Core reserve. v1–v13 migrate to an empty fuel buffer, 500,000 J reserve and zero conversion budget while preserving existing source depletion, revision and expedition data. Original bytes remain until the next successful commit. E05 adds those trusted catalogs, durable reservations, progress and tray state in schema v16.
- An invalid or missing v14 Core field or store, unknown future schema, incompatible item catalog or corrupt envelope must fail closed, preserve source bytes and block new writes; never silently reset the campaign. A valid v13 migration cannot mint machine fuel or output items. Revision/compare-and-replace/Web Lock behavior continues unchanged.
- The original E01 proposal introduced no runtime change. E02 and E03 implement the Core portions; E05 implements the Fabricator domain checks below; E04 delivers the Core console and E06 owns the Fabricator console.

### Required downstream implementation checks (E02–E07)

| Boundary | Required scenario |
| --- | --- |
| Pure Core model | yield ranking/ties, exact gram debit, budget carry, full reserve pause, power cap, split vs batched ticks, no fuel/no power |
| Fabricator model | recipe and trusted catalog, 20-second/100 kJ accounting, single job, tray full, power pause/resume, no-cancel, claim success/failure |
| Transfers / conservation | mass exactly once across inventory, raw store, both buffers, reservation and tray; rejection/stale revision/save failure atomic |
| Save/migration | fresh and v1–v13 migration, v14 round trip at every transition, corruption rejection, no replay/duplication |
| Browser integration | E/Use at both existing anchors, keyboard/Escape/focus/range, full inventory/tray, long work, reload and observable power feedback |
| Flight regression | unchanged energy-free launch/land, manual propulsion and scanner; dynamic payload mass responds to fuel/crafted transfers |

Use docs-only local link/content review for #873. Node tests and build are N/A because this PR must not touch runtime.

## Scope exclusions

No real propulsion fuel or ship/scanner power gating; no new planetary resources, geology or mining rates; no multiplayer, network authority or real offline simulation; no gear bench, suits/modules/upgrades, multi-job queue, advanced refinery, dynamic recipes, alternate energy tech or economy balancing beyond this minimum path. Legacy native Rust wattage, joules/litre, density and recipe defaults are **not** authoritative browser contracts.

## Decision gate and dependencies

**E07 approval (2026-11-23, active implementation chat):** The owner approved deploying one carried kit at the current outdoor position into a persistent visible marker with local direction/range guidance, preserving free notebook marks and free marker renaming, with no reclaim. Failed, cancelled or stale actions consume nothing; scene changes and reload preserve deployment. Guidance is bounded to 384 m on the same surface; up to 256 deployed markers are retained independently of the 256 notebook observations. Review design against [active resource contract](resource-inventory-and-gathering.md), [browser status](current-client-status.md), [save continuity](save-contract.md) and [flight contract](browser-launch-landing.md).

#873 established the E01 design contract. Scope beyond the E02 approval follows the remaining decisions above. Existing #783–#788 and #838–#845 / #847–#854 are prior foundations, not evidence of delivered Core/Fabricator behavior.

## E03 deterministic processing and power API (#875)

`scoutEnergyCore.ts` owns one pure tick; `ResourceSession.advance(world, power)`
is the only production adapter. Integer-watt requests represent one active
Fabricator consumer (default 0 W); `{ watts, paused: true }` pays nothing.
The consumer receives an all-or-nothing draw from the pre-tick reserve, then
whole-gram conversion refills the reserve. An insufficient draw makes no paid
progress; newly converted energy becomes available on the following tick.
No Fabricator job, recipe, item or UI is introduced here.

`reserveJ60`, `conversionBudgetJ60` and `spentJ60` store integer 1/60 J units.
Generation adds 10,000 units per tick while eligible fuel exists. Choose the
highest-yield gram that fits both budget and reserve headroom, then stable ID.
Budget is capped to the highest available fuel's one-gram yield; it resets when
fuel is depleted and cannot accumulate an offline backlog. Full-reserve waiting
may retain this bounded budget. Sub-gram headroom leaves fuel untouched. All
unconverted grams remain withdrawable; there is no separate fuel reservation.

The committed `corePower` receipt exposes `powered`, `idle`, `empty`, or `full`
and consumer `idle`, `powered`, `paused`, or `insufficient-power`, plus exact
joules drawn/generated and per-resource gram debits. `powered` means this tick
paid a consumer or converted fuel; otherwise `full` means exact capacity,
`empty` means no reserve/eligible fuel, and `idle` covers waiting for budget or
whole-gram headroom. Receipts clear on duplicate/rejected ticks and storage
failure. HUDs read receipts and durable state; they never simulate energy.
Future job progress must join the same save transaction before publication.

Schema v15 preserves the key/catalog and migrates v14's legitimate initial
reserve and zero budget losslessly, without rewriting bytes until commit.
`convertedGrams` joins the existing ledger conservation equation and is excluded
from payload. Initial energy + converted yields must equal reserve + cumulative
expenditure. Invalid sums, unsafe counters, fractional grams/units and future
schemas fail closed. `core.tick` records the last committed processing tick;
reload restores the single world clock to at least that tick, preventing replay.
No closed/inactive-tab catch-up is performed. Flight/cockpit/scanner prototype
power remains independent.

Checks: `corePower.test.ts`, `scoutEnergyCore.test.ts`, existing ledger/session,
survival and flight tests, and built `tests/core-power.spec.ts`. Raw state and
original captures stay in ignored `client/test-results/` and existing CI artifacts
(14-day retention); PR descriptions carry delivery and review results.


### E04 console delivery (#876)

The reachable `scout:core` fixture now opens the existing shared industrial
EquipmentDialog/transfer form. Its buffer/reserve/status and last-tick W receipts
come directly from ResourceSession; UI timing never simulates energy. Pending
transfers normalize fuel plus converted totals to tolerate ordinary conversion,
while other material mutations retain stale-revision rejection. The confirmed
whole grams must still exist in the live buffer; converted fuel cannot be
withdrawn. Escape, reach, focus and save failures preserve committed state.
See the [controls and built-browser checks](../client/README.md#reachable-energy-core-console-876).

## Fabricator domain and durable state (E05 / #877)

`scoutFabricator.ts` owns the frozen recipe constants, validated one-job state,
monotonic started/completed/claimed job receipts, explicit reserved grams, remaining
60 Hz ticks, persisted power-paused state and count-item tray. `ResourceSession`
exposes `depositFabricatorInput`, `withdrawFabricatorInput`, `startFabricator` and
`claimFabricator`; mutations share the exact ledger revision and atomic save gate.
E06 enforces reachable ship-local interaction through the shared machine adapter
and industrial EquipmentDialog. There is no instant-craft UI or new remote machine control.

An active job owns the Core consumer request (5,000 W). The existing sole world
clock draws the entire tick before fuel conversion; only its paid receipt advances
the job. Power loss keeps inputs reserved and pauses progress; closing a panel or
changing view cannot cancel the committed job. Completion turns the reservation
into one tray item atomically. A full tray blocks another start. Claim requires
exact carrying mass and stack slots, the current revision and matching tray job ID.
The E07 `deployMarker` command is the sole kit removal path; generic item grants/removals cannot bypass manufacture/claim/deployment accounting.

Schema v16 retains the storage key and resource catalog version. v1–v15 migrate
with empty input/job/tray/receipts and trusted kit metadata, preserving original
bytes until a successful write. Existing catalog entries still require exact
metadata. Unknown fields, invalid recipes/IDs/progress/reservations, unsupported
versions, unpaid work, invented or missing output counts and raw mass imbalance
fail closed. Fabricator-paid ticks plus a separate historical/non-Fabricator power
receipt must equal total Core expenditure exactly; unrelated past draw cannot pay
for fabricated job progress. Ledger material totals include reserved mineral and completed
transformations; physical ship mass includes the reservation or tray once, then
player cargo only aboard. Fuel burned by the Core remains a separate mass sink.

Checks: `scoutFabricator.test.ts` covers limits, partial/stale/failed transactions,
full tray/cargo, exact grams/counts/100 kJ, no-cancel persistence, power restoration,
reload/migration and malformed saves. `tests/fabricator.spec.ts` seeds a job through
the real domain API and exercises the built production clock, pause/hidden tab,
completion/power loss and reload; it does not claim the E06 actual-control journey.
Original JSON/captures/logs stay in ignored `client/test-results/`; validation belongs
in the PR and existing 14-day CI artifacts.


### E06 console delivery (#878)

`ResourceContainerInteraction` admits the reachable `scout:fabricator`, routes
exact raw transfers to its independent input store and invokes the existing
revision-bound start/claim APIs. Reservation-aware capacity includes job inputs;
withdrawals expose only unreserved grams. `ResourceContainerPanel` composes the
trusted recipe, blocked/ready/powered/paused/complete job status, Core receipt and
persistent tray with shared UI primitives. No UI timer advances a job or grants
an item. Full carrying weight/slots preserve output with a specific reason.

See [controls and validation](../client/README.md#reachable-fabricator-console-878).
`tests/vertical-slice.spec.ts` owns the combined actual-control mine/storage/fuel/start/
power-loss/refuel/claim/use/cockpit/reload loop. `tests/fabricator-console.spec.ts` owns negative transactions/focus and fixed-policy
repeat captures at 960/1280/1920. Evidence remains in ignored output and existing
CI artifacts; the PR records the same-session review. E07 implements the separately approved deployment rules below.


## E07 usable field marker (#879)

`fieldMarker.ts` validates durable surface deployment records and bounded local
guidance. `ResourceSession.deployMarker` requires outdoor world authority, a living
character able to work, a carried kit, current revision and the reviewed position.
It atomically removes one kit and adds one immutable-position deployment record.
Duplicate positions, full 256-marker capacity, malformed names and failed/stale
saves publish nothing. The ledger enforces **claimed = carried + deployed**;
manufactured mineral is counted once and deployed kits are outside ship payload.
No remote placement, reclaim, equip slot, offline work or propulsion change.

`MarkerDeployment.tsx` exposes the inventory action and explicit confirmation;
Escape, cancel, blur, blocked authority or changed position/kit count discards the
review. Ordinary foreground time continues. The field notebook provides free
renaming and guidance selection; HUD direction/range only appears on the same
surface within 384 m. A flag glyph distinguishes kits from free observation cues.
Schema v17 requires validated deployment records; v1–v16 migrate with none and
retain original bytes until successful commit. See the [gameplay guide](../client/README.md#deploy-a-fabricated-field-marker-879).


## E08 built-production acceptance (#880)

The production bundle served by Vite preview is the reproducible deployment
candidate. This check does not publish or verify a live hosted deployment.
`tests/vertical-slice.spec.ts` extends the E06/E07 journey instead of maintaining
another copy; shared valid expedition seeds/control helpers live in
`tests/fabricatorFixtures.ts`. The cockpit project remains in the existing full
world CI group. PR smoke retains its existing journey coverage under the new
`SLICE-880 built` selector; no additional workflow or baseline policy is introduced.

| Criterion | Reproducible check / evidence |
| --- | --- |
| Complete playable loop | `SLICE-880 built mine…` uses real keyboard/mouse controls on the built route; low-reserve prior-expedition seed forces outage. Trace, original power-loss/marker/cockpit images and `conservation.json` retain state. |
| Exact conservation | Each recorded transaction stage checks 60 kg authored mineral = source + carry + stores + reserved + converted + tray/carried/deployed products; reserve + spent = initial + conversion; paid ticks bind expenditure to the single job. |
| Interruptions and clock | Pause and injected hidden-document events freeze durable state; reload preserves interrupted work; cockpit has one 60 Hz clock, marker and receipts survive view/reload. Escape/blur cancel drafts. |
| Capacities | Full Core/Fabricator 50 kg buffers reject surplus; existing FAB-UI-2 full carry rejects claim without losing tray then recovers. `scoutFabricator.test.ts` covers tray, weight/slots, reservation capacity and duplicate receipts. |
| Save failures | Built stale/quota-failed claim preserves bytes and tray, reload recovers exactly once; malformed/unsupported/unpaid saves remain untouched. Competing-tab Web Lock prevents job advancement/claim. |
| Visuals | Existing 960/1280/1920 fixed-policy console and marker repeats plus injected CSS/missing-marker negatives; original screenshots and traces are retained, references/thresholds unchanged. |
| Domain/build | `npm test`; `npm run build`. Record actual PASS/FAIL and exact tested revision in the PR. |
| Smoke / full | `npm run test:browser:smoke` is bounded; existing Navigation client and daily/manual full regression are separate outcomes. Record NOT RUN until current-revision results are supplied; smoke green never implies full green. |

Completion and same-session review belong in the PR with head/base SHA, exact
commands, PASS/FAIL/NOT RUN and original evidence paths. Local artifacts remain
ignored under `client/test-results/`; CI artifacts retain 14 days. The user tracks
CI and reports results in the same chat. CI awaiting that report is UNVERIFIED,
not QA acceptance. No M1-only qualification is required for this browser slice.

Limits: this is one mineral recipe, one active job, one tray item and a deliberate
surface marker with bounded 384 m local guidance. No offline conversion/crafting,
remote automation or server authority. Future **suits**, **planetary hazards**
(the hazard domain is not production planetary sampling), **ecology**,
**multiplayer** and **propulsion energy** remain deferred. Flight retains its
unlimited-energy prototype; cabin power is separate. Wider economy, equipment,
recipes, full charting and native systems are outside this acceptance claim.

## Approved planetary survival contract (#913)

[Survival and equipment](spacesuit-and-hazards.md) owns approved gear identities, mass/custody, environment publication, proportional wear, bench recipes/power order and repair rounding. #914–#922 deliver these separately; this contract publication changes no runtime behavior.
