# Issue 254 — native foreground admission

Validated locally on 2026-10-17, Apple M1 iMac21,1, 16 GiB RAM, macOS
26.2 (25C56), Metal, 1920×1080 drawable, 2× scale, AutoVsync. Rust 1.89.0,
Python 3.14.2. Source base `220eb84` plus this PR's runtime changes; exact staged
binary digest and build provenance: [build-info.json](build-info.json).

Native benchmark warmup now requires actual window focus, known visibility,
no minimization and no reported occlusion. Missing readiness cannot start sampling.
Loss during warmup restarts its clocks; loss after retained intervals, suspension,
surface loss and configuration changes reject collection. Raw results retain PID
and monotonic lifecycle transitions. The Python summary and relative comparison
both reject missing/incomplete readiness, including historical unverified baselines.
CPU-only measurements retain their existing behavior. No refresh-rate admission
threshold, physics/sleep/accounting change or new OS focus automation was added.

## Results

- Focused bundled executable: passed, 481 samples in eight seconds. Median
  successful-present interval 16.689 ms (~59.92 intervals/s), p95 17.282 ms,
  maximum 18.856 ms. [Raw samples](focused/raw.json.gz),
  [PID focus records](focused/probe.json), [native log](focused/stderr.log).
- Deliberate Finder activation five seconds after verified focus: correctly
  rejected with exit 1, `benchmark native readiness lost: unfocused`.
  [Probe records](focus-loss/probe.json), [native log](focus-loss/stderr.log).
  No accepted result was emitted.
- `cargo test --workspace --locked`: 482 passed, 13 pre-existing opt-in ignores.
  [Full log](rust-tests.log).
- `cargo clippy --workspace --all-targets --locked -- -D warnings`: passed.
  [Log](clippy.log).
- `cargo fmt --all -- --check` and `git diff --check`: passed.
- `python3 -m unittest discover -s scripts -p 'test_*.py'`: 80 passed.
  [Log](python-tests.log). Python 3.12 is unavailable here; tests used 3.14.2.
- `python3 scripts/build_game.py --profile release --output artifacts/build/native-254`:
  passed. [Log](build.log).

## Reproduce

Copy the staged binary into a local macOS app bundle as in the
[earlier native setup](../issue-199/2026-10-16/README.md#reproduction-and-checks).
Compile the reviewed [PID input helper](../issue-198/2026-10-16/input.swift)
with `swiftc -O reports/issue-198/2026-10-16/input.swift -o artifacts/native-input`.
The explicit opt-in probe activates/clicks only the target game; its loss variant
activates Finder. Run on an idle local desktop with event-posting permission:

```sh
python3 reports/issue-254/native_probe.py --binary artifacts/SalimonAcceptance.app/Contents/MacOS/salimon-client --helper artifacts/native-input --output artifacts/native-254-focused
python3 reports/issue-254/native_probe.py --binary artifacts/SalimonAcceptance.app/Contents/MacOS/salimon-client --helper artifacts/native-input --output artifacts/native-254-focus-loss --lose-focus
```

Successful present calls do not certify physical scanout or a visible-display
60 FPS floor. This PR validates foreground/visibility admission, not #255's
active 500 performance budget. Unfocused/unknown windows remain subject to the
runner's bounded timeout. No new asset exports or aesthetic baseline approvals
are involved. Other platforms and a full interactive lifecycle smoke were not run;
the native admission-loss path and deterministic lifecycle contracts were tested.
