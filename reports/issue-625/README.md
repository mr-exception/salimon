# Issue 625 — suit measurements and field observations

Parent: PR #706, `issue-427-canonical-play`, exact head
`8d6aa969e978c87b243895a00a78bc94fd164a69`. Prerequisites #611/#421/#469/#406
are merged and their required code is included in this ancestry. No merge performed.

## Delivery

F10 records an estimated spot, F11 a calibrated bearing with unknown range, and
F12 disconnects/reconnects suit navigation. The production adapter requires a
powered, visible, in-range Scout beacon for a new quantized fix. Between fixes,
per-tick inertial displacement carries explicit drift. No available reference
means unavailable knowledge; disconnected/unreferenced F11 cannot save a true
camera direction as a claimed measurement. Readout words, numerical error and
range-unknown text remain explicit. No equipment/consumption control is replaced.

Stable per-observation IDs, body/definition/frame versions, shared TT timestamps,
measurement source/configuration/error and optional bounded notes survive schema
27 saves. Legacy canonical sources migrate empty; current omissions, malformed
source, future epochs and changed definitions reject before publishing. No new
POI, Earth route, scientific identification or scanner pulse is inferred.

[Canonical contract and source/test map](../../docs/suit-observations.md) records
all model defaults and scope. Universe owns measured knowledge, runtime owns
physical inputs/controls and persistence owns the envelope. Detailed personal
editing/chart review remains the separately scoped #626 deliverable.

## Actual verification

Debian Linux x86_64, Rust/Cargo 1.99.1, Python 3.12.14; workspace local Rust/sysroot
environment (`source /workspace/.salimon-tools/env.sh`). Production Xvfb/Mesa
llvmpipe Vulkan, 1280x800, LP_NUM_THREADS=2, protocol input, fixed 16 ms steps.

- `cargo test --workspace --locked --offline`: **1217 passed**, zero failed,
  49 existing opt-in ignored ([log](workspace-tests.log.gz)). This full run
  preceded the final unavailable-bearing admission tightening; affected final
  tests and the final production route below pass with that tightening.
- `cargo test -p salimon-universe --locked --offline surface_observations`:
  **4 passed** ([log](domain-tests.log.gz)): error envelopes/partitioning,
  strict failure/limits, unavailable/bearing knowledge, exact rotating-frame
  transforms and stale epoch rejection.
- `cargo test -p salimon-client --locked --offline field_`: final **3 passed**
  ([log](session-tests.log.gz)): real controls, positive beacon acquisition after
  ordinary walking, power loss, unchanged chart/scanner source, pause, exact
  roundtrip, current omissions, legacy migration and atomic rejection.
- `cargo build --workspace --locked --offline`: final pass.
- `cargo clippy --workspace --all-targets --locked --offline -- -D warnings`:
  final pass. `cargo fmt --all -- --check`, `git diff --check`: pass.
- `python3 scripts/check_docs.py --base issue-427-canonical-play`: zero failures.
- `python3 -m unittest discover -s scripts -p 'test_*.py'`: **136 passed**;
  intentional timeout/failure-fixture output is not a suite failure.
- `LP_NUM_THREADS=2 MESA_SHADER_CACHE_DIR=/workspace/salimon/artifacts/mesa-cache /workspace/.salimon-tools/start-display.sh scripts/salimon-test run scenarios/evidence/suit-observations.json --binary target/debug/salimon-client --screenshot-command '["native"]' --summary --artifacts artifacts/issue-625-final`:
  **20/20 passed**, 34.138 s; [result](result.json.gz),
  [lossless protocol](protocol.jsonl.gz), [binary/capture hashes](hashes.json).

Final originals were inspected: [bounded fix](step-009-bounded-fix.png),
[bearing-only](step-013-bearing-only.png),
[unavailable navigation](step-017-navigation-unavailable.png). Readouts fit the
viewport and state quality/error and unknown range with text, not color alone.

## Corrected initial failures and limits

Initial runtime tests used equality on checkpoint DTOs without PartialEq; they
now compare serialized source values. An overbroad pause assertion also compared
unrelated tutorial progression; the regression now checks the field source while
separately checking full save roundtrip. Clippy's unnecessary unit expression was
removed. The first native route passed 20/20 (106.639 s); final captures above
were regenerated after tightening unavailable-bearing admission. No failed or
unrun criterion is promoted to a pass.

The instrument model is declared simulated measurement, not real sensor accuracy
or universal GPS. Current Vesper axes remain fixed; inertial consumers must use
an evaluated rotating FrameTree. No celestial fix, suit battery, global map,
autopilot, automatic landmark verification or new scanner mode is added. Actual
physical controls/platform behavior (#557), subjective readability/art (#561)
and M1/Metal budgets (#563) remain separately unverified release qualification.
