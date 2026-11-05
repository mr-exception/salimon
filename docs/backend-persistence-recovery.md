# Backend persistence, strict checkpoint window and recovery

**Status:** Approved future backend design, **not implemented** (2026-11-05). Companion to [backend authority, simulation and networking](backend-architecture.md). This describes NestJS/TypeScript + MongoDB server persistence, **not** the browser's current `ResourceSession` localStorage v3, nor the archived Rust/native whole-session save schema. Read [current client status](current-client-status.md), [resource/inventory](resource-inventory-and-gathering.md), and [cockpit/reconnect](cockpit-navigation-and-reconnect.md) for current implementation and gameplay precedence.

## 1. Explicit decisions and invariants

1. **Recovery policy A: checkpoint recovery.** On unexpected server failure, a bounded interval of uncommitted *continuous* movement/physical motion may be lost. No successfully **confirmed critical outcome** may disappear or be applied twice.
2. **Strict checkpoint-staleness policy (the selected "section 9" recommended A).** Target one verified checkpoint every **5 seconds** initially. If the newest verified checkpoint becomes older than a configured maximum age, **suspend affected gameplay admission/simulation** until a new checkpoint is verified. Five seconds is a target cadence, **not a guaranteed rollback bound**. Exact overdue threshold and safe suspend granularity are deferred tunables, to be measured and documented before shipping.
3. **Durable before confirmation.** Raw-resource debits/credits, item transfers, ship-wide destruction/death, ownership changes and accepted irreversible machine jobs are critical and require an acknowledged durable commit before final presentation to clients.
4. **Conservation and identity.** Each transfer changes the authoritative source and destination together, using integer grams and per-item count-stack rules, validated capacity, stable entity IDs, revision and one unique operation receipt. No item duplication, fractional counts, resource creation or partial credit on error.
5. **No resurrection.** Ship destruction and death are terminal durable facts. A restored physical checkpoint must never revive a ship, player or expended source that a newer committed event invalidated.
6. **Never invent safe progress.** Offline coasting, collision detection, power/machine processing and published canonical planet/catalog versions must be validated; missing source/epoch/geometry rejects resume or suspends affected entities. No blind post-crash movement extrapolation through possible collisions.
7. **One writer per logical region.** Initially a single NestJS process owns the world. Generation-based fencing must prevent a stale owner from committing snapshots or terminal writes if another process is taking over.

These are correctness requirements, not statements that MongoDB guarantees all of them automatically; the application must implement and test reconciliation, fencing and transaction discipline.

## 2. State classes and persistency boundaries

| State class | Examples | Data owner / behavior |
| --- | --- | --- |
| Ephemeral | WebSocket, open UI, key state, predicted movement, pending menu, camera, chair lease/session lock | Live process only; clear on disconnect/recovery |
| Continuous checkpoint | Authoritative player/ship physical pose, velocity, orientation, tick/epoch, noncritical simulation state | Fixed-step in-memory state + periodic validated durable snapshots |
| Durable current state | Accounts/characters, ownership, inventory, remaining source grams, registered stores, processing commitments, terminal ship/death status | MongoDB authoritative documents and revisions |
| Durable operation receipt | Accepted source-to-inventory transfer, crafting reservation, death/destruction or ownership change | Stable `operationId` with deterministic result, affected IDs, revisions and causal tick |
| Canonical immutable source | Published catalog/body/planet definition ID, version and digest | Pinned external/versioned source contract; checkpoint/region only references it |

An acknowledged ordinary walk or ship-frame advance is **not** an acknowledged durable checkpoint; clients need to understand that a server failure may reconcile continuous position backward. An orderly disconnect explicitly saves a safe frame-relative anchor durably before confirming clean exit.

### Initial suggested MongoDB collections (proposal)

```text
accounts
characters
ships
inventories
resource_sources
world_deltas
simulation_checkpoints
checkpoint_heads
gameplay_events
processed_operations
presence_records
simulation_metadata
```

