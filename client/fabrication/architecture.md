# Fabrication architecture

`salimon-fabrication` is an independent portable domain depending only on
`salimon-world` raw resource contracts and std. It does not call the ship,
runtime, renderer or world session. Canonical behavior is the
[accepted Fabricator specification](../../docs/spaceship-building-and-survival.md#accepted-fabricator-contracts-2026-10-19-336).

| Source | Ownership |
| --- | --- |
| `mass.rs` | Positive integer milligrams, one-time kg quantization |
| `recipes.rs` | Stable IDs, validated immutable recipe book, proposed starter tuning and unlock/category schema |
| `machine.rs` | Typed commands/results, bounded storage, atomic committed costs, stable jobs, FIFO stepping, single tray, observations and mass contribution |
| `lib.rs` | Reexports and typed errors |
| `tests/fabricator.rs` | Public transactions, edge arithmetic, power/tray/time boundaries and deliberate mass transitions |

`RecipeBook::new` accepts data records and validates the complete set before
publishing it. Callers may build records from a later data adapter; there is no
file format/parser in this crate. Both starter recipes are unlocked; advanced
requirements can be represented but cannot be acquired here. Jobs capture recipe
values so balances/progress never depend on later tuning. There are no setters
that mutate a validated recipe book inside a running machine.

`Fabricator::execute` dispatches the same public insert/enqueue/cancel/collect
methods. Bins use world catalog identities in stable catalog order. All balances
and recipe costs are integer milligrams; world positive finite kg round once to
the nearest mg (half upward). Sub-half-mg and overflowing values are rejected,
preserving the original fragment. Accepted insertions consume the entire original
fragment, including zero accepted mass at a full bin. `Insertion` reports accepted
and discarded mg; runtime must remove the source exactly once. There is no raw
withdrawal operation. This crate cannot itself despawn physical entities.

`enqueue` checks positive quantity, technical job capacity, recipe/unlock, job-ID
space, multiplication and every balance before mutation. It then debits the full
batch and captures the job. No reserved material remains in raw storage and no
cancellation can credit it back. Capacity defaults to 64 queued jobs (including
active head), configurable at construction; quantity is not counted as extra jobs.
This is an implementation bound, not an approved gameplay upgrade/capacity.

`power_demand` exposes current watts and duration until output. `step(delta,
powered)` advances only the funded head and returns actual worked time/joules and
unused elapsed time. Runtime must split at Core exhaustion and settle every
returned worked interval; `powered` is a caller funding assertion, not an energy
storage implementation. Partial funding uses a funded substep plus an unpowered
substep. The machine cannot debit Core itself. Power demand may be nonzero in a
NoPower snapshot; it is requested, not actual, consumption. Idle and occupied tray
request zero watts. A step stops on its first output; blocked/unpowered/unused
time is never banked. Duration arithmetic is exact and bounded without a dt loop.

A tray item has stable `(JobId, unit)` identity. Collection removes it once; the
runtime must first validate the receiving slot. A final output may outlive its
completed job; cancel only applies to pending work and never deletes tray output.
The separate integer storage/tray mass components avoid an overflowing combined
u64 and exclude jobs/committed inputs/collected inventory deliberately. Runtime
maps these to cached ship ledger entries alongside world fragment removal.

All state is in-memory. Cloning supports portable tests, not disk persistence.
Snapshots are owned observations, not a restorable save schema. Future persistence
must version material/recipe/job IDs, captured job parameters and state and
validate restoration atomically, as documented in the canonical specification.
Runtime/physical/power/UI adapters are separately tracked by #339–#347; native
reference signoff is #348. This crate alone does not make crafting playable.
