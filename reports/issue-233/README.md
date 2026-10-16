# Issue 233 — analytic spherical fragment contacts

Loose fragments now use the same bounded mass/density radius as the shared
sphere visual, with no mesh hull/SAT contacts. Physics owns spherical inertia,
radial contact arms, cached friction/rolling impulses, sixteen pre/post contact
passes and diameter/speed-adaptive steps (4096 cap). Grid/correction/wake ordering
and stable-ID sleep remain shared with convex objects. Deep sphere penetration
cannot qualify for sleep; floor support cannot absorb downward pair correction.
Attached/carried mass is excluded from loose simulation.

Validation (Linux x86-64, Rust 1.89.0):

- `CARGO_INCREMENTAL=0 cargo +1.89.0 test -p salimon-physics --locked`:
  **64 passed, 1 opt-in benchmark ignored**. Includes momentum/energy, rolling,
  stack/support removal, 500-sphere candidate/sleep/storage and fast small-sphere
  impact tests; existing convex/grid oracle regressions pass.
- `CARGO_INCREMENTAL=0 cargo +1.89.0 clippy --workspace --all-targets --locked -- -D warnings`:
  passed.
- `cargo +1.89.0 fmt --all -- --check`, `git diff --check`: passed.
- Runtime integration test/build results are recorded below after completion.

Initial incremental artifacts failed to link (undefined generated symbols);
non-incremental physics tests pass. Runtime dependency archives also exposed
zero-length object errors; validation retries use one codegen unit and no debug
symbols, without changing repository build settings.

This changes contact size deliberately from enlarged shard envelopes to the
actual sphere. Final pickup ray/carry clearance and legacy emission adapters
are #234; their interim inflated clearance is conservative. Pile measurement
and portable integrated regression are #235. Native Apple M1 remains #236;
no hardware/GPU FPS is inferred from CPU tests. Ordinary bounded stepping is
not general continuous collision detection.

Runtime validation: 144 tests passed on the first final run; the remaining
transfer route still compared old shard resting coordinates. Updated its two
matched checkpoints to the measured spherical resting pose with unchanged
2 cm tolerance; the focused production-input transfer route now passes. Earlier
fixture support/canonical-snapshot expectations were updated and passed in that
144-test run. The runtime build passes with `CARGO_INCREMENTAL=0`,
`CARGO_PROFILE_{DEV,TEST}_CODEGEN_UNITS=1` and
`CARGO_PROFILE_{DEV,TEST}_DEBUG=0` (Rust 1.89.0). No build flags were committed.
