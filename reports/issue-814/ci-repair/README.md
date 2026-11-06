# CI repair after the owner's failure report

The owner reported failed pipelines for PR #821. Inspected the failed Navigation
client run [42232661938](https://github.com/mr-exception/salimon/actions/runs/42232661938)
for head `4f1b95193bd807d16693873db28ceafc1ac6e32f` and its failed browser step.
The run recorded 58 passed / 2 failed. The skipped later jobs are not accepted.
[Original failed-step log](reported-ci-failure.log).

- HUD-1 timed out expecting 40 contacts, receiving zero. It assumed five domain
  seconds would elapse within its wall-clock wait on the software WebGL runner.
  The fixture now offers a domain-clock advance, split into one-second calls to
  respect NavigationClock's per-call catch-up bound. HUD-1/HUD-3 acquire contacts
  through five real domain seconds before testing their browser UI. Scanner
  cadence, range and authority are unchanged and retain their domain tests.
- LOOK-1 captured the frame observer immediately after clicking Zoom. CI read
  the old 75° FOV, then compared it against the correctly zoomed 27.590958° after
  Look at. The test now waits for the zoomed FOV to be published by the render
  observer before capturing the starting state. The exact preservation assertion
  remains; no tolerance or pixel baseline is weakened.

The first local repair run passed four scenarios but exposed a separate HUD-1
wheel timeout ([local log](initial-local-browser.log),
[context](local-wheel-timeout.md)). Focused forms correctly blocked optical zoom,
but the canvas left wheel events to default browser scrolling. The camera input
adapter now consumes ordinary canvas wheel events before checking form focus;
focused forms still cannot zoom. Ctrl/Meta wheel behavior is preserved. HUD-1
asserts cancellation and unchanged FOV, then exercises the real Playwright wheel.

Changes are limited to fixture/test synchronization and this canvas input guard.
Scanner timing, camera orientation/FOV rules, test timeouts and CI gates remain
unchanged.
The artifact download returned HTTP 403 at GitHub's redirected blob URL; failure
diagnosis uses the available failed-job log. No bypass or remote rerun attempted.

## Verification and review

Final source revision, focused local results and same-session review follow below.
CI for the updated head remains awaiting the owner's report. No CI polling,
merge, baseline approval or deployment performed.

Environment additions: None. Reused installed project dependencies, Node/npm,
system Chromium and the existing WebGL test setup.

Repair source: `d681fb9439f1a0adfa02d69fe990fcdcbb0b24db` on top of original
publication `4f1b95193bd807d16693873db28ceafc1ac6e32f`. The final publication adds
only this report and evidence above that source. LOOK-1 also waits for the
render observer to publish manual alignment cancellation before asserting it.

- `cd client && npm run build`: PASS; [log](build.log). Existing bundle-size
  advisory remains unchanged.
- `node --experimental-strip-types --test src/scanner.test.ts src/contactCommands.test.ts`:
  PASS (2 test-file entries); [log](domain.log).
- `CHROMIUM_PATH=/usr/bin/chromium SALIMON_EVIDENCE_DIR=test-results/issue814-ci-repair-final npm run test:browser -- --project=panoramic --project=look-at --project=observation`:
  PASS (10/10, 2.6 minutes); [log](browser-final.log),
  [results, original captures and raw measurement index](evidence/results.json).
  This includes real canvas wheel while form-focused, unchanged view/ship,
  telescope bounds/zoom, camera independence, all compass states, keyboard
  cancellation, resize/DPR and repeat/negative pixels.

Same-session source/evidence review by the implementing Codex agent. Inspected
the failed CI log, local wheel-timeout trace/context, final input guard and test
changes, raw states and representative original captures. The fixture advances
the existing domain clock without assigning contacts or bypassing range/cadence;
frame-readiness waits preserve exact assertions. Wheel cancellation still
precedes any optical mutation while a form owns input. The 10-scenario matrix
confirms ordinary zoom and focus cancellation. Previous visual evidence remains
applicable to unchanged rendering; fresh captures and measurements are indexed
above. No visual baseline was edited or approved.

Local review PASS after the recorded fixes; no unresolved local finding.
**QA pending / updated CI UNVERIFIED**. The original head is known to have failed;
acceptance of the repaired head requires the owner's new CI report. Existing
CI functional, negative, repeat visual and baseline-approval gates are intact.

Fresh-page repeat verification of the two original CI failures:
`CHROMIUM_PATH=/usr/bin/chromium SALIMON_EVIDENCE_DIR=test-results/issue814-ci-repair-repeat npm run test:browser -- --project=panoramic --project=look-at --grep 'HUD-1|LOOK-1' --repeat-each=2`:
**PASS 4/4** (1.5 minutes), [log](repeat.log),
[per-run results and canonical raw evidence](repeat-results.json).
Identical PNG originals are retained once and referenced by both result indexes.

LOOK-1 measurements confirm 75° initially, 27.590958087858176° after Zoom and
exactly the same FOV after Look at; no page errors and byte-identical occlusion
control. HUD-1 retains 40 scanned IDs, keyboard traversal and a single final exit;
its real wheel event completes without mutating view/ship during form focus.
