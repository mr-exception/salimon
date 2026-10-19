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
