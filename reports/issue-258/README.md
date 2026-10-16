# #258 — fast PR builds, periodic verification and reports

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
Manual dispatch of **Source build** runs the same source build; the visual-evidence input is removed.
All local verification/build scripts and scenario files remain available.
The follow-up request adds the separate schedule/manual-only workflow and
dashboard described below; these do not add PR/push checks.

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

## Scheduled verification and dashboard follow-up

The expanded PR adds `.github/workflows/periodic-checks.yml` with nightly Linux
verification (00:37 UTC), weekly CPU benchmarks plus macOS/Windows build checks
(Sunday 01:17 UTC), and manual suite/force choices. There is no PR/push trigger.
Successful unchanged revisions skip expensive work; failed/missing suites retry.
All suites check out the planned default-branch revision.

The standard-library `scripts/ci_reports.py` collects required step outcomes,
validates complete benchmark output and source/binary provenance, writes run
summaries, merges bounded durable history on `ci-reports`, and generates the
static dashboard. Failed and missing reports stay failures. Benchmark series
isolate CPU/platform/configuration rather than comparing incompatible runs or
converting CPU timings into native FPS.

Reports include suite status, commit, check outcomes and Actions links. Raw
screenshots/logs/measurements and the downloadable standalone dashboard have
30-day artifact retention. Compact history retains the last 90 suite results.
The report branch keeps normal commit ancestry and ordinary non-force pushes;
publishing is serialized. Only that job has repository write permission.

GitHub Pages deployment is opt-in and uses a separate least-privilege job.
After merge, select **Settings → Pages → GitHub Actions** and set the repository
variable **CI_DASHBOARD_PAGES=true**. Then manually run the periodic workflow.
The expected project URL is `https://mr-exception.github.io/salimon/`; deployment
outputs supply the actual URL. Existing Pages hosting should be reviewed before
enabling this project-root deployment. Until then, Actions summaries and the
downloaded dashboard work. See the [operation/activation guide](../../docs/periodic-ci.md).

Validation for the follow-up:

- `artifacts/actionlint -shellcheck= .github/workflows/*.yml`: passed with actionlint 1.7.7.
- `python -m unittest discover -s scripts -p 'test_*.py' -v`: 73 tests passed,
  including twelve report tests for skip/retry, failures, provenance, numeric
  samples, hardware isolation, bounded history, HTML embedding and branch ancestry.
- `node --check scripts/ci_dashboard/dashboard.js`: passed.
- Headless Chrome 155 + Playwright browser QA: populated and empty dashboards,
  seven trend points, history filtering and a 390px mobile viewport pass without
  overflow or JavaScript errors. The mobile select overflow found in QA was fixed.
- New/changed relative documentation links and `git diff --check`: passed.

The scheduled/manual workflow file first becomes runnable after merging onto
the default branch; no actual nightly/weekly run, history-branch write or Pages
deployment is claimed here. Those initial runs remain the post-merge activation
check, not native M1 signoff. The existing source-build PR check remains active.

### Dashboard preview

These browser screenshots deliberately use **synthetic fixture data**, visibly
labelled in the preview. They demonstrate layout/filter/chart behavior, not real
benchmark measurements or an already deployed dashboard.

[Desktop preview](dashboard-preview-desktop.png) · [Mobile preview](dashboard-preview-mobile.png)
