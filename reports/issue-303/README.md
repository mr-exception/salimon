# #303 — Checked spherical fixtures and current 500/1,000-object workloads

Carrying's requested 10 kg case previously called legacy extraction for one
second, creating at most 2 kg. It now uses checked default spherical extraction,
including tiny residuals and all three materials. The append-only test builder
checks actual mass, independently calculated density radius, stable growth ID,
source, count and release before behavioral assertions. Multiple cap releases,
residual depletion and one-object carrying have dedicated coverage.

Contact fixtures now construct current spheres; the all-material settling test
uses two actual 10 kg spheres per material. Core conversion fixtures use the
production lifecycle from 1e-9 kg through 10 kg and assert exact energy yield
from the actual admitted physical mass. Conservation checks retain exact
identity/material comparisons across motion; rounding budgets apply only when
comparing requested versus produced fixture mass/radius.

## Legacy fixture audit

| Owner | Disposition |
| --- | --- |
| Runtime carrying | Replaced one-second legacy setup; actual 10 kg, residual and radius checks precede carrying/drop behavior |
| Runtime contact tests | Default lifecycle builder; original two-piece adapter populations retained; cap-sized all-material settling added |
| Explicit unsealed-tail growth regression | Retains labeled legacy extraction because released current spheres must never reconnect to a later hold |
| Ship Core tests | Default current sphere lifecycle; actual ID/count/material/mass/radius checked; cap-sized and tiny inputs added |
| Runtime release/carry/drop/Core route | Existing 10 kg and depleted residual path now independently validates radius and released ID |
| Flight/bootstrap fixtures | Retain exact historical 2 kg bootstrap and 0.032 kg cargo, with explicit mass/population rejection guards (including an already held bootstrap piece) |
| #47 mixed transfer and generic legacy benchmarks | Retain their explicit 2 kg-piece or mixed 0.5/2 kg populations; no cap-sized acceptance inferred |
| #196 flight CPU baseline | Explicit `legacy-onboard-032kg-v1` metadata; historical 500 × 0.032 kg = 16 kg baseline unchanged |
| #235 sphere matrix/native sphere layouts | Existing separate 10/100/500 matrix and 500-object native cap unchanged |

The canonical [fixture/workload guide](../../scripts/FRAGMENT_BENCHMARKS.md#current-sphere-fixture-and-cpu-workload-contracts-303)
records ownership, commands, identities, tolerances and interpretation limits.
No production mining, contact, damping or sleep rules were changed.

## New portable CPU evidence

[Raw populations, metadata and 100 samples per case](current-spheres-cpu.json)
use the separate `current-spheres-303-v1` workload. The release run passed all
population/identity/material/selection/onboard-accounting assertions.

| Actual count | Profile | Actual total kg | Radius range m | Adapter p95 ms |
| --- | --- | --- | --- | --- |
| 500 | cap-10kg-iron-v1 | 5000.000000000 | 0.088027 | 3.629 |
| 500 | mixed-residuals-v1 | 1019.406252084 | 0.005000–0.137417 | 107.429 |
| 1000 | cap-10kg-iron-v1 | 10000.000000000 | 0.088027 | 214.467 |
| 1000 | mixed-residuals-v1 | 2039.094501167 | 0.005000–0.137417 | 1766.059 |

Layout: supported 30 cm deck cells with conservative largest-radius layer
spacing, 30 cm initial lift and 0.4 m/s upward velocity. Mixed masses cycle
1e-9, 0.000003, 0.032, 0.25, 2 and 10 kg across all materials. Population metadata
retains every actual ID/material/mass/radius/initial local pose. These cases
include settling; observed moving counts range 0–500, 0–500, 0–1000 and 161–1000
respectively. No sleep state or forcing was installed.

Timings are descriptive fresh-drop CPU costs, measured locally on iMac21,1,
arm64/macOS 26.2, Rust 1.89.0 release. Workspace validation overlapped collection;
this is not a controlled performance comparison or an approved baseline. The
large mixed workload is expensive; no timing budget or performance pass is
claimed. Damping/indexing/scratch work remains in existing
[#304](https://github.com/mr-exception/salimon/issues/304),
[#305](https://github.com/mr-exception/salimon/issues/305), and
[#306](https://github.com/mr-exception/salimon/issues/306).
Raw metadata retains the tested executable hash and unchanged workload-source
hashes. Measurement preceded final legacy-fixture assertion/test corrections.

## Checks

- `cargo test --workspace --locked`: passed, 496 tests; 17 opt-in ignores.
- `cargo test -p salimon-client --locked fragment_physics::tests::`: 12 passed.
- `cargo test -p salimon-client --locked flight_regression::tests::loaded_blackout`:
  passed after correcting fixture count validation to include the held bootstrap.
- `cargo test -p salimon-ship --locked conversion::`: 4 passed after final exact
  physical-input energy assertions.
- `SALIMON_CURRENT_SPHERE_CPU_REPORT="$PWD/reports/issue-303/current-spheres-cpu.json" cargo test --release -p salimon-client --locked current_sphere_workloads_cpu -- --ignored --nocapture`:
  passed, four 500/1,000-object cases, 100 samples each (99.80 s test run).
- Independent JSON integrity check: four cases, unique IDs, requested/actual
  population equality, 100 samples each and mass sums within 1e-8 kg passed.
- `cargo clippy --workspace --all-targets --locked -- -D warnings`: passed.
- `cargo build --workspace --locked`: passed.
- `python3 -m unittest discover -s scripts -p 'test_*.py'`: 86 passed with
  Python 3.14.2; 3.12 unavailable. The printed simulated timeout is expected test
  output; the suite exits 0.
- `cargo fmt --all -- --check`, `git diff --check` and affected relative-link
  existence checks: passed.

Native Metal visuals, actual presented FPS, active-only budgets and OS gameplay
were not tested here. Pending native sphere/performance signoff remains in
[#313](https://github.com/mr-exception/salimon/issues/313) and its coordination
with [#255](https://github.com/mr-exception/salimon/issues/255).

Review order: stacked on [#317](https://github.com/mr-exception/salimon/pull/317),
which follows [#316](https://github.com/mr-exception/salimon/pull/316). Merge those
first and retarget this PR to main if needed. Historical evidence is preserved.
