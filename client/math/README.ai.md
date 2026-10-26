# Math AI maintenance

Read [architecture](architecture.md) and [invariants](invariants.md), then the
root [conventions](../../docs/coding-conventions.md) and
[validation matrix](../../docs/validation.md). The
[shared math decision](../../docs/technical-architecture.md#shared-math-decision-115)
records the inventory and deliberately limited adoption.

Keep this crate dependency-free and portable. It owns component arithmetic,
not frames, coordinate types, unit validation, normalization policy or gameplay.
Do not broaden it just to eliminate a small expression. Cross-crate changes
require workspace format, Clippy, test and build gates; record task evidence
under root `reports/`.


The six arithmetic primitives carry `#[inline]` so contact loops in dependent
crates can optimize their existing expressions across the crate boundary (#592).
Operation order, IEEE behavior, f64 types and APIs are unchanged; no fast-math,
reassociation, FMA substitution or physics policy is introduced. Matched archived
solver counters and terminal states accompany workspace checks in the task report.
