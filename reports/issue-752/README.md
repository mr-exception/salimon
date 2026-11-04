# #752 — independent game reviewer and CI acceptance gate

Implemented the trusted [review runtime](../../.github/qa/review.py), strict
[veto policy](../../.github/qa/policy.py), bounded revision invalidator and
[workflow](../../.github/workflows/independent-qa.yml). Independent AI review runs in the active Codex session; CI verifies its exact
source/evidence-bound handoff. The runtime never invokes an external AI API or
requires an AI API key, and never executes PR code with privileged credentials. It publishes exact-head commit statuses, actionable
PR comments and durable machine-readable cycle records. Human retains merge
authority. [Operations/rollout/limits](../../docs/independent-game-qa.md).

Delivery branch `issue-752-independent-qa-gate`, base `main` at
`8e2adab5d138bab6da4f96591fcd990193f3e1b6`. No open stack existed at preflight.
Verified #749/#750/#751 code, approval discussion and merged delivery; formal
blocked-by list is empty. No unrelated gameplay or reference image is changed.

## User-directed scope

The user explicitly declined external AI/API-key/unattended review and requested
matching plugin instructions. Salimon Issue Manager0.7.1 is saved and read-back
verified with session reviewers as the default; external AI infrastructure needs
an explicit future request. The earlier external-provider proposal in the first
published PR revision is superseded by the session handoff gate. No automated
model execution is claimed.

## Acceptance matrix

| Criterion | Production evidence and verification | Result before final PR review |
| --- | --- | --- |
| AC-1 wrong indicator requests changes; repair requires new-SHA tests/evidence | Actual red-velocity CSS commit + genuine distinct image review; production orchestrator replay against controlled PR API; repair at different real commit with 24 E2E passes | PASS, [machine-readable demo](demo.json) |
| AC-2 missing reviewer/capture blocks; finite loops | 29 Python policy/runtime/session-handoff tests, including missing session reviewer/capture, expiry, stale head/base/merge, malformed response, withdrawn records, full-context binding, cycle cap and duplicate replay | PASS, [test log](qa-tests.log) |
| AC-3 actionable comments, machine report, human merge authority | Demo includes exact comments/status records; runtime writes same JSON to PR and artifact; independent source review; no merge/close/deploy endpoint | PASS locally; final current-head CI/reviewer verdict bound in PR conversation |
| Mandatory existing client gates | 38 assertions, build/typecheck, all 24 Chromium routes; final-head CI also repeats existing deterministic/visual mutation checks and baseline approval | Local PASS; final remote run recorded in PR |
| Manual/native/hardware | No graphics/product/input change; hardware-specific visual qualification is separate from cloud acceptance | N/A to infrastructure; hardware/native/physical input NOT VERIFIED |

## Controlled real-image review demo

The PR/GitHub status transport is **simulated**, using the production `run`,
`publish`, `decide` and `invalidate` functions. Review records use the
production handoff contract for actual distinct-agent image verdicts; the gate
never makes a model call. Browser runs, source commits, screenshots,
measurements and independent AI image inspections are real.

- Wrong source `08faa0d6cf2fc822dd77b16f1b62f8f7b73211a6`: actual velocity stroke
  `#b6f5a2` → `#ff0000`. Browser comparison fails at 520 / 630 pixels in
  heading-velocity / opposite-vectors; six controls unchanged. Reviewer
  `/root/review_752` independently inspected all eight originals, changed
  references/diffs and metadata, and verified trace archive integrity/availability;
  returned REQUEST_CHANGES / AC-1 FAIL.
- Repaired source `edd62718eb6dbfb55ddde51570a8afc892c02bf3`: actual restoration
  commit. 38 unit assertions/build/typecheck and all 24 browser routes pass;
  all eight frames have zero perceptual and raw differences. The distinct
  reviewer verified all hashes, two repaired originals, eight source/backend
  observations and trace archive integrity/availability, then returned ACCEPT for
  the navigation mini-criterion. No interactive trace replay is claimed.
  This does not self-accept the entire #752 implementation.
- Replay: wrong → failure/REQUEST_CHANGES; new head before CI → pending;
  old CI on repaired head → failure/BLOCKED; new tests/captures → success/ACCEPT;
  parent advances → pending. Actual state/comment objects and exact 34-file
  evidence hashes per condition are in [demo.json](demo.json).

Actual reviewer records: [wrong](wrong-review.json) and
[fixed](fixed-review.json). These are source/hash-bound agent records, not
cryptographic signatures or human baseline approvals. References inherit the
already approved #751 manifest; nothing regenerates/promotes it.

The [Git bundle](fixtures.bundle) preserves both actual fixture commits and their
implementation ancestors, with only merged `8e2adab` as prerequisite. Verify with
`git bundle verify reports/issue-752/fixtures.bundle`; import into a disposable
checkout using `git fetch .../fixtures.bundle refs/heads/qa-fixture-752`.
The full client tree of the repaired fixture equals delivery source `0a791e2`.
Later QA-only fixes do not change that tree; [all client hashes](client-inputs.sha256.json)
bind the reused browser/build/unit evidence. The revised 29 runtime tests and demo
run against session-only source `b18600a31a2e934d582c3e00c3ba3c6a5ce47ad5`;
current-head CI is required after publication. Client
inputs and the original real image reviews are unchanged and reused by hash.

