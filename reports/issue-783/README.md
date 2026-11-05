# #783 — Atomic raw-resource ledger

A pure multi-inventory ledger now validates exact integer-gram transfers between
fixed sources, the existing player inventory and explicitly declared ship stores.
Schema v3 saves the whole ledger plus location in one replacement at one revision;
v1 litre / v2 gram migration preserves mass and original bytes until commit.
No store, UI, recipe, capacity or count-item content is enabled in gameplay.

Parent: PR #789, branch `issue-782-browser-docs`,
`10918786422529ca51ed5f95cebc591e3470938a`; prerequisite #757 / #781 verified
in ancestor `e4a9548`. Unique child diff contains ledger, production session/mass
integration, tests, owner docs and this evidence. `source-hashes.json` binds
validation to exact source/test/config/lockfile bytes; the final publication
review records the committed head SHA. Documentation/report-only completion does
not change those runtime inputs.

## Acceptance and actual checks

| Criterion | Result and evidence |
| --- | --- |
| LEDGER-1 | PASS: immutable transfer returns an entirely validated next ledger, or throws; destination validates before debit; global per-material initial mass equals remaining source + player + stores. Exact add/remove APIs preserve inputs |
| LEDGER-2 | PASS: table-driven boundary amounts, gram precision, shared item mass/slots and per-type raw stacking; stores enforce declared gram caps without invented slots. Unsafe integers/overflow and malformed routes/IDs rejected |
| LEDGER-3 | PASS: production ResourceSession real source IDs, one storage call/revision, metadata roundtrip, ten restart/deposit/withdraw loops; complete ledger committed with location |
| LEDGER-4 | PASS: valid v1 and v2 migrate losslessly; failed migration writes, corrupt/unsupported saves, metadata changes, stale revisions and stale bytes preserve storage and block publication |
| LEDGER-5 | PASS: selected-ship stores plus onboard player counted once, other ship excluded, repeated recomputation does not accumulate mass; production TopDownView consumes session payload |
| LEDGER-6 | PASS: 4/4 browser scenarios and all repeat comparisons; production mining rate, interruptions, movement and cockpit behavior preserved by existing suites |

Commands from `client/`:

- `npm ci --cache /tmp/salimon-npm-cache --no-audit --no-fund`: PASS.
- `npm test`: PASS, 15 test files including ledger and production session; `unit.log`.
- `npm run build`: PASS, TypeScript and Vite; `build.log`. Existing bundle-size warning retained.
- `CHROMIUM_PATH=/usr/bin/chromium npm run test:browser -- --project=cockpit`:
  see `browser.log`, `browser-results.json`, original captures and raw attachments.

Initial install attempts failed because network was unavailable in the default
command sandbox and npm's default cache was outside writable roots. Installed
successfully with network permission and a `/tmp` cache. An earlier full test
attempt failed on missing Three/pngjs dependencies during that incomplete install;
no application failure was waived. Final required tests/build use a complete
locked install. No tolerances, masks, baseline files or gameplay constants changed.

## Boundaries and review

Linux cloud, Node v24.19.0; headless system Chromium via Playwright, 1280×800/DPR 1,
en-US/UTC, fixed simulated clock and actual synthetic keyboard/mouse events.
2D SVG CPU rendering; radar WebGL adapter is recorded in cockpit measurements.
New-widget visual qualification N/A; existing repeat captures remain mandatory
and are reviewed with the functional results. No human/hardware gate applies to
browser-local persistence. No M1/Metal/physical-input/backend claim.

Owner contracts: resource-inventory-and-gathering, playable-2d-foundation,
technical-architecture, maintenance-map, current-client-status and client README.
Future composition supplies stable store declarations on load; missing/changed
store metadata fails closed. Raw sources cannot be refilled, stores are flat,
and no raw material is manufactured or consumed by this API. Machines/backend
and equipped-suit policy remain separate future work.

Same implementing agent reviews the unique diff, criteria, source-bound tests,
original captures and raw browser measurements in the active chat. Publication
comment records exact head/base, verdict, CI and limitations. Local review: **ACCEPT** for LEDGER-1–6. No independent
execution is claimed; human retains merge authority. The initial issue-comment approval rejection for #782 was resolved by a read-only
GitHub REST check proving the issue destination, then retrying the same connector.
No approval bypass or alternate posting route was used.

Browser result: **4 passed, 0 failed/skipped/flaky (2.7 min)**. COCKPIT-1,
GATHER-1, GATHER-2 and SLICE-1 passed. Raw save snapshots, handoff measurements,
backend identification and comparator results live in their named directories.
All repeat comparisons passed with unchanged policy; the wrong-checkpoint
negative comparison failed as expected. Original gathering full-weight and cockpit
return screenshots were inspected: existing kg/slot feedback, cutaway and controls
remain legible with no new widget. The cockpit fixture can show the existing
lock-unavailable inventory panel while its paused clock starts; GATHER-1/2
separately exercise the active save adapter and actual tab lock.

Link check: `python reports/issue-782/check-links.py` passed against all changed
guides (final output in `links.txt`); `git diff --check` passed. Broad navigation,
negative mutation, repeat baseline and Docker smoke coverage is delegated to
current-head existing CI, recorded in the final PR review comment. No new CI jobs.

A final committed-diff check found trailing blank lines in link-check output.
Trimmed both reports, propagated the documentation-only parent cleanup and
revalidated the complete diff and unchanged source hashes. Runtime evidence
remains applicable; current-head CI is tracked after this cleanup.
