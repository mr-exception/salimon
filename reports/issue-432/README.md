# Issue #432 — work in progress

Status: **incomplete; no PR, no issue closure, no end-to-end acceptance**.
Base: `e68ccb8` (`origin/main`, 2026-10-31); branch
`codex/offline-universe-five-tasks`. This is the first part of the requested
single-PR batch for #432, #433, #434, #441 and #435.

The local `salimon-ship::offline` candidate adds bounded resumable gravity-only
propagation over the production gravity, drag and continuous-contact operators.
It stages trajectory and timed-consumer changes together, records progress and
contact uncertainty, follows verified support after impact and rejects incomplete
results or invalid provider/consumer work. No ordinary gameplay/save/UI behavior
has changed. The [contract and pending motion proposal](../../docs/universe-offline-propagation.md)
record exact boundaries and remaining work.

## Verification so far

Environment: existing Linux cloud executor, Rust/Cargo 1.99.1 toolchain from
`/workspace/.salimon-tools/env.sh`, locked Cargo dependencies, debug profile.

- `cargo test -p salimon-ship --locked offline::`: **5 passed**, zero failures;
  four new propagation regressions and the existing closed-power regression.
  Orbital fixture budget: <30 m analytic and half-step position difference after
  one approximately 5,829-second orbit at radius 7,000 km. This is a fixture
  accuracy measurement, not a certified global error bound.
- Final `cargo test -p salimon-ship --locked`: **156 unit tests and 1 integration test passed**, zero failures, 1 existing opt-in test ignored. Includes moving-body impact/support velocity and final-nanosecond progress rounding.
- `cargo clippy -p salimon-ship --all-targets --locked -- -D warnings`: **passed**.
- `cargo fmt --all -- --check`: **passed**.
- `python scripts/check_docs.py --base e68ccb8`: **passed**, zero findings.
- `git diff --check`: **passed**.
- Iteration failure retained: the new moving-body fixture initially asserted an exactly representable 50 m/s derivative; cubic differentiation produced `49.99999999999999`. The assertion now allows 1e-12 m/s floating-point roundoff. No production tolerance or physics contract was changed. The final owner-crate run passes.

[Checked source hashes](source-hashes.json) identify the source/test inputs of the final owner-crate checks.
- Broad workspace, native GPU, scanner/timed-system, checkpoint/restart, final
  audit and integrated benchmark acceptance: **not run/not implemented yet**.

## Open decision and limitations

Vesper's frozen data and current saves lack the epoch/orbital phase/orientation/
spin needed to reconstruct an inertial journey. A concrete fixed fictional
motion-publication proposal is pending in chat. No proposed value has been
inserted into production definitions, no save migration is implemented, and no
permission or scientific accuracy is inferred from the test fixtures.

Routine cloud work does not establish physical platform/input (#557), subjective
presentation (#561), hardware/Metal performance (#563), source redistribution
review (#659), balance (#660) or first-hour experience approval (#661).
