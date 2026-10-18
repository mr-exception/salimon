# #304 — Elapsed-time fragment damping

Linear/angular retention now uses `0.995^(dt/0.008)` / `0.98^(dt/0.008)`.
The reference is the ordinary 16 ms tick's two 8 ms subdivisions: normal
motion retains its existing loss, while small/fast unrelated objects no longer
multiply the loss by imposing additional steps. Unpowered floors remain undamped.
Factors are evaluated once per substep, outside the body loop. Contact, rest,
sleep and cutoff thresholds are unchanged.

Regressions cover a realistic 10 kg iron sphere, 1/4/8/16/40/160 ms partitions,
restarting calls, and a distant 5 mm fast residual that changes scheduling by
more than 10× without changing the subject's linear/angular retention. The
2e-14 velocity tolerance covers repeated floating-point exponential products;
semi-implicit position/gravity integration is not claimed partition invariant.
Existing contact/sleep/conservation/zero-g regressions remain required.

## Portable validation

- `cargo test -p salimon-physics --locked` against the standalone #304 baseline:
  **69 passed, 2 opt-in ignored**. Contact, sleep/wake, zero-g and the new damping
  regressions passed before indexing/scratch changes.
  [Baseline verification](https://github.com/mr-exception/salimon/actions/runs/39136864104).
- `cargo test --workspace --locked` on the first complete stack: **507 passed,
  21 opt-in ignored**. `cargo build --workspace --locked` and
  `cargo clippy --workspace --all-targets --locked -- -D warnings`: **passed**.
  [Complete stack run](https://github.com/mr-exception/salimon/actions/runs/39136204385).
- `python3 -m unittest discover -s scripts -p 'test_*.py'`: **92 passed**.
- The first formatting check exposed new-test formatting; runner rustfmt output
  was applied. `git diff --check`: **passed**. Rust 1.99.1/Ubuntu 24.04 were used;
  local Rust commands were unavailable because this editing container lacks Cargo.

Native M1 motion/performance/visual verification is **not run** and remains
pending in [#313](https://github.com/mr-exception/salimon/issues/313).
