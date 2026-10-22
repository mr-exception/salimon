# #476 — versioned Scout/Absenat validation suite

2026-10-22. Delivered a version-1 matrix of **26 invariants**, exact portable
Scout orchestration, native comparison protocols and explicit backend fallback.
**Full experience, human native comparison and M1 performance remain pending**.

## Implemented and exercised

- [Matrix](../../scenarios/portable/experience-v1.json) traces #462 and the four
  approved experience/narrative/Archive/Absenat contracts. It covers waking and
  practical memory, cockpit/physical machinery, recovery and uncertainty, unknown
  Earth/discovered navigation, freeform alternatives, wormhole ambiguity, absolute
  precompletion spoiler protection, ceremony, durable unlock, ranking and revisits.
- [Runner](../../scripts/experience_suite.py) executes twelve exact current Scout
  tests through Cargo. Every mapping was checked against the compiled test list.
  Zero-selected tests, real failures and timeouts fail; blocked/native rows never
  pass by default. Output is preserved in a new directory with source revision,
  exact matrix hash and per-case command/logs; no prior failure is overwritten.
- [Actual portable result](portable-run-final/results.json): **12 passed, 14 blocked**,
  overall pending / exit 3. Native status is `not_run` on every row; backend and
  native signoff false. Original tests and runtime behavior are unchanged.
- [Tooling regression log](tooling-tests.log): three tests passed for zero-test
  rejection, failure precedence, incomplete native/backend acceptance, safe case
  names, explicit blockers and evidence preservation.

## Native comparison and fallback

[Canonical protocol](../../docs/experience-validation.md) requires continuous
physical gameplay, human images/audio/UI/accessibility review, actual M1/Metal
1080p context and matched raw CPU/GPU/presentation metrics. Linux screenshots
cannot satisfy those gates. Existing fixed-step/controlled poses cannot measure
an authentic one-hour opening or whole physical resource route.

E05 #471 and N03 #470 are open; #469 charting and A04 #475 remain open. No account
backend exists; #474 forbids implementation before that infrastructure. The
matrix retains those blockers and related prerequisites. Fallback is explicit
**blocked Hall acceptance**, with runnable Scout checks; no fake service,
production ceremony/rank, free hidden coordinates or public spoiler menu added.
Future native comparisons must exercise every matrix row after dependencies ship.
No great-circle/discontinuity/tie rule is invented by existing local distance tests.

## Validation and documentation impact

Portable run above passed its implemented checks; docs paths/anchors and
`git diff --check` passed. The ordinary workspace Rust tests passed at the base
revision with game sources unchanged. Full script contracts are also checked;
logs/results are recorded with final verification below. Owning runtime and
renderer architecture/invariants and all crate sources are unchanged. Updated
experience implementation map, new canonical validation guide and portable
runner guide expose the matrix's source/test/native boundaries in the same PR.

This PR is draft while full #476 acceptance remains pending; no source issue
closure or native/backend waiver is claimed from the matrix/tooling delivery.

## Final shared checks

- `cargo test --workspace --locked --offline`: **849 passed, 0 failed,
  28 ignored** across 39 test targets ([log](workspace-tests.log)). Those ignored
  graphical/workload tests remain unrun; no native signoff follows from CI tests.
- `python -m unittest discover -s scripts -p 'test_*.py'`: **118 passed**
  ([log](script-tests.log)). An earlier concurrent capture/compilation run
  [failed one existing 90ms screenshot-timeout test](script-tests-initial-contention.log)
  before its invalid-PNG assertion. The unchanged suite passed on rerun after
  resource contention ended; no timeout limit was relaxed. Printed simulated
  profile timeout belongs to a passing failure-path tooling test.
- The final exact Scout run retains matrix and runner SHA-256; game crate sources
  are unchanged from its recorded revision. The initial portable run is also
  retained as historical evidence in `portable-run/`.
