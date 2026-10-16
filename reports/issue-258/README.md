# #258 — source-build-only CI

Baseline: `0cc9764` (2026-10-16). Replace the three-platform Native builds workflow
with **Source build**, one Ubuntu 24.04 job named **Source build (Ubuntu)**.
It compiles and links `cargo build --workspace --locked` once in the debug profile.

The five authored steps are checkout, stable Rust setup, Cargo cache, minimal
native build dependencies, and workspace compilation. Keep read-only repository
permissions, a 15-minute timeout and per-PR/branch cancellation of superseded runs.
Explicit `RUSTUP_TOOLCHAIN=stable` avoids installing local-development rustfmt/Clippy
components from the repository toolchain file for a build-only job.

Removed automatic OS matrix, Python/Node setup, tests, formatting/lint, model
processing, debug/release staging, optimized release compilation, benchmarks,
Xvfb/Mesa/input/capture setup, E2E/visual/smoke checks and artifact uploads.
Manual dispatch runs the same source build; the visual-evidence input is removed.
All local verification/build scripts and scenario files remain available.
Maintained guides now describe manual verification separately from automatic CI.

## Required-check migration

New check: **Source build (Ubuntu)**. If branch protection/rulesets require
`build (ubuntu-24.04)`, `build (macos-14)` or `build (windows-2022)`, a maintainer
must replace those requirements with the new check. This change does not edit
repository protection settings.

## Validation

- YAML parsing and scope assertions with Python 3.12 / PyYAML 6.0.3: passed.
  Verified the PR/main/manual triggers, exactly one Ubuntu job and five steps,
  real locked workspace build, timeout, concurrency cancellation, read-only
  permissions and checkout/Rust/cache setup.
- `git diff --check`: passed.
- Reviewed maintained documentation for stale automatic test/graphics/matrix claims.
- [PR source-build run 38899019926](https://github.com/mr-exception/salimon/actions/runs/38899019926):
  passed, exactly one Ubuntu job with the five intended steps. The real
  `cargo build --workspace --locked` compiled and linked successfully.
- New/changed Markdown relative file links: passed.

No native M1 or graphical check is needed for this workflow/documentation change.
Compilation errors propagate normally through the plain Cargo command; there is
no error suppression or `continue-on-error`.

## Timing baseline

[Previous workflow run 38895873017](https://github.com/mr-exception/salimon/actions/runs/38895873017)
(PR #257, commit `e33a6c8`): all three jobs passed. Execution durations from
GitHub job start/completion timestamps (excluding queue time):

| Previous job | Duration |
| --- | ---: |
| Ubuntu | 16m 18s |
| macOS | 3m 32s |
| Windows | 5m 50s |

The Ubuntu job included compilation plus all extra model, profiling, graphics,
evidence and smoke work. The new check measures source compilation only. Cache
and runner variability affect comparisons; this is not a performance guarantee.

## Measured result

The [first source-build job](https://github.com/mr-exception/salimon/actions/runs/38899019926/job/116966381213)
passed on commit `d990545` from 2026-10-16 12:13:27 to 12:14:51 UTC:
**1m 24s** execution, excluding queue time. Cargo reported **46.66s** for the
debug build. The cache action reported **No cache found**.

Compared with the previous cached Ubuntu job's **16m 18s**, this run is about
**91.4% shorter** (11.6× less elapsed time). The previous run reported a cache
hit and restored it successfully. These runs verify the reduced work rather
than a guaranteed runner speed: source revisions/cache states differ, and the
old job also performed tests, optimized builds, profiling and graphics work.

Step durations from GitHub:

| Step | Seconds |
| --- | ---: |
| Checkout | 4 |
| Stable Rust setup | 8 |
| Cargo cache restore (miss) | 1 |
| Native dependencies | 13 |
| Compile workspace | 46 |
| Cargo cache save | 7 |

GitHub job setup, cleanup and orchestration account for the remaining time.
See [ci-timing.json](ci-timing.json) for timestamps and the exact comparison.
