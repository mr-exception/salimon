# Periodic verification and reports

PRs keep the single **Source build (Ubuntu)** check. The separate
[Periodic verification workflow](../.github/workflows/periodic-checks.yml) has
only schedule and manual triggers; it never runs on a PR or each push.

| Suite | Schedule (UTC) | Coverage |
| --- | --- | --- |
| Nightly | Daily 00:37 | Model and Python contracts, Rust fmt/Clippy/tests, GLB compliance/importer parity, software-Vulkan resource tests, one release build, deterministic baseline, screenshot evidence, model temporal and reviewed appearance comparison, packaged OS-input smoke |
| Weekly | Sunday 01:17 | Release CPU benchmark executable on Linux, 500/1,000-fragment scattered/dense × settled/active matrix with at least 100 samples per case, 100-sample flight/cargo regression profiles, resource-preparation comparison; debug workspace builds on macOS and Windows |

Schedules run the default branch. GitHub schedules can start late; these are
verification windows rather than exact-time deadlines. UTC times remain fixed
across daylight-saving changes (03:37 / Sunday 04:17 at UTC+3; 02:37 / Sunday 03:17 at UTC+2).
Schedules avoid the top of the hour. Native M1 performance remains local; a
hosted macOS build or software-Vulkan capture is not reference-machine signoff.
The generic model suite reports its normal Blender-dependent skips unless
Blender is installed; committed runtime exports and importer contracts are checked.

## Running manually and skipping unchanged work

After merging, open **Actions → Periodic verification → Run workflow** on the
default branch. Select `nightly` or `weekly`. **Force** defaults to true, so a
manual run can repeat measurements on the same commit. Manual runs from other
branches do not execute suites or publish reports.

Scheduled runs skip expensive jobs only when the latest report for every suite
in that group succeeded on the current commit. Missing reports, a changed commit
or a failed latest suite cause a retry. Weekly success requires all three lanes:
Linux benchmarks, macOS build and Windows build. Failed new revisions do not
inherit a previous pass. A plan summary explains skipped work.

## Finding the reports

1. Open [Salimon Actions](https://github.com/mr-exception/salimon/actions), select
   **Periodic verification**, then the run. Each suite and the reports job have a
   readable summary with status, commit and evidence links.
2. Download the run's suite artifacts for raw benchmark JSON/stdout/stderr, GLB
   diagnostics, screenshots and E2E state/logs. These expire after **30 days**.
3. Download **ci-dashboard**, unzip it, and open `index.html`. It works offline
   with its adjacent CSS/JS files, includes its history, and needs no server/CDN.
4. The same dashboard can be published as a persistent GitHub Pages URL below.

The reporting job stores compact metadata and CPU measurements on a dedicated
`ci-reports` branch, outside source history on main. It retains the most recent
**90 suite results**, including failures, pending appearance approval and separate rerun attempts. Failed-job reruns reuse passing
lanes from the same run's earlier attempts and select the newest available
attempt per lane; duplicate artifacts for the same lane/attempt are rejected. Report
publishing is serialized and uses ordinary non-force pushes, so nightly/weekly
results merge into the same history rather than overwriting each other. The
branch contains only the static dashboard and `data/history.json`, not game
sources, executables, screenshots or credentials. Its commits do not trigger the
main-only source-build workflow.

The dashboard shows latest suite cards, a filterable run history, and interactive
CPU p95 series with source/run links and the change from the previous comparable
measurement. Increases above 20% (previous p95 at least 0.1 ms) are marked
for review, without turning shared-host timing noise into a test failure. Series separate CPU models, platforms, runner images and complete
benchmark configurations. Missing or malformed output is a failed report, never
a fabricated zero. Failed suites with valid timing samples remain identifiable.
Shared-runner timing changes are diagnostic observations, not an automatic FPS
regression verdict. Flight profiles retain the existing reviewed limits in
`reports/issue-196/baseline.json`; raw fragment accounting failures still fail.
Reference-machine native FPS stays outside this dashboard's hosted CPU claims.

## Enable the published dashboard after merge

No repository settings are changed by this PR. A maintainer enables hosting once:

1. **Settings → Pages → Build and deployment → Source → GitHub Actions**.
2. **Settings → Secrets and variables → Actions → Variables**: set the repository
   variable **`CI_DASHBOARD_PAGES`** to **`true`**.
3. Run **Periodic verification** manually (force enabled), or wait for its next
   non-skipped scheduled run. The `dashboard` job publishes the generated artifact
   through the `github-pages` environment and exposes the deployment URL.

If the workflow succeeds but the site returns 404, check the `dashboard` job.
It must finish successfully; a skipped job has not deployed the site. Confirm the
opt-in is a repository **variable**, not a secret, with value exactly `true`.
The deployment condition explicitly permits skipped inactive suite lanes while
requiring successful report generation and a non-cancelled run.

With the repository's normal project Pages address, the dashboard will be at
`https://mr-exception.github.io/salimon/`. Use the actual deployment URL if a
custom Pages domain/source is already configured. This deployment owns that
project Pages root; review an existing site before enabling the variable.
Until enabled, the downloaded dashboard, Actions summaries, suite artifacts and
`ci-reports` history work without Pages. To stop public publishing, remove/set
`CI_DASHBOARD_PAGES=false`; this does not delete an already published Pages site.
Disable Pages separately in Settings if the existing site should be removed.

No external reporting service or token secret is required. Only the report
publisher receives `contents: write`; Pages permissions (`pages: write` and
`id-token: write`) are isolated to the opt-in deployment job. Verification and
benchmark jobs use read-only repository permissions. Publication only happens
from reviewed default-branch code, never PR artifacts.

## Maintaining the reporting tools

`python -m unittest discover -s scripts -p test_ci_reports.py -v` covers scheduling,
failures/missing evidence, provenance, bounded history, numeric validation,
configuration isolation, standalone rendering and report-branch ancestry.
`scripts/ci_reports.py` reads the existing benchmark schemas, generates summaries,
and renders `scripts/ci_dashboard/` into a static site. It does not modify game
simulation or benchmark thresholds. Workflow timeouts keep every lane bounded.
Cancellation or failure before the reporting job can run remains visible in
Actions; it cannot promise a dashboard record when the workflow never publishes.

The initial scheduled workflow and first Pages deployment can run after merge
because the new schedule/manual workflow must exist on the default branch.

## Appearance coverage (#308)

Nightly comparison uses the independently reviewed software-Vulkan corpus under
`scenarios/visual/baselines/software-vulkan-option-a-v1`. A missing/candidate/
pending corpus produces an explicit pending appearance check and pending lane,
not a green appearance result. Temporal stability and fixture completeness remain
required. Any real failure overrides pending status. Metadata mismatch or changed
appearance against an approved corpus fails. See the [baseline review workflow](../scenarios/visual/README.md#approval-policy-and-style-replacement).
Native Metal approval (#314) and software approval (#321) remain independent.
