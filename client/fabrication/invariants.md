# Fabrication invariants

1. Each world material bin is independently within 0..=20,000,000 mg (20 kg).
   Entire accepted fragments are consumed by the adapter; excess is discarded.
2. Recipe mass is positive integer mg, time is positive Duration, watts and
   per-operation joules are positive finite; ingredients and recipe IDs are unique.
3. Every rejected command preserves all authoritative state, including next job ID.
4. Enqueue deducts every unit's ingredients together. No refunds or withdrawal.
5. Job identities increase monotonically and are never reused, including after
   cancellation/completion. Technical capacity includes the active head job.
6. Only the FIFO head progresses. No waiting job accumulates time. Batches remain
   contiguous; an occupied tray blocks even another unit of the same batch.
7. Blackout, blocked tray and idle steps consume zero joules and advance no work.
   The caller funds actual worked time; state never infers energy from a clock.
8. Completion produces exactly one item and stops the step. Excess elapsed time is
   returned and never banked. Snapshots, collection and cancellation do not step.
9. Cancelling active/waiting work preserves survivor order, stored balances and
   completed tray output. A partial active unit is lost without refund.
10. Collection removes one item exactly once. External inventory validation and
    physical fragment deletion belong to runtime, not this domain.
11. Ship contribution includes only accepted raw balances and tray output; reserved
    inputs, partial progress, queue and collected consumable inventory contribute
    zero. This deliberate mass exception is covered at every domain transition.
12. Definitions/state contain no platform/GPU/clock/disk/backend or health effects.
    Unlock and future category records do not implement progression/upgrades.
13. Consumable receiving slots are exactly three food and two medical, one item
    each; matching first-vacant placement is atomic and rejects unsupported
    categories, duplicate resident identities and full categories without mutation.
14. Consumption removes only the first occupied slot in the requested typed
    consumable category. It does not shift survivors, affect the other category,
    apply vitals, change ship mass or depend on a future full-health/hunger state.

## Whole-session saves (#365)

`FabricationSaveState` validates recipe definitions, ordered complete bins, queue
IDs/captured recipes/committed quantities/progress, allocator high-water marks,
tray and exact inventory slots together. Cross-owner item identities cannot be
both collected and on the tray. Runtime's `SessionState` is the concrete payload
inside `SaveGame`; `save_bytes` and transactional `restore_bytes` integrate all
physical state, equipment and exact elapsed/backlog clocks. Stored mass and
recipe watts are reconstructed without charging or refunding. UI/input/focus,
feedback and render caches reset; interactions are recomputed from restored
world state. The native host's automation policy stays host-owned. No disk IO,
autosave scheduling or offline simulation is implied; #366 owns durable storage
and shell lifecycle integration. See [canonical save contract](../../docs/save-contract.md).

- Explicit `consume_slot` (#370/#377) removes only that named slot; survivors never
  compact and no first-available fallback occurs. Runtime checks the selected
  stable job/unit before committing validated survival benefits. The historical
  invariant 14 describes only the retained `consume_first` compatibility helper.

- #379's new-run constructor allocates initial output identities from the same
  high-water mark as future jobs and copies exact starter recipe ingredients.
  No job/tray work, fake fragment, extra inventory, unlock acquisition or capacity
  bypass is introduced; restoration never replenishes supplies.

- Advanced foods are locked validated definitions in the starter recipe book.
  Only basic food is initially unlocked; medkit availability is unchanged. No
  gameplay unlock acquisition or inventory capacity increase is implied (#378).
