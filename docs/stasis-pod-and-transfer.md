# Stasis Pod, logout, transfer and permanent-death contract — future 2D multiplayer design

**Status (2026-11-06):** Owner-approved gameplay direction, **not implemented** in the local browser prototype or a multiplayer backend. This is the authoritative future contract for pod-based reconnect, relocation, and pod-loss death. It **supersedes** exact-last-position reconnect and the blanket rule that the destruction of any ship containing an offline passenger causes death. Keep cockpit presentation, physical boarding, frame-safe simulation, and server-authoritative rules from [cockpit/reconnect](cockpit-navigation-and-reconnect.md), [backend architecture](backend-architecture.md), and [backend persistence](backend-persistence-recovery.md). This does **not** introduce a rescue-only session, rescue deadline, or offline reassignment of an already dead character.

## 1. Purpose and identities

- The module is named **Stasis Pod** (not an ordinary cosmetic bunk or the general sleep-survival meter). It is a buildable, physically placed module with a stable module instance ID and an associated ship-local footprint.
- **One pod can be assigned to at most one living character/run; one character has at most one assigned pod.** Binding follows character/run ID, not account ID. Account identity persists across deaths, but neither the pod nor the character ID transfers to the next run.
- Every newly created run starts **outside a landed Scout on the canonical unknown alien starting planet** and is provisioned with one functional, initially assigned Stasis Pod inside that character's own Scout. Do not change the narrative opening to waking inside the pod.
- A character/run permanently owns one ship; moving their pod assignment does **not** move, transfer or unassign ownership of that ship, grant another character pilot control, or create a second universe.
- An intact ship belonging to a deceased character **remains in the shared physical world**, with historical ownership immutable. It may still contain working modules, resources, other characters' pods, and salvageable components. No automatic despawn or reassignment of pilot authority.

## 2. Logout and next-login rules

| Situation | Authoritative result |
| --- | --- |
| Player interacts with **their assigned functioning pod** to log out | **Safe logout:** commit safe exit; existing personal inventory, equipment, equipped suit and installed suit modules remain with character. Next login at that pod, if it survives. |
| Player closes game/logs out/loses connection away from a successful safe pod interaction | **Unsafe disconnect:** next login at active pod, provided it survives; ordinary carried resources, count-items, equipment, equipped suit and installed modules are lost from the character as **recoverable world loot at departure location**. Protected primary items excepted. |
| Player reconnects after either type of completed logout | Spawn in overhead 2D at their currently bound, functioning pod. Do not resume a chair, module UI, mining interaction or last physical position. |
| Active assigned pod is destroyed or deliberately dismantled while character is **offline** | That run has a **terminal death** outcome; on next login show the one-time outcome and start a fresh character/run with a new Scout and starting pod. **No rescue window, rescue-only session or posthumous replacement.** |
| Active assigned pod is lost while character is **online** | Character remains alive while currently playing; pod binding becomes invalid. They may assign another valid pod while alive. If they end the session without doing so, next login starts a new run. |
| Ship containing player's active pod is wholly destroyed | The bound pod is lost; apply the same online/offline pod-loss rule. Destruction of a ship that contains a player **but not their active pod** is not, by itself, an automatic pod-loss death. Independently lethal in-world events still kill normally. |
| Pod temporarily loses power or is disabled without being physically destroyed/dismantled | **TBD:** functional admission, repair/recovery and safe-logout behavior. Do not silently classify a temporary power outage as permanent death or invent a fallback spawn. |

Successful safe logout is a **distinct, server-confirmed pod interaction**, not merely standing beside the pod or pressing ordinary Logout. Once committed, disconnect removes the avatar/collider, releases chair/controls/interaction locks and never simulates an offline body. A failed/uncertain pod logout must not be acknowledged as safe.

Unsafe-loss items are **not silently deleted**: persist a recoverable loot cache/source with exact instance IDs and integer grams at the character's last validated departure position and reference frame. A ship-local cache remains a physical object associated with that ship until normal world events affect it; if its support ship is destroyed, a verified wreck/salvage conversion must resolve its fate. **TBD** loot container geometry, ownership/PvP, lifetime, decay, difficult locations, network timeout/grace and last-safe-position rules. No inventory mass creation/duplication. Cargo already stored in separate ship/module inventories is not player-carried equipment and is unaffected.

**Primary item exception:** An item explicitly and authoritatively designated as unique/unobtainable again may be protected from **unsafe logout and Stasis Transfer loss**; this is a reserved future classification, **not evidence any such items currently exist**. **TBD** catalog eligibility, whether those items survive *permadeath*, protected storage and equipped-primary edge cases. Default all normal items/gear/suits to droppable.

## 3. One-hour code and approved Stasis Transfer