Retained primary originals, expected frames, diffs, metadata, full E2E result
JSON and trace:

- [Wrong primary evidence](evidence/wrong/navigation-captures-VIS-1--c220e-eens-match-reference-pixels-navigation-captures/comparisons/summary.json),
  [trace](evidence/wrong/navigation-captures-VIS-1--c220e-eens-match-reference-pixels-navigation-captures/trace.zip),
  [raw failure results](evidence/wrong/results.json).
- [Repaired primary evidence](evidence/fixed/navigation-captures-VIS-1--c220e-eens-match-reference-pixels-navigation-captures/comparisons/summary.json),
  [trace](evidence/fixed/navigation-captures-VIS-1--c220e-eens-match-reference-pixels-navigation-captures/trace.zip),
  [raw 24-route results](evidence/fixed/results.json).

Raw Playwright attachment paths retain their original `/tmp` execution locations;
the corresponding primary files are preserved under these relative directories.
Text log/error-context exports trim trailing whitespace; original images, trace
archives, JSON measurements/results and all reviewer-bound hashes are unchanged.
Nonvisual success screenshots are not duplicated into this report. Final CI
uploads the full fresh suite artifact under its own exact tested merge revision.

## Commands and environment

Cloud Linux x86_64, Python 3.12.14, Node 24.19.0; locked Playwright 1.64.1 /
Chromium 156.0.8078.4, headless real WebGL SwiftShader. Fixed 1280×800 / DPR1,
seed750, epoch2026-11-04T00:00:00Z, en-US/UTC/dark, vendored fonts and grayscale
text. Browser synthetic input; no physical-device claim. No external AI service or API key is used by the final design.

| Command | Actual result |
| --- | --- |
| `npm ci --no-audit --no-fund --cache /tmp/salimon-npm-cache`; `PLAYWRIGHT_BROWSERS_PATH=/tmp/salimon-playwright npx playwright install chromium` | PASS |
| `python3 -m unittest discover -s .github/qa -p 'test_*.py' -v` from root | PASS, 29 tests; [log](qa-tests.log) |
| `npm test` from repaired fixture client | PASS, 38 assertions; [log](client-tests.log) |
| `npm run build` from repaired fixture client | PASS; existing bundle-size warning; [log](build.log) |
| `PLAYWRIGHT_BROWSERS_PATH=/tmp/salimon-playwright CI=true SALIMON_EVIDENCE_DIR=/tmp/issue-752-wrong npm run test:visual` on wrong commit | Expected FAIL, only two visual frames; [log](wrong-browser.log) |
| `PLAYWRIGHT_BROWSERS_PATH=/tmp/salimon-playwright CI=true SALIMON_EVIDENCE_DIR=/tmp/issue-752-fixed npm run test:browser` on repair | PASS, 24 routes in 2.3 minutes; [log](browser.log) |
| `python3 .github/qa/demo.py --wrong /tmp/issue-752-wrong --fixed /tmp/issue-752-fixed --wrong-review /tmp/issue-752-wrong-review.json --fixed-review /tmp/issue-752-fixed-review.json --output /tmp/issue-752-demo` | PASS, real reviewer records and controlled status transitions |
| Python compilation, both workflow YAML parsing, `git diff --check`, bundle verify | PASS |

Initial sandboxed git/network and browser webserver startup failed before tests;
the environment's authorized network capability resolved both. Wrong-frame
failure is retained as required negative evidence, never reclassified as a
passing screenshot run. No threshold, mask, approved baseline or feature test is
weakened. Earlier implementation-review findings (expired-evidence replay, state
visibility, aggregate budgets, malformed output, CI plumbing and stacked baseline
trust) were corrected and covered by targeted tests before final verification.

## Independent review and limits

Distinct reviewer `/root/review_752`, implementation cycles1–2, independently
reviewed production code, fixtures, original images and hashes. Initial findings
were handed back and fixed. Final overall verdict is QA pending until the exact
published head/base and actual required remote checks are reviewed; the final
structured verdict/run links are published in the PR and source issue. The
navigation mini-verdicts above do not substitute for that full review.

No external AI API client, model request, API-key secret or model retry path is
present in the final active source/workflow. Independent review executes in the
Codex session. CI verifies an authorized submitter's exact-packet-bound review
record; this is an attestation, not cryptographic proof of agent execution. A
missing or withdrawn handoff is BLOCKED. No persistent agent is manufactured.
After owner merge, trusted workflow activation is required. No separate AI service
configuration is needed. Published-head CI from the earlier scope cannot stand
in for new-head CI; final publication/check status is explicit in the PR.

Repository rulesets were empty; branch-protection read returned403 (integration
permission), so protection settings are UNVERIFIED. This PR does not edit them.
Only after jobs/statuses exist and have been observed should the owner require
them, with strict branch freshness or merge queue. Per-head statuses cannot
atomically encode a moving parent; invalidation plus strict freshness is required.
No merge, issue closure, deployment, release or baseline approval performed.
