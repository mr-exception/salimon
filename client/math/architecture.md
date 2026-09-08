# Math architecture

`salimon-math` is a leaf library with six functions on `[f64; 3]`: `add`, `sub`,
`scale`, `dot`, `cross`, and `length`. Character, ship and runtime fragment
simulation import compatible primitives; runtime carrying imports `cross`.
There are no external or domain dependencies, generic scalar types, vector
wrappers, conversions or reexports of domain types.

Callers own units, frames, validation and normalization policy. `WorldPosition`
and universe-coordinate semantics remain in world. Renderer retains its
`f32`, scaled-length and finite-input preparation math. Quaternion composition,
interpolation and axes remain in ship; resource quaternion validation stays in
world. See the canonical
[decision and inventory](../../docs/technical-architecture.md#shared-math-decision-115).


The six arithmetic primitives carry `#[inline]` so contact loops in dependent
crates can optimize their existing expressions across the crate boundary (#592).
Operation order, IEEE behavior, f64 types and APIs are unchanged; no fast-math,
reassociation, FMA substitution or physics policy is introduced. Matched archived
solver counters and terminal states accompany workspace checks in the task report.