Domain records are not required to be one collection per NestJS module. Prefer atomic document boundaries where cohesion makes sense, and MongoDB transactions for cross-document critical commits. Index (1) account and character ownership, (2) ship and anchored offline passenger membership, (3) region+checkpoint generation, (4) region+event sequence, and (5) globally or scope-uniquely identified operations. Avoid a giant unbounded world document. Exact schemas/indexes are an implementation deliverable.

## 3. Periodic checkpoint publication

**Initial target cadence:** once every five seconds of active authoritative simulation time, with suitable wall-clock staleness monitoring. The time of the last successfully persisted checkpoint, not the last attempt, determines staleness.

A checkpoint envelope includes:

```ts
type CheckpointHeader = {
  schemaVersion: number;
  regionId: string;               // One global region in phase one is valid
  generation: string;             // Losslessly encoded monotone identifier
  ownerFence: string;             // Current simulation owner generation
  simulationTick: string;
  universeEpoch: string;          // Canonical game epoch; not a random client timestamp
  catalogId: string;
  catalogVersion: string;
  catalogDigest: string;
  durableWatermark: string;       // Last canonical committed operation in this snapshot
  stateDigest: string;            // Canonically computed integrity hash
};
```

The full document also contains complete validated physical/scheduler state needed to reconstruct at that tick, with stable IDs and reference-frame information. Astronomical position **split cells** and any integer outside JSON safe-number bounds must be losslessly encoded (for instance decimal strings); no flat `Number` galactic coordinates. Derived presentation caches, WebSocket sessions, held inputs, authoring sprites and browser-only state are excluded.

**Safe writer protocol:**

1. Capture an immutable, internally consistent tick/region snapshot and its associated durable operation watermark; do not serialize a partially advanced tick.
2. Validate finite/physical values, unique stable identities, declared catalog/source digests, coordinate frames and state integrity. Serialize a candidate with schema/owner fence and digest.
3. Write the candidate durably without modifying the previously active checkpoint generation.
4. Confirm candidate was acknowledged and is readable/validated before atomically compare-and-setting `checkpoint_heads` to the new generation, using current owner fencing. Head publication itself must have the required write concern.
5. Retain at least two **previously verified** generations for fallback; prune only after a newer head and its required durable-history coverage are safe.
6. Any failed/unknown publication is **not** a valid freshness update; log it and keep existing verified head. A process crash mid-write cannot replace a valid head.

A snapshot's `durableWatermark` is a **real committed-sequence barrier**, never inferred from elapsed simulation ticks or client acknowledgments. The snapshot must agree with the durable entity revisions visible at capture, or recovery must have sufficient history to reconcile their divergence.

## 4. Critical operations: commit and idempotency

Example: deposit/source -> player's 10 kg / 12-slot inventory. Approved browser domain uses exact integer grams and validates mass, item stack limits, per-source depletion and revision. Backend logic must preserve those rules while making shared ownership authoritative.

1. Receive a client-authenticated command with a stable **operation ID** distinct from movement input sequence and a claimed expected revision; the server looks up canonical state and checks reach/tool/authority/capacity.
2. The authoritative simulation produces a validated transfer proposal at a definite server tick; conflicting entity modifications wait until this operation finishes.
3. Within a MongoDB transaction, condition on current entity revisions, debit the exact source amount, credit precisely the same quantity into the inventory, update both revisions, append a durable event and insert the unique operation receipt.
4. Use supported transaction retry/unknown-commit-result handling. If the acknowledgment is ambiguous, resolve via the operation receipt; **never blindly re-execute** an unknown-commit operation.
5. Confirm and broadcast only when a successful majority-acknowledged journaled commit is established (appropriate replica-set transaction write concern, along with transaction-level read concern/isolation requirements). A failed transaction cannot leak a provisional credit to gameplay clients.
6. If two miners contend, only one can commit against the matching source revision at a time; re-evaluate against the now-current source quantity instead of overspending it. The final depleted resource remains a stable known entity.

