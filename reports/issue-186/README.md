# Issue #186 — signed persistent throttle and held input

Selected throttle is signed i8 −100..+100%. Native/automation Up/Down edges
change one point immediately and repeat every 100 ms of simulation time. OS
repeat and synthetic presses are ignored; duplicate portable presses are
idempotent. Opposing held keys cancel repeats. Release/direction changes restart
cadence. Authority, flight state and nonzero Core fixture energy gate changes.
Exit, power/focus/lifecycle/cursor/view loss and assists clear held input without
resetting selection; landing completion/collision resets selection to zero.

Ship force selection, snapshot, renderer DTO/cache and diagnostic observations
preserve the sign. Reverse display keeps its minus sign and shows absolute bar
magnitude. E2E `thruster_up`/`thruster_down` keys use the native gameplay route;
one-point `thruster` actions report whether authority accepted them.

## Validation

Linux x86_64; Rust/Cargo 1.99.0.

- Passed: `cargo test --workspace --locked` (including 120 runtime tests,
  signed-input composition, renderer atlas and reverse diagnostic formatting).
- Passed: `cargo test --workspace --locked controller::throttle` after adding
  immediate power-loss clearing; all six throttle regressions passed.
- Passed: `cargo clippy --workspace --all-targets --locked -- -D warnings`.
- Passed: `cargo build --workspace --locked`.
- Passed: `cargo fmt --all -- --check`, `git diff --check`.
- Passed: `/usr/bin/python3 -m unittest discover -s scripts -p test_baseline_scenarios.py`.
- An isolated renderer-only build hit an archive mapping failure in a different
  dependency feature build; workspace renderer tests passed, including the
  signed atlas contract. This isolated failure is not counted as a passing check.

Depends on #201 / PR #205 through the requested integration branch. Core remains
an energy telemetry fixture: this adds its input gate, not E01 consumption or
cargo accounting. Endpoint contact reset remains until #202 replaces it with a
swept crash state machine. Native M1 signoff is excluded; graphical launch is
unavailable locally (desktop packages/package-manager lock).