1. A **living, online character** generates a cryptographically random, one-time Stasis Transfer code valid for **one hour**. The backend stores a verifier/hash, character/run identity, issuance/expiry and consumed/revoked state, not plaintext in logs. Revocation, brute-force throttling and secure presentation are required.
2. They share the code with a friend out of band. The friend physically interacts with a **functional, available, unassigned** Stasis Pod in their ship and enters it.
3. Server validates code and destination identity/capacity/structure and sends the code owner an **explicit approval prompt** identifying the target pod/ship and warning: **all ordinary inventory and gear, including equipped suit and its modules, are dropped at the departure location**. Code entry alone never relocates anyone.
4. If rejected or validations fail, keep the original pod and equipment untouched. If approved, block conflicting movement/critical interactions; show a **loading/transfer screen** and prepare the destination at a server-validated safe ship-local spawn (no worn suit is guaranteed).
5. Durably commit the transition **exactly once**: move all non-primary carried inventory and equipped item instances into recoverable loot at original departure anchor, update the one active pod binding (unassign prior pod, assign new pod), update character's authoritative frame/location, consume code and write a receipt/event.
6. After durable acknowledgment and region readiness, admit the character at the newly assigned pod in 2D with ordinary gear/suit removed; keep any protected primary items according to eventual item definitions. A new binding changes future reconnect location. Original ship identity, stored cargo and owner-only navigation rights remain unchanged.

Transfer is a deliberate **relocation**, possible without physically returning to the old pod. It is **not** a command that can be used to resurrect a dead run or rescue an already-dead offline character. Friend must build a **new available pod for a newly created character/run** to invite that character after permadeath. A code is bound to the issuing character/run, not to a permanent account tag.

**TBD:** exact code format/length, one-code-at-a-time/reissuance policy, pending approval timeout relative to the one-hour expiry, combat/critical-event transfer interdiction, recipient UI and progress states, target atmospheric/power/life-support suitability for a suitless character, disconnect mid-approval and capacity/structural changes during pending transfer. Do not implement transfer as an unconditional combat-escape teleport.

## 4. Physical module construction, destruction and dismantling

- Stasis Pods follow the approved [modular Scout construction](scout-and-modular-construction.md) geometry, funding, support/clearance, module ID and commissioning contracts. A sleeping nook/bunk is not implicitly a working pod.
- Ship salvage is **part by part**, reversing eligible construction: fixtures, modules, walls, rooms/structural cells, and eventually remaining structure. There is **no one-click whole-hull dismantle**. A completed multi-cell room can be dismantled in valid component operations while conserving its original part identities and material ledger.
- Deliberate removal must pass graph connectivity, load/support, occupancy, access and installed-module dependency checks. Normal dismantling cannot strand remaining ship sections; the **Core is a protected root** whose ultimate terminal decommission/recovery is **TBD**, not an ordinary removable wall/module.
- Dismantling an **assigned Stasis Pod** is a critical operation with consequences for its bound character: if offline, commit terminal pod-loss death; if online, invalidate binding without immediate death and require reassignment before the next login. Warn the dismantling actor; do not automatically prevent removal merely because a player is assigned.
- Removing a supporting structural part that would destroy or disconnect a pod must be resolved consistently with its pod-loss effects. Do not retain a functioning pod on nonexistent support or silently orphan its spawn anchor.
- **Scope distinction:** per-part *deliberate construction/dismantling* is an approved future mechanic; per-part *combat damage/detached hull physics* remains deferred in the current 2D ship-wide damage model. Older native Rust behavior is historical evidence, not shipped React functionality.
- **TBD:** exact salvage yields, timer/tools, Core terminal recovery, passenger/connected-avatar clearance during structural removal, effects on autonomous machinery, modules/cargo in removed sections, and live pod damage/disablement. Use exact recovered grams and item identities; no free new mass or auto-inventory overflow.

## 5. Backend domain proposal and critical transactions

**Design direction, not final Mongoose schema:**

```ts
type CharacterState = 'alive' | 'dead';
type PodState = 'functional' | 'disabled' | 'destroyed' | 'dismantled';

interface PodRecord {
  podId: string; shipId: string; moduleInstanceId: string;
  state: PodState; revision: number;
}
interface SpawnBinding {
  characterId: string; podId: string | null; revision: number;
  // Unbound + online: alive until current session ends; never a rescue-only state.
}
interface TransferCode {
  codeId: string; issuingCharacterId: string; verifierHash: string;
  expiresAt: string; status: 'issued' | 'pending_approval' | 'used' | 'revoked' | 'expired';
  proposedPodId?: string;
}
interface LootCache {
  lootId: string; sourceCharacterId: string; reason: 'unsafe_logout' | 'stasis_transfer';
  frame: unknown; // versioned planet-fixed, ship-local or validated physical anchor
  resourceGrams: Record<string, number>;
  itemInstances: string[]; // includes suit + installed module identity graph
  revision: number;
}
```

