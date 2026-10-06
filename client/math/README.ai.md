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
