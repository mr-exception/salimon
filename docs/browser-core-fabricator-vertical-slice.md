# Browser Core + Fabricator vertical slice — proposed decision contract (E01 / #873)

**Status: PROPOSAL — OWNER APPROVAL REQUIRED.** None of the new numeric balance values, item behavior, save additions or machine commands below are implemented or approved. E02–E07 MUST NOT implement this proposal until the owner approves the decision rows and resolves the marked TBD. A docs PR does not itself authorize runtime balance changes.

## Ground truth / ownership

This contract applies only to the active React/TypeScript single-player browser. The current `resourceSession.ts` publishes **schema v13** at `salimon:expedition:v1`, retains `landing-resources:v2`, and declares `ITEM_CATALOG = registerCountItems({})`. The active sources expose `mineral` and `biomass` (the original authored source quantities are 60,000 g and 15,000 g). `resourceInventory.ts` validates integer-gram raw quantities and count items; `resourceLedger.ts` owns revision-bound, conserved raw transfers; `scoutResourceContainer.ts` provides the existing 50,000 g `scout:storage` store. The machine graphics are inspection-only.

All machines belong to `ship:scout` and are physically accessed at the existing `scout:core` anchor (1.9, 1.8 m) and `scout:fabricator` anchor (-1.85, 3.9 m) in `scoutLayout.ts`; use current nearby selection, same-scene reach, E/Use, keyboard/modal/Escape and focus ownership. They are **not remotely operable**. Exact equipment reach is **TBD: use an existing interaction policy only after checking collision and access in a browser fixture**; do not hardcode an unreviewed radius.

## Proposed minimum economy — approval matrix

Every value labelled **P** is a concrete *proposal*, not a shipped or owner-approved value. Existing **A** rows are already established contracts. All time is simulated foreground time unless specified.

| Decision | Candidate value / exact units | Status | Reason / edges |
| --- | --- | --- | --- |
| Player carrying | 10,000 g, 12 slots; raw resource one slot per type | A | Weight and slots are independently checked. No volume. |
| Existing raw store | `scout:storage`: 50,000 g raw only | A | Do not reuse it as either machine buffer. |
| Core buffer | `scout:core:fuel`: 50,000 g, raw only, no slot cap | A capacity / P ID | Owner's independent 50 kg Core buffer; no nested inventories. |
| Fabricator input | `scout:fabricator:input`: 50,000 g, raw only, no slot cap | A capacity / P ID | Separate from Core and storage. |
| Starter fuel eligibility | `biomass` 2,000 J/g; `mineral` 500 J/g | P | Both existing source types can fuel a first run; no new geology. Exact yields need approval. |
| Core peak generation | 10,000 W (= 10,000 J/s) | P | Provides visible rate cap and supports proposed recipe. Not a flight engine rating. |
| Core energy reserve | 2,000,000 J maximum; 500,000 J initial for **fresh** expeditions | P | A nonempty start enables first fabrication without forcing immediate mining. Migrated saves: see below. |
| Core conversion | Prefer higher J/g, then ascending stable resource ID; process as integer-gram discrete debits against a continuous energy budget | P | Deterministic across reload, ticks and unordered maps. No whole-kg rounding. |
| Core rounding | Each elapsed 1/60 s step accrues up to 10,000/60 J in **integer microjoules**; consume 1 g only if the whole gram's yield fits remaining reserve **and** accrued generation budget; carry fractional budget | P | No fractional-gram material creation or lost fractional ticks. Cap budget to one gram of the highest eligible yield when reserve is full, and never mint free joules. |
| Reserve-full behavior | Pause conversion when no complete eligible gram fits the energy headroom; resume after draw | A intent / P atomic rule | No input debit if energy cannot be credited atomically. |
| Core idle draw | 0 W in E01; no background drains or battery self-discharge | P | Makes a bounded and testable prototype. |
| Fabricator job | `field-marker-kit:v1`: debit/reserve 1,000 g `mineral`, produce one count item of 1,000 g, stack limit 1 | P / **TBD use effect** | One real mass-bearing item and a single job; consuming it to create a field mark is a suggested use, but the existing exploration mark API is currently free. **Do not change mark gameplay until owner approves the use effect.** |
| Fabricator processing | 20 simulated seconds at 5,000 W, total 100,000 J | P | Duration * power equals energy cost; no free completion. |
| Power allocation | Core generation supplies Fabricator draw first, then charges reserve; reserve may also discharge to cover a shortage; other machine draws are 0 W in E01 | P | One global tick, no alternating-order double spend; if available budget below requested draw, job makes **no progress** this step and no energy is debited for that step. |
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
7. **TBD before E02–E07:** if a user cancels an in-progress job, whether reserved resources are refunded and whether consumed joules are lost. E01 candidate: no cancel command once started; leave the job resumable. **TBD before crafting is made usable:** whether a field marker kit consumes on creation of a mark, what happens to already free marks and how marker count is enforced at the existing mark API.