- **Separate inventory documents** are confirmed. Each holder/compartment (character, installed storage module, machine buffer) owns an independent inventory; enforce weight/slot/stack rules and type restrictions. Never duplicate equipment that is both equipped and packed.
- Unique constraints: one character-run active binding, at most one active character per pod, immutable pod/module IDs, unique code verifier and single-use operation receipt, unique character/run identity, immutable ship owner identity. Implement races with conditional revision writes, indexes and MongoDB transactions.
- **Safe logout:** validate assigned functional pod + character session and durable exit; release in-memory interaction state. It must not accidentally enter the unsafe-loot path.
- **Unsafe logout:** atomically move carried ordinary items/gear into a recoverable loot record, protect classified primary item instances, save departure frame, and record idempotent loss outcome. World cache must be reachable by a validated persistent frame; if it cannot be safely committed, fail closed rather than delete gear.
- **Pod loss:** atomically mark module terminal, revoke associated active binding, and durably record offline terminal death with unique cause/event. For bulk whole-ship destruction, use safe idempotent death derivation before account admission when a single bounded transaction cannot cover all affected pods. Do not kill a character merely for being offline **aboard a different ship**; compare actual active binding.
- **Transfer acceptance:** in one logically atomic critical transaction, validate authorizing online character/run, code expiry, available destination pod + rev, old binding rev, inventory/equipment rev, departure-frame evidence and safe destination, then commit loss cache + inventory debit + binding switch + new frame + code consumption + receipt/event. Region readiness and any environmental safety condition must be established before committing; if unknown, fail closed.
- **Run restart:** a dead character and their historical ship/pods retain immutable provenance; create fresh character/run, new ship, new personal inventory and assigned starter pod exactly once. A new run must never inherit the dead run's code or pod binding.
- **Recoverability:** replay canonical durable events/records over older physical checkpoints (target verified every **5 s**, suspend when verified age reaches **30 s**); no old checkpoint can resurrect a dead character, consumed code, dismantled pod, removed structural part or duplicated loot.

## 6. Acceptance scenarios

| Scenario | Required result |
| --- | --- |
| Safe logout via assigned pod | Inventory, suit and gear preserved; next login at pod; no seated avatar |
| Ordinary logout outside pod | Exact ordinary carried contents + suit/modules become one world loot cache; protected primary kept; next login at bound pod |
| Brief network fault, ambiguous logout | **TBD grace policy**; never double drop or claim safe logout without commit |
| Live transfer with valid one-hour code and approval | Old binding released, new pod assigned, suit + inventory dropped once, new ship-local spawn |
| Expired/replayed/wrong-character code | No transfer, no loss, no binding change |
| Two recipients race for same pod | At most one commits; no double occupancy |
| Pod destroyed while its character offline | One terminal death; next login starts a new run, never rescue screen |
| Pod dismantled while character online | Character stays alive in current session; must bind another pod before a future login |
| Friend creates new pod after old character death | Old character stays dead; fresh run may accept a **new** code and new pod |
| Deceased owner's intact ship | Remains physical/salvageable; no inherited pilot rights |
| Dismantle one wall/module | Only validated part changes, graph/occupants checked, exact salvage output; no instant ship deletion |
| Crash after transfer commit before next checkpoint | New binding, loot, character frame and consumed code persist; old state cannot respawn |

## 7. Explicitly deferred questions — do not implement by assumption

1. **Timeout/grace and intentional vs accidental logout:** how long to allow session recovery before irreversible unsafe-loot commit, and what happens in combat.
2. **Pod environment:** power requirements, disablement/repair vs terminal destruction, life support, oxygen and suitless arrival validation.
3. **Code UX/security:** exact characters/entropy, issuance count, code renewal, approval timeout, request visibility, rate limits and loading cancellation.
4. **Primary items:** catalog policy, storage, whether equipped unique items may be protected, and whether unique items survive true permadeath.
5. **Loot mechanics:** cache visual, precise safe anchor if in space/ship, salvage after ship destruction, ownership/theft, decay and cleanup.
6. **Damaged or dismantled pod while online:** exact notification, time to rebind during a session, logout behavior and server crash vs intentional disconnect.
7. **Safe pod interaction UX:** whether an animation/timer is required; no unapproved instant invulnerability or remote logout.
8. **New run multiplayer starting placement:** avoid overlapping Scouts; preserve canonical universe and narrative alien-planet opening.
9. **Modular removal details:** yields, tooling/timing, structural/occupancy support, Core-root terminal handling, cargo and module job dependency resolution.
10. **Atomic cross-region moves:** transfer during destination loading/handoff and validated world/ship frame availability; not an excuse to run multiple unfenced simulation owners.

These questions are **open** even though the high-level rules above are decided. This document is not authorization to implement guessed values.
