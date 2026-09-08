# Portable persistence and survival scenarios (#375)

Run from the repository root, without a display, GPU or user save directory:

```sh
cargo test -p salimon-client --locked persistence_scenario
cargo test -p salimon-client --locked persistence_extended -- --ignored
```

The first command executes the bounded integration and failure matrix. The second
explicitly opts into the 500-fragment full-session continuation and eighteen-hour
16/33/100 ms survival partition matrix. Nightly CI runs the latter in its existing
Rust step and retains `artifacts/periodic/persistence-survival.log`; ordinary PR CI
runs bounded tests through `cargo test --workspace --locked`.

[`persistence-survival.json`](persistence-survival.json) is a version-1 test-only
DSL executed by
[`session/persistence_scenarios.rs`](../../client/runtime/src/session/persistence_scenarios.rs).
It extends portable protocol requests with checkpoint/reload lifecycle actions.
It is not a graphical `salimon-test` scenario and grants no save mutation operation
to the live automation protocol. Unknown operations/fields and versions fail.

| Operation | Meaning |
| --- | --- |
| `protocol` | Existing production JSON control request, with expected `ok` |
| `advance` | Admit milliseconds to the production fixed-tick session |
| `checkpoint` | Encode and verify a full session through the real two-slot filesystem manager at explicit UTC |
| `reload` | Drop the old session and use the normal launch transaction at explicit UTC; restart the test-only protocol afterward |
| `expect` | Assert an existing JSON Pointer in the whole-session save, including explicit nulls |
| `freeze` | Assert paused full-save identity after an elapsed interval |
| `compare_continuation` | Fork a validated full-save restore and require identical saved source state after both advance |
| `enqueue`, `collect` | Existing Fabricator session transactions; initial reserves come from `new_run` |
| `energy`, `damage` | Explicit test setup through ship/survival domain APIs |

Protocol `wheel_hover` uses **zero-based 0–4** indexes; the exact-identity `consume`
request uses 1–5. No M/N consumption semantics are introduced.

The Rust matrix supplies landed, moving, outside-EVA, cargo, blackout/Core and
terrain fixtures with partial crafting, active healing/damage, fractional clocks
and starter inventory. Full-source comparisons include power, mass, transforms,
identities and tutorial state, not just selected inspection values. Separate
fault-injected storage tests exercise the actual launch/checkpoint worker/restart
transactions. See the [coverage and results](../../reports/issue-375/README.md).

Fixtures may set initial placement/meters or seed a deposit. Timed advancement,
crafting, consumption, effect processing and restore use production owners. Long
meter tests advance survival directly to avoid millions of unrelated world ticks;
a separate session partition/death test verifies fixed-tick routing. These checks
do not certify OS input, visual/audio quality, physical gathering routes, native
M1 frame timing or owner acceptance of balance values.

## Scout and Absenat experience matrix (#476)

[experience-v1.json](experience-v1.json) is the versioned 26-invariant matrix for
the [approved experience/narrative contracts](../../docs/experience-validation.md).
It is separate from the persistence-survival DSL above. Each implemented Scout
row names an exact existing production-controller test and a native comparison
protocol. Unimplemented opening/discovery/story/Hall rows retain explicit issue
blockers; their prose is a future acceptance protocol, never fake gameplay.

```sh
python scripts/experience_suite.py --output artifacts/experience-v1-run
python -m unittest discover -s scripts -p test_experience_suite.py -v
```

Run from the repository root with the normal Rust toolchain/prerequisites.
Each exact test must report one actual passed test; zero-selected cases fail.
Logs, source revision, matrix digest and all blocked/native outcomes are retained
in a new output directory. Existing evidence is never overwritten. Exit 1 means
failure, and exit 3 means portable checks passed with native/backend work pending.
This portable runner cannot emit full native/backend acceptance. Do not install a
production account service or unlock the Hall to make this suite appear green.
