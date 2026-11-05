# #757 — whole-litre gathering and local expedition saves

Implemented against `main` at `24277ba71c1519188f6401266f8794d7803839b0`.
All #761–#767 prerequisites are completed and their delivered world, sources,
doorway and cockpit paths are present. No active PR stack existed at preflight.
Branch: `issue-757-resource-gathering`, base: `main`.
Tested implementation: `83d79ea49a3e15857e96664e73d89cca688fd0cd`.
This report/log successor changes no runtime/test/config inputs; hashes are in
[`evidence/source-inputs.sha256`](evidence/source-inputs.sha256).
The published PR comment binds final head/base and immediate CI status.

## Result

The existing continuous world now has an equip/stow gathering tool and **E / Gather**
action. Its two stable sources collect directly into the player inventory at
one whole litre per 60 simulation ticks. Exact integer grams/L enforce weight and
volume independently. HUD stacks, density/remainder, progress and success/full
feedback expose the result. Depleted objects retain IDs/colliders and dim visually;
no fragment, carrying or second world runtime is added.

`resourceSession.ts` commits source depletion, inventory/capacities and exact
frame-relative location in one versioned localStorage replacement before publishing
credit. A lifetime Web Lock prevents concurrent tab writers; byte/revision checks
reject stale writes. Corrupt/unsupported/nonconserving saves remain untouched and
disable gathering. Reload restores standing with the tool stowed; interrupted
partial seconds never resume. Cockpit saves restore beside its chair. Payload
grams contribute once to the ship while the player is aboard; mass display keeps
gram precision. The resource/client/playable/architecture guides describe ownership
and limitations.

## Verification and same-session review

Reviewer: implementing Codex assistant, source/diff, actual unit/integration/build
logs and original desktop/narrow preview inspection. No independent reviewer or
physical/GPU execution is claimed.

| Criterion / category | Evidence and actual status |
| --- | --- |
| R1 independent 10 kg / 10 L, mixed resources, fractional kg and exact remainders; unit | **PASS:** `resourceInventory.test.ts` (9 cases), including 333 g/L and overflow; `resourceSession.test.ts` actual-world mixed/weight/volume cases |
| R2 shared-world tool/source interaction and clear feedback; integration/browser | **PASS** for domain composition and two layout previews. **UNVERIFIED** for complete built browser gameplay until CI `GATHER-1`/`GATHER-2` |
| R3 no duplicate/fractional material on interruption, repeated action or reload; integration | **PASS:** 12 session cases cover atomic failure, stale writers, partial ticks, reload, upgraded capacities, malformed saves and exhaustion. Real-browser reload/tab-lock proof remains **UNVERIFIED** pending CI |
| R4 resource mass counted once; integration | **PASS:** repeated ship-mass recomputation, board/leave frame ownership and exact payload tests; actual navigation regression remains in CI |
| Build/typecheck | **PASS:** `npm run build`; existing large-bundle warning retained, no lockfile/dependency change |
| Browser/visual regression, negative and repeat checks | **NOT RUN locally**, per user instruction to leave e2e to CI and not wait. New cases are included in the existing cockpit project/workflow; current-head CI is pending |
| Visual originals | **PASS for layout review only:** desktop 1280×800 and narrow 390×844 PNGs, DPR1, CPU SVG/image rendering; zero page errors or horizontal overflow. No approved navigation baseline changed |
| Docker smoke | **NOT RUN locally**; unchanged Docker workflow will build the updated client in CI |
| Physical input, M1/Metal, mobile controls, real GPU | **NOT RUN**; synthetic desktop input/software previews do not qualify hardware |

**Verdict: BLOCKED for final acceptance / QA pending required browser and visual CI.**
No blocking source finding remains after the same-session review, but unrun
mandatory gameplay/negative/repeat checks are not called PASS or waived. The user
explicitly requested not to wait for e2e completion; no watcher or local e2e
campaign was started. Keep #757 open until accepted delivery is merged.

Commands executed from `client/` unless stated otherwise:

```sh
npm ci --cache /tmp/issue-757-npm-cache --no-audit --no-fund
npm test
npm run build
node --experimental-strip-types src/resourceInventory.test.ts
node --experimental-strip-types src/resourceSession.test.ts
npm run preview -- --host 127.0.0.1 --port 4175 --strictPort
# Repository root, against the built preview:
node reports/issue-757/evidence/preview.cjs
git diff --check
```

Full `npm test` reports **14/14 test files passing** (the focused 9 + 12 cases
are part of that coverage, not additional suites). Raw logs are retained under
`evidence/`. Environment: managed Linux x64, Node 24.19.0, npm 11.9.0,
Chromium 154.0.8037.57; preview metadata/input/backend is in `preview.json`.
CI uses the repository's Node 22/locked Playwright Chromium configuration.

## Failures, originals and limits

The first narrow preview exposed caption/tool overlap; the original is retained
in `failures/field-inventory-narrow-overlap.png`. Responsive flow was corrected
and the final narrow/desktop originals reviewed. The first inventory test edit
also had an incorrect expected value (10 versus 2 L for a 2 L source); corrected
expectation and full final checks pass. Initial npm install could not write its
default cache; using the task-local cache succeeded without dependency changes.

Persistence is single-browser/same-origin, requiring HTTPS or localhost and
available browser storage. A hard interruption may lose movement since the last
one-second/stop checkpoint, never half a material transfer. This is not server
authority, cloud sync or resistance to intentional save deletion/editing.
Sample densities/amounts and interruption semantics are documented explicitly;
upgrade UI/costs, machine transfers/Core/Fabricator, multiplayer and balance tuning
remain future work. Walking has no separate mass-based physics. No merge or
deployment is performed.