`processed_operations` records scoped unique ID, operation kind, initiator, affected stable IDs, causal tick, result, revisions, server owner fence and committed sequence. Its retry retention must cover the supported reconnection/retry window; for critical outcomes that must remain auditably immutable, preserve adequate durable event/history independently of receipt cleanup.

**Event versus snapshot:** the transaction updates *canonical materialized critical state* and commits a compact durable event/receipt together. The event is for causality, recovery and outbound retries; do not interpret an uncommitted in-memory event as permanent. No requirement to event-source every ordinary 60 Hz movement tick.

### Ship-wide destruction and offline death

A terminal collision is a critical domain outcome; it cannot be exposed as final until the ship's terminal status/event is durable. Record the ship ID, precise causal time/tick, involved state/anchor needed for reconciliation, and an irreversible terminal revision. Any offline characters anchored aboard that ship must be marked dead by the same transaction when feasible, or by a reliable idempotent **terminal-death derivation** before they may reconnect. The server **must check the ship's terminal status first** so a crash between ship destruction and individual death materialization cannot spawn passengers in a destroyed vessel. The death notification is delivered once or retried idempotently; fresh run follows approved permadeath rules. A character anchored on a planet is not killed by destruction of a distant ship. Nonfatal ship damage is not terminal.

## 5. Reconciling newer committed state with an older physical checkpoint

**Do not overwrite current durable documents with stale copies embedded in a checkpoint.** On crash, critical records may be newer than the last verified physical snapshot.

Example:

| Server time | Committed state |
| --- | --- |
| 12:00:00 | Physical checkpoint at tick T, source 3,000 g |
| 12:00:02 | Mining commit: source -2,000 g, inventory +2,000 g (durable) |
| 12:00:03 | Player advances 5 m (continuous, not yet checkpointed) |
| 12:00:04 | Server crashes |
| Recovery | Restore tick T physical pose if necessary, but **source must remain 1,000 g and player keeps +2,000 g** |

The recovery coordinator starts with the checkpointed continuous physical state but overlays/reconciles the **latest canonical critical documents**, using immutable committed operation ordering, entity revisions and durable watermarks. Any terminal event that invalidates an old checkpoint's entity takes precedence; remove or replace the stale physical object as required. For nonterminal critical interactions whose physical preconditions depend on uncheckpointed movement, the durable event must contain a validated causal frame/tick or effect anchor so recovery can safely preserve the outcome without spoofing reach or duplicating an action.

Reconstruct masses from authoritative inventories and attached objects once, never double-count payload aboard a ship. Stage reconciliation privately and validate all cross-entity invariants before publishing anything. If a checkpoint/event combination is insufficient to reconstruct safely, **fail closed** rather than create a plausible position or undo a committed outcome.

## 6. Crash recovery state machine

```text
STARTING
  -> ACQUIRE_OWNER_FENCE
  -> LOAD_VERIFIED_CHECKPOINT
  -> VALIDATE_CATALOG_AND_SCHEMA
  -> RECONCILE_DURABLE_OPERATIONS
  -> CLEAR_EPHEMERAL_CONTROLS_AND_LOCKS
  -> VALIDATED_OFFLINE_CATCH_UP
  -> VERIFY_AND_PUBLISH_NEW_CHECKPOINT
  -> READY_FOR_GAMEPLAY
Any invalid/unavailable stage -> RECOVERY_BLOCKED (no gameplay admission)
```

