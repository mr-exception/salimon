# #313 — current-sphere native verification

Work in progress on reference Apple M1, 2026-10-20. Native current-cap and
current-mixed layouts now expose the existing 500/1,000 CPU populations through
one shared production-extraction initializer. Legacy layouts are preserved.
Raw benchmark accounting includes actual contact radius with identity/material/mass.

The new 1,000-object population/mass/radius/containment regression passes.
Full Rust/tooling checks and native measurements are in progress. No hardware
performance, native visual or complete mixed-residual acceptance is claimed here.

An original `current_sphere_workloads_cpu` run at 4d8faf8 logged completion of
500 cap, 500 mixed and 1,000 cap cases, then was terminated after 903 wall seconds
while executing the 1,000 mixed case. It did not produce a complete report.
That is incomplete evidence, not a passed matrix. A retained macOS sample places
work in the physics pair-contact path. Final report will retain all results.
