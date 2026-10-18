# #302 — Unified gameplay input admission and lifecycle resets

Native and automation keys now share `dispatch_key` and a typed `InputPolicy` /
`InputSource`. Real initial native gameplay presses require focused, captured,
non-occluded Gameplay view. Repeats and synthetic presses cannot install controls;
release edges clear both movement/steering meanings even after a view/location
change. Native mining and mouse look share native gameplay admission conditions.
Automation explicitly bypasses desktop admission, preserving view/domain gates.

E/L protocol action aliases now use the same production dispatch route. Tour
interaction/landing requests report `applied: false`; this field describes route
admission, not successful domain interaction. Tour camera/diagnostics controls
remain available. The runtime architecture is the canonical input policy.

One held-control reset handles capture/focus loss, view switches and lifecycle
clock resets. It clears movement, steering target/ramp, F/mining and held throttle
without changing selected throttle, momentum or pose. Ship exposes a narrow
steering-reset method; no new simulation or event framework was introduced.

## Validation

Local macOS; Rust 1.89.0, Python 3.14.2. Python 3.12 was unavailable locally.

- `cargo test --workspace --locked`: passed; 594 tests, 16 existing opt-in ignores.
- `cargo test -p salimon-client --locked app::input_tests`: 4 production dispatch /
  lifecycle composition regressions passed after final cleanup.
- `cargo clippy --workspace --all-targets --locked -- -D warnings`: passed.
- `cargo build --workspace --locked`: passed after final cleanup.
- `python3 -m unittest discover -s scripts -p 'test_*.py'`: 86 passed. The suite's
  printed simulated profile timeout is an intentional failure-path test; exit 0.
- `cargo fmt --all -- --check`, `git diff --check`: passed.
- Coverage includes Escape → W/E/L and recapture, walking/cockpit hold → F2 →
  release → return, mining/F reset, ignored OS-repeat/focus-replay presses, focus
  loss, occlusion and suspend clock-reset paths, automation bypass, blackout and
  uncancellable assist authority. Policy tests cover toolbar, throttle and steering.

## Limits and review order

Actual OS delivery, native minimize/restore/suspend, input feel and Metal gameplay
were not tested. Portable method calls do not establish those results; native
signoff remains pending in [#312](https://github.com/mr-exception/salimon/issues/312).
No native requirement was marked passed or waived.

This PR is stacked on [#316](https://github.com/mr-exception/salimon/pull/316)
(issue #301). Review/merge #301 first, then retarget this branch to main if GitHub
has not done so automatically. Only #302 changes belong to this PR's initial diff.
