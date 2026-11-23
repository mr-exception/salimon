# Browser Core + Fabricator vertical slice — decision contract (E01 / #873, E03 / #875)

**Status: Core processing implemented through E03 (#875); Fabricator decisions pending.** The E02 Core-only values listed immediately below were approved by the owner; issue #874 implements the durable Core buffer/state boundary. Other proposed balance values, item effects and machine commands remain unimplemented and require their own approval. A docs PR alone does not authorize the remaining runtime features.

> **E02 Core-only approval (2026-11-22, #874):** The owner approved the proposed independent `scout:core:fuel` 50,000 g buffer, 2,000,000 J reserve cap, 500,000 J initial reserve for both new and v1–v13 migrated expeditions, biomass 2,000 J/g, mineral 500 J/g, 10,000 W maximum generation, and job draw before conversion in a future single-tick machine loop. Issue #874 ships only the durable fuel account, exact transfers and v14 Core state; it does **not** convert fuel, add UI, gate propulsion or settle Fabricator product/use decisions. The other E01 proposals and TBDs remain pending their respective implementation approvals. The historical v14 accumulator was strictly zero. E03 migrates it to exact 1/60-joule accounting in v15.

## Ground truth / ownership

This contract applies only to the active React/TypeScript single-player browser. The current `resourceSession.ts` publishes **schema v14** at `salimon:expedition:v1`, retains `landing-resources:v2`, and declares `ITEM_CATALOG = registerCountItems({})`. The active sources expose `mineral` and `biomass` (the original authored source quantities are 60,000 g and 15,000 g). `resourceInventory.ts` validates integer-gram raw quantities and count items; `resourceLedger.ts` owns revision-bound, conserved raw transfers; `scoutResourceContainer.ts` provides the existing 50,000 g `scout:storage` store. The machine graphics are inspection-only.

All machines belong to `ship:scout` and are physically accessed at the existing `scout:core` anchor (1.9, 1.8 m) and `scout:fabricator` anchor (-1.85, 3.9 m) in `scoutLayout.ts`; use current nearby selection, same-scene reach, E/Use, keyboard/modal/Escape and focus ownership. They are **not remotely operable**. Exact equipment reach is **TBD: use an existing interaction policy only after checking collision and access in a browser fixture**; do not hardcode an unreviewed radius.

## Proposed minimum economy — approval matrix

Every value labelled **P** is a concrete *proposal*, not a shipped or owner-approved value. Existing **A** rows are already established contracts. All time is simulated foreground time unless specified.

| Decision | Candidate value / exact units | Status | Reason / edges |
| --- | --- | --- | --- |
| Player carrying | 10,000 g, 12 slots; raw resource one slot per type | A | Weight and slots are independently checked. No volume. |
| Existing raw store | `scout:storage`: 50,000 g raw only | A | Do not reuse it as either machine buffer. |
| Core buffer | `scout:core:fuel`: 50,000 g, raw only, no slot cap | A | Independent 50 kg Core buffer. |
| Fabricator input | `scout:fabricator:input`: 50,000 g, raw only, no slot cap | A capacity / P ID | Separate from Core and storage. |
| Starter fuel eligibility | `biomass` 2,000 J/g; `mineral` 500 J/g | A | Both existing source types can fuel a first run. |
| Core peak generation | 10,000 W (= 10,000 J/s) | A | Caps Core generation. |
| Core energy reserve | 2,000,000 J maximum; 500,000 J initial for fresh and migrated expeditions | A | A nonempty start will support initial fabrication. |
| Core conversion | Prefer higher J/g, then ascending stable resource ID; process as integer-gram discrete debits against a continuous energy budget | E03 | Deterministic across reload, ticks and unordered maps. No whole-kg rounding. |
| Core rounding | Each elapsed 1/60 s step accrues exactly 10,000/60 J in **integer sixtieths of a joule**; consume 1 g only if the whole gram's yield fits remaining reserve **and** accrued generation budget; carry fractional budget | E03 | No fractional-gram material creation or lost fractional ticks. Cap budget to one gram of the highest eligible yield when reserve is full, and never mint free joules. |
| Reserve-full behavior | Pause conversion when no complete eligible gram fits the energy headroom; resume after draw | E03 | No input debit if energy cannot be credited atomically. |
| Core idle draw | 0 W; no background drains or battery self-discharge | E03 | Makes a bounded and testable prototype. |
| Fabricator job | `field-marker-kit:v1`: debit/reserve 1,000 g `mineral`, produce one count item of 1,000 g, stack limit 1 | P / **TBD use effect** | One real mass-bearing item and a single job; consuming it to create a field mark is a suggested use, but the existing exploration mark API is currently free. **Do not change mark gameplay until owner approves the use effect.** |
| Fabricator processing | 20 simulated seconds at 5,000 W, total 100,000 J | P | Duration * power equals energy cost; no free completion. |
| Power allocation | Draw the whole consumer tick from reserve first, then convert fuel to replenish reserve; other machine draws are 0 W | E03 | One global tick, no alternating-order double spend; if available budget below requested draw, job makes **no progress** this step and no energy is debited for that step. |
| Tray | 1 completed job, 1 item, 1,000 g total; one active job maximum | P | Full tray blocks starting another job until claim. Never auto-credit player inventory. |
| Claim | On deliberate E/Use claim, validate 1 free count-item slot and 1,000 g free carried mass; atomic tray → inventory transfer | P | On full slots/mass keep output in tray with separate reason. No loss, duplicate or implicit discard. |
| Offline | No Core conversion or Fabricator progress while tab closed/unloaded; clock starts from restored saved tick, no wall-clock catch-up | P | Consistent with explicit foreground machine loop; offline fatigue remains governed by survival rules. |
| Flight | **Keep `simulation.ts` unlimited-energy propulsion** (100/50 MW prototype thrust law) | A / exclusion | Core energy MUST NOT gate thrust, turns, takeoff/landing, cockpit power or scanner. Separate approval needed. |

### Atomic conversion and reservation semantics

1. A player transfer to either buffer uses the existing expected-revision, strictly positive integer gram, all-or-nothing pattern; unavailable grams, over-capacity, invalid IDs, save failure and stale revisions leave **all** endpoints, energy, tick and revision unchanged.
2. The proposed Fabricator reserves the exact 1,000 g of mineral **on Start**. Reservation is represented as a distinct, immutable job-input account, removed from *withdrawable* input grams but still contributing to ship mass until completion. A Start failure (including full tray or insufficient input) changes nothing. **One job at a time; no queue beyond active + completed tray** for E01.
3. On completion, the reserved 1,000 g becomes the 1,000 g count item on the tray atomically. Input mass must not be counted once in both reservation and tray. There are no modeled byproducts or mass loss in this candidate recipe. Claims change only the tray/inventory location, not the total ship payload when aboard.
4. The Core has no user-set consumption reservation or batch queue. At each simulated step choose the highest-yield **eligible** material, tie-break lexicographically by stable ID; debit only a complete gram in the same state transition that credits exact energy. A gram in the conversion operation is no longer withdrawable. No hidden fractional material is consumed, and generation never exceeds the exact watt/energy limits. Recompute eligibility as energy headroom and buffer contents change.
5. Explicit withdraw actions target **unreserved, unconverted** grams only; reject a request that includes Fabricator reserved grams or Core already converted grams. No machine-to-machine transfer in E01; route through player inventory after explicit retrieval/deposit. Reject simultaneous Start/withdraw/claim races via one revision.
6. During work the Fabricator holds reserved grams, accrued progress ticks and spent joules. A power loss *pauses* job progress without resetting reservation or cumulative energy spent; restored power resumes. Closing panel, moving away, leaving ship, focus loss or tab unload cancels the **interaction UI**, not the already committed job. A terminal ship state cannot silently duplicate committed machine contents.
7. **TBD before Fabricator implementation:** if a user cancels an in-progress job, whether reserved resources are refunded and whether consumed joules are lost. E01 candidate: no cancel command once started; leave the job resumable. **TBD before crafting is made usable:** whether a field marker kit consumes on creation of a mark, what happens to already free marks and how marker count is enforced at the existing mark API.

### Tick, power and persistence invariants

- Work only from the session's **single authoritative simulated tick** (60 ticks/s). Do not use a second `setInterval`, `Date.now`, or independent RAF time source for generation or crafting. Make batched and individual ticks equivalent; no offline wall-clock conversion. Use exact integer gram and exact sixtieth-joule and integer progress fields; reject NaN, negative and unsafe integers.
- Process each tick by committing an input/debit and output/credit **together** to one local session snapshot. Fund the whole requested consumer tick from the pre-tick reserve first; if insufficient, the job pauses without a partial paid tick. Then convert fuel to replenish reserve, subject to cap. Choose and convert fuel only within the cap; no energy should be credited then lost to a full reserve. E03 implements the approved job-draw-before-conversion order. Verify full-reserve behavior through deterministic replay.
- The browser's current resource ledger validator equates initial raw resource mass with raw source + player + store mass. **It must be extended, not bypassed**: include all machine raw accounts, reserved jobs, tray items and manufactured count items. Define raw-material-to-item conversion as a typed, explicitly authorized mass-balanced transformation instead of pretending the raw resource still exists. Record per-resource transformed totals or an equivalent immutable transaction journal sufficient to reject replay/fabricated mass. Generated geologic deposits retain their own source identity/depletion accounting.
- The current `resourceLedgerMass` returns player mass and raw ship-store mass. Ship mass must count Core input, Fabricator input, reserved job and tray **exactly once**, regardless of UI visibility, with the player payload counted only aboard. Mass converted to Core energy leaves the ship as fuel mass (no automatic return of consumed grams). Manufacturing conserves mass by construction for the candidate recipe.
- Schema **v14** keeps the existing localStorage key and catalog version. It validates the trusted Core store and Core reserve. v1–v13 migrate to an empty fuel buffer, 500,000 J reserve and zero conversion budget while preserving existing source depletion, revision and expedition data. Original bytes remain until the next successful commit. Future Fabricator work will define trusted recipe/item catalogs and durable job reservations, progress and tray state.
- An invalid or missing v14 Core field or store, unknown future schema, incompatible item catalog or corrupt envelope must fail closed, preserve source bytes and block new writes; never silently reset the campaign. A valid v13 migration cannot mint machine fuel or output items. Revision/compare-and-replace/Web Lock behavior continues unchanged.
- The original E01 proposal introduced no runtime change. E02 and E03 implement the Core portions; Fabricator checks below remain downstream requirements; E04 delivers the Core console.

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

**Remaining decisions** include the recipe and its usable effect, job duration and energy, tray behavior, and the cancellation policy. Settle the field marker effect and existing free marks before changing field-mark gameplay. Review design against [active resource contract](resource-inventory-and-gathering.md), [browser status](current-client-status.md), [save continuity](save-contract.md) and [flight contract](browser-launch-landing.md).

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