- **Owner fencing:** acquire the one-writer ownership generation with durable compare-and-set. All writes and head publication must reject an old generation even if a past owner returns from an unknown/slow operation.
- **Checkpoint:** pick latest *verified* head. Corrupt/incomplete/unknown schema or mismatched canonical digest must not become live. Try a previous verified generation only with sufficient durable history for safe reconciliation; otherwise block.
- **Critical reconciliation:** canonical records + events newer than checkpoint watermark, exactly once. Verify rev chains, resource conservation, terminal tombstones, region ownership and lossless position codecs.
- **Transient reset:** no lingering avatar for absent players, no held thrust/steering, mining or cockpit authority. Locks/chairs from dead sockets are cleared while independent queued production remains.
- **Universe catch-up:** advance passive ship motion, approved independent machines and physical environmental time according to verified authoritative source versions. Do not retrospectively grant offline active player input. If a potentially intersected collision/event cannot be resolved, block affected recovery instead of assuming empty space.
- **Publication:** privately reconstruct to a consistent current epoch, durably publish a new verified checkpoint and only then enable game commands. Observe whether a ship was destroyed while offline **before** admitting its passenger.
- **Failure:** return actionable error/health status and preserve existing durable state and previous checkpoint. No automatic reset to a new character or recreated source on recovery error.

Recovery after an **orderly deployment** first stops new command admission, drains/settles pending critical commits, cancels active inputs, writes disconnect anchors as appropriate, publishes a final verified checkpoint, then terminates. Abrupt process loss has no such final-snapshot guarantee.

## 7. Strict checkpoint freshness and storage failures

The selected policy is **strict**, not best-effort.

- Attempt checkpoints at the initial five-second cadence; record last *verified* persisted checkpoint age by region.
- Define an explicit configurable maximum checkpoint staleness before implementation. When age exceeds that bound, suspend further affected simulation and command admission **before additional unverifiable motion increases recovery risk**. Do not merely report a warning while continuing indefinitely.
- If a critical transaction is unavailable/ambiguous, withhold confirmation and keep conflicting entities in a safe pending/paused state. It is forbidden to say "resource credited" or "ship destroyed" and later revoke that confirmed result.
- If checkpoint publishing cannot make progress but critical writes still work, strict staleness still suspends affected gameplay when the freshness bound is crossed. Do not bypass this by disabling persistence.
- Resume only after a safe verified snapshot/freshness state is established and pending operations are reconciled. Clients receive explicit temporary-unavailable/synchronizing status rather than silently diverging.
- A single Node process makes a global pause a valid initial conservative fallback. More targeted per-region/per-entity suspension is a later performance design, **not** permission to let one affected dependency race ahead.
- A checkpoint interval of five seconds does not itself guarantee five-second rollback. Bound the interval between verified heads, define the stale threshold, and record this in metrics/acceptance.

Operational metrics: verified checkpoint age, write latency, failed attempts, blocked admissions, pending critical commits, reconciliation duration, universe catch-up backlog, owner fence, invariant failures, active region count and per-region tick lag.

## 8. Reconnect anchors and session departure

An ordinary *graceful* disconnect must commit the exact last **safe noninteraction anchor** (planet-fixed point and body ID, or ship-local point and ship ID), cancel current interaction/seat/held inputs and remove visible avatar, with one coherent durable presence update. If the player was using an object, save an adjacent reachable point rather than the occupied interaction target. This is not the same promise as an **unexpected server crash**, where continuous movement can revert to a checkpoint.

During a network failure, server detection delay and simultaneous reconnect/disconnect order need a documented timeout/grace policy; never allow two sessions to issue control for one character simultaneously. Reconnect checks terminal state before avatar creation, revalidates identity/catalog/frame and anchors, then restores into overhead 2D standing mode without previous interactions. Saved physical coordinates and reference-frame definitions must not be silently rewritten when the planet rotates or ship moves.

Unresolved exact fallback if saved point is obstructed, furniture moved, planet surface migrates or owner ship is transferred without destruction is deferred to the owning gameplay specifications. Reject unsafe overlap rather than improvising teleportation.

## 9. Data model examples (illustrative, not final Mongoose schemas)

