# Issue #416: Dominant-source celestial gravity

The `gravity` module evaluates known positive gravitational parameters using split galactic subtraction, ranks a bounded complete candidate set deterministically and selects four contributors by default. Existing sources receive an explicit 5 percent default hysteresis; membership transitions use a one-second default smoothstep blend with at most twice the configured contributors during handoff. These are documented numerical policies, not approved scientific accuracy budgets. No new handoff starts until the previous one finishes; callers must retain old sources through transition. Missing active sources, duplicates, nonfinite input and body interiors reject without committing selector state. An all-source comparator quantifies truncation error. Frame-tree adapters evaluate moving body centers at the requested epoch. Candidate spatial indexing and actual runtime adoption remain downstream; this version scans at most 65536 supplied candidates, so full-catalog performance is unverified.

Validation results are recorded below. Linux x86_64, Rust 1.99.1; private fixtures only. No runtime/rendered change or hardware claim. Production source and full-snapshot approval remains #659, scientific/resource coverage #524/#526; those checks are unrun. This PR is a development implementation, not full production qualification.

- `cargo test -p salimon-universe --locked`: 83 passed, zero failures.
- Inverse-square/reference comparisons, stable ties/order, source-selection hysteresis/continuous handoff, singularity/missing/duplicate-source failures pass.
- `cargo clippy -p salimon-universe --all-targets --locked -- -D warnings`, formatting and docs/base-impact checks pass.
- Added the existing workspace math crate and deliberately updated Cargo.lock. Full-catalog spatial candidate indexing and measured performance budgets remain incomplete; current caller supplies the bounded complete candidate list.
