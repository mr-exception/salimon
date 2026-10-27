# Dominant-source celestial gravity (#416)

The `gravity` module evaluates known positive gravitational parameters using split galactic subtraction, ranks a bounded complete candidate set deterministically and selects four contributors by default. Existing sources receive an explicit 5 percent default hysteresis; membership transitions use a one-second default smoothstep blend with at most twice the configured contributors during handoff. These are documented numerical policies, not approved scientific accuracy budgets. No new handoff starts until the previous one finishes; callers must retain old sources through transition. Missing active sources, duplicates, nonfinite input and body interiors reject without committing selector state. An all-source comparator quantifies truncation error. Frame-tree adapters evaluate moving body centers at the requested epoch. Candidate spatial indexing and actual runtime adoption remain downstream; this version scans at most 65536 supplied candidates, so full-catalog performance is unverified.

This is portable private-development implementation, separate from playable runtime adoption. Production source approval remains #659 and full-data acceptance #524/#526. No hardware, scientific model accuracy or production completeness is inferred from fixture tests.

Run `cargo test -p salimon-universe --locked`, `cargo clippy -p salimon-universe --all-targets --locked -- -D warnings`, `cargo fmt --all -- --check` and `python3 scripts/check_docs.py`.

## Indexed selection and cloud acceptance

`SourceIndex` now supplies the production indexed-candidate path. It builds an
immutable evaluated-source snapshot (at most one million known sources) with
conservative enclosing-sphere bounds and per-node maximum gravitational parameter.
Branch-and-bound ranks actual mu/r², not nearest distance; a massive distant body
cannot be omitted by a nearest-neighbor assumption. Physical radii are included
in bounds so interior failures cannot be pruned. Split-position subtraction and
expanded numerical bounds preserve conservative pruning at distant origins.
`Selector::sample_indexed` pins active old/new handoff sources and uses the same
hysteresis and acceleration kernel as complete-source selection. Snapshot motion
updates belong to the epoch-aware caller; U21 owns playable streaming, not this
portable service. Query work counters expose visited nodes and evaluated sources.

The 32768-source / 512-query serial CPU workload exactly matches complete-source
selection for every query. It evaluates 176768 sources versus 16777216 complete
scan candidates (~1.05 percent), with observed median query time 155.505 us versus
9587.361 us for complete scans. Times are retained observations, not hardware FPS
budgets. Earth/Moon/Sun handoffs, small-body truncation (<1e-8 relative error for
the declared fixture), far massive-source admission and conservative interior
refusal have regression coverage. Existing stable transition/configuration and
all-source reference APIs remain available. No human signoff is a cloud gate.