```json
{
  "regionId": "planet:vesper:site:001",
  "generation": "42",
  "ownerFence": "7",
  "schemaVersion": 1,
  "simulationTick": "180000",
  "universeEpoch": "2026-11-05T12:00:00Z",
  "catalogId": "example-universe",
  "catalogVersion": "1",
  "catalogDigest": "sha256:...",
  "durableWatermark": "420",
  "state": { "players": [], "ships": [], "objects": [] },
  "stateDigest": "sha256:..."
}
```

The example names/IDs and sample timestamps are placeholders, not published game canon. Large values are strings to avoid precision loss. Use validated, versioned codecs rather than direct JSON serialization of runtime `bigint` values.

```json
{
  "operationId": "uuid-from-client-retry-scope",
  "kind": "resource.transfer",
  "actorId": "character:sample",
  "sourceId": "source:landing:mineral:01",
  "destinationId": "inventory:sample",
  "resourceId": "mineral:sample",
  "grams": 1000,
  "serverTick": "180020",
  "committedSequence": "421",
  "outcome": "committed"
}
```

The domain validation owns the actual transfer amount, not the caller-supplied request. Stable ledger revision, catalog identity and transaction receipt fields are mandatory in the eventual typed schema.

## 10. Acceptance and fault-injection matrix

| Scenario | Mandatory result |
| --- | --- |
| Two miners contend for last 1,000 g | Debit=credit once; no negative source, overfill or duplication |
| Commit succeeds but network ACK is lost | Same operation ID returns committed result; no replayed debit/credit |
| DB fails before durable critical commit | No canonical success or client-confirmed irreversible effect |
| Node process dies while candidate checkpoint writes | Previous verified head remains usable |
| Crash after 2 kg mining but before next checkpoint | Canonical inventory/source still reflect transaction; continuous location may revert |
| Crash immediately after whole-ship terminal commit | Ship stays destroyed, offline passenger can't spawn, terminal death resolved once |
| Checkpoint hash/catalog mismatch | No gameplay admission using invalid source/geometry |
| Post-checkpoint critical event invalidates old physical object | Reconciler cannot resurrect object/ship or double-count mass |
| Replica/DB temporarily unavailable | Safe pending/fail-closed behavior; critical ACK not fabricated |
| Verified checkpoint staleness exceeds configured bound | Affected gameplay stops until safe durable recovery |
| Two server owners attempt to publish | Stale owner fence rejects writes/head swaps |
| Graceful disconnect mid-mining/cockpit | Completed transaction preserved; ongoing interaction cancelled, chair released, safe durable spawn anchor |
| Reconnect after ship moves | Same ship-relative point, no phantom offline avatar |
| Offline passenger's ship destroyed | One durable terminal result and "You died while disconnected" before fresh run |
| Ship destroyed while player logged off on planet | No unrelated offline death |
| Repeated kill/restart during startup reconciliation | No partial published state, no duplicate events or items |

Test with a real MongoDB replica-set fixture, transaction fault injection, actual process termination/restart and two simultaneous browser clients. Unit-only mocks do not prove MongoDB transaction or crash behavior. Include schema migration rejection/rollback tests and production-build browser E2E evidence. Explicitly record checks not run and limitations.

## 11. Implementation decisions still open

- Maximum verified-checkpoint staleness threshold (distinct from proposed **5 s write cadence**) and whether initial safety pause is global or safely partitioned.
- Checkpoint storage chunking/size limits, compression, checksum canonicalization and garbage collection/history retention.
- How long to retain idempotency receipts versus durable event/history, and exact operation/region sequence composition.
- MongoDB topology, majority semantics under failover, transaction retry budget and backup/restore/RPO/RTO operational targets.
- Exact isolation of physics tick versus critical commit barriers and crash reconciliation for mixed physical/transaction events.
- Time source selection and clock health policy for server-wide epoch/offline catch-up; no guessing ephemeris coverage.
- Explicit network session timeout/grace and safe adjacent spawn fallback.
- Detailed multi-region ownership/lease handoff; phase one deliberately does not run >1 active owner.