### Tick, power and persistence invariants

- Work only from the session's **single authoritative simulated tick** (60 ticks/s). Do not use a second `setInterval`, `Date.now`, or independent RAF time source for generation or crafting. Make batched and individual ticks equivalent; no offline wall-clock conversion. Use exact integer gram, joule (or fixed microjoule accumulator) and integer progress fields; reject NaN, negative and unsafe integers.
- Process each tick by committing an input/debit and output/credit **together** to one local session snapshot. Supply fresh power to the active job first; if insufficient, job pauses (no partial paid tick); otherwise draw the exact deficit from energy reserve and credit any unused generation to storage, subject to cap. Choose and convert fuel only within the cap; no energy should be credited then lost to a full reserve. **TBD exact ordering** between full-reserve fuel conversion and immediate job draw if both happen in one tick; propose job draw first, then generation to replenish, with deterministic replay tests.
- The browser's current resource ledger validator equates initial raw resource mass with raw source + player + store mass. **It must be extended, not bypassed**: include all machine raw accounts, reserved jobs, tray items and manufactured count items. Define raw-material-to-item conversion as a typed, explicitly authorized mass-balanced transformation instead of pretending the raw resource still exists. Record per-resource transformed totals or an equivalent immutable transaction journal sufficient to reject replay/fabricated mass. Generated geologic deposits retain their own source identity/depletion accounting.
- The current `resourceLedgerMass` returns player mass and raw ship-store mass. Ship mass must count Core input, Fabricator input, reserved job and tray **exactly once**, regardless of UI visibility, with the player payload counted only aboard. Mass converted to Core energy leaves the ship as fuel mass (no automatic return of consumed grams). Manufacturing conserves mass by construction for the candidate recipe.
- Proposed next save version **v14**, while keeping the existing localStorage key and catalog version; add strict, trusted machine ID/recipe/item catalogs plus separate versioned machine state for energy, fractional conversion budget, buffer quantities, job reservation/progress/spent energy, and tray. Do not accept arbitrary catalog definitions, foreign machine IDs or floating item counts from saved bytes. Migrations v1–v13 add empty machine raw buffers, no job/tray, and the **TBD approved migration energy default**; preserve old resource source amounts, revision, dry ship mass, ship/location/survival/hazards/journal and old bytes until successful atomic commit. Explicitly approve whether migrated runs get the proposed 500,000 J charge or start at 0 J before writing migration code.
- An invalid/missing current v14 machine field, unknown future schema, incompatible item catalog or corrupt envelope must fail closed, preserve source bytes and block new writes; never silently reset the campaign. A valid v13 migration cannot mint machine fuel or output items. Revision/compare-and-replace/Web Lock behavior continues unchanged.
- E01 should introduce **no runtime schema, catalog, item, fixture, machine, UI, HUD or flight change**. Tests below are acceptance requirements for implementation tasks, not claimed E01 results.

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

**Approval request:** accept/modify the candidate fuel yields, power/capacity/initial charge, whole-gram conversion strategy, recipe product + *usable* effect, job duration/energy, tray and active-job semantics, and pre-v14 migration charge. Explicitly settle the two TBDs (field marker effect / old free marks; migration starting energy) and the noted conversion ordering/cancel policy before downstream task coding. Review design against [active resource contract](resource-inventory-and-gathering.md), [browser status](current-client-status.md), [save continuity](save-contract.md) and [flight contract](browser-launch-landing.md).

#873 is the E01 design gate for E02–E07; until owner approval, downstream issues are **BLOCKED by #873**. Existing #783–#788 and #838–#845 / #847–#854 are prior foundations, not evidence of delivered Core/Fabricator behavior.
