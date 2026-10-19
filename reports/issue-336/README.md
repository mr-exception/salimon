# Issue #336 — Fabricator contracts

Documented all five approved Q&A rounds in the existing ship/survival specification,
including typed command/snapshot design, ownership, transaction sequences,
simplified mass accounting, persistence design and explicit deferred capabilities.
Updated current scope and navigation; superseded old example consumable capacities.
Proposed starter recipe values and technical bounds are explicitly not owner tuning.

Validation: reviewed requirements against #336–#338 and the approved Q&A;
`git diff --check` passed. Relative documentation links verified locally.
No Rust behavior changes; Rust/native tests not run for this documentation PR.
Implementation tests belong to #337/#338 and subsequent integration issues;
this change does not claim playable crafting or native visual signoff.
