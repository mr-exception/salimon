# Issue 196 — 500-cargo CPU regression profiles

Adds release-only `--flight-profile`, a Python recorder/comparator and a Linux CI
gate against the committed [baseline](baseline.json). Twelve rows cover 0/500
physical fragments: event-driven idle mass reads, boundary exit/re-entry, moving
powered/zero-G cargo, power transitions and dense authored deck contacts.
Every sample validates unchanged IDs and expected onboard mass outside its timed
region. Raw timings, p50/p95/p99/max, motion observations, binary/source/machine
metadata and explicit regression limits are retained. The 500-piece fixture is
16 kg of authored iron fragments (0.032 kg each), with all centers onboard.

## Validation

Linux x86_64, Rust 1.99.0, shared container; release codegen units = 1.

- `cargo build --release --locked -j 2`: passed.
- `python scripts/profile_flight.py --binary /tmp/salimon-target/release/salimon-client --samples 20 --output reports/issue-196/baseline.json`: all 12 rows passed.
- Final release `--baseline reports/issue-196/baseline.json` run: [all 12 rows passed](verification.json), including the final fixture-tagged CLI.
- Baseline self-comparison through `compare(baseline, baseline)`: passed; validator checks all rows, raw percentiles, sample counts and mass.
- Locked workspace tests passed; final focused profiler tests: 2 passed.
- `cargo clippy --workspace --all-targets --locked -- -D warnings`: passed.
- Python runner contracts: 58 passed, including missing/duplicate/nonfinite/mass/statistics rejection, slowdown detection and timeout evidence preservation.
- Formatting and diff checks passed. Test builds use the same single-codegen-unit container workaround recorded for #193.

## Recorded p95 CPU work (milliseconds)

| Workload | 0 fragments | 500 fragments |
| --- | ---: | ---: |
| mass-idle | 0.0003 | 0.0003 |
| mass-crossing | 0.0000 | 0.0008 |
| moving-powered | 0.0020 | 26.5958 |
| moving-zero-g | 0.0019 | 7.7425 |
| power-transition | 0.0021 | 17.0764 |
| dense-contacts | 0.0020 | 688.7018 |

## Interpretation and limits

These are elapsed CPU update windows, not rendered frame times. They do not
establish hardware FPS, an M1/60 FPS pass or general multilayer-pile acceptance.
Collision rows use supported single-layer authored hull contacts at 12 cm spacing;
moving rows start separated above the aft deck; motion counts expose settling.
The existing `--benchmark --layout dense` remains available for multilayer stress.

An exploratory overlapping stacked fixture timed out and is explicitly rejected;
[failed evidence](rejected-overlap.json) retains its completed rows/stderr, never
an accepted dense baseline. A later [non-overlapping multilayer stress run](stacked-stress.json) completed
20 samples per row but **failed** the dense p95 ceiling: 25,156.9 ms versus
1,000 ms. It is retained as a failure, outside the accepted deck-contact baseline.
Reproduce it with the same command plus `--stacked` (no deck baseline comparison).
Its source/binary metadata records the pre-flag experimental build using the
same 12 cm, ten-by-ten-by-five initial geometry. The solver's
previously documented worst-case dense-pile costs remain unresolved; this issue
adds measurements/guards and does not replace or optimize the solver.

CI enforces workload-specific absolute p95 ceilings and 4× baseline plus explicit
noise allowances. Missing cases or malformed summaries fail. Machine contention
and small samples limit absolute comparisons; use matching release builds and
machine conditions for optimization decisions. No M1 hardware signoff was run.
