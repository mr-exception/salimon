# #785 — Playable Resource Container transfers

The existing reachable `scout:storage` anchor now opens a native keyboard/mouse
modal in the built default world. It displays both inventories in fractional kg,
10 kg/12-slot starting player limits, 50 kg container capacity and exact weight
progress. Players enter positive whole grams, review a deposit/withdrawal and
confirm; only ResourceSession transactions publish material changes. Confirmation
is consumed once, while editing/closing discards only draft state.

Parent: non-draft PR #794, `issue-784-resource-container`,
`8f9a92498cc07e9ea9c118b9840b3627f0a92096`, verified ancestor.
Transitive #783 is merged in main `cf46092`. The unique child diff owns UI/input,
controller, tests, owner docs and this evidence. Tested source:
`8c9de7985bab03d7a8531254f464829f8b1f0e3b`; source-hashes.json covers
runtime, tests, lockfile and configuration. Final report-only publication preserves
those inputs. Child PR targets the parent head branch, never main.

| Criterion | Actual evidence / result |
| --- | --- |
| UI-1 | PASS in final built STORE-UI-1: actual default spawn → mine 1000 g → walk cargo → deposit 333 g → withdraw 111 g → exit/reload, twice. Player 778 g + container 222 g + source 59000 g = 60000 g. Final test also asserts simulation keeps advancing with the panel open |
| UI-2 | PASS: real controller/session tests for invalid/unknown/insufficient, both weight caps, stale confirmation, quota and stale bytes. Browser full-container/player-overweight/malformed/stale-save failures preserve exact prior state. Equipped gathering tool + selected container opens storage, no mining; duplicate Enter cannot repeat a consumed confirmation |
| UI-3 | PASS: production controller tests movement, range, wrong target, seated/radar transitions; browser Escape/focus-loss/resize/Close/draft cancellation and restored walking. Text input retains WASD/E; commits survive cancellation/reload |
| UI-4 | PASS: integer-gram drafts and domain commits; exact 333/111 g roundtrip, 0.778/0.222 kg text, progress value=222/max=50000, remaining capacity and slot feedback. No rounding influences commands |
| UI-5 | PASS for final panel originals/repeats; broader existing regressions pending current-head CI: originals for open/transferred/error/full/save-error/narrow states; new repeat comparisons and wrong-checkpoint negative. Existing unaffected cockpit/camera/collision/approved navigation comparisons use full current-head CI |

## Verification plan and executed commands

Commands from repository root (npm --prefix client runs the client scripts):

- `npm --prefix client test`: PASS, 17 test files (`unit.log`), including real
  ResourceSession/domain integration and controller/slot feedback.
- `npm --prefix client run build`: PASS, TypeScript/Vite (`build.log`); existing
  large-bundle warning remains. No dependencies/lockfile changes.
- `CHROMIUM_PATH=/usr/bin/chromium SALIMON_EVIDENCE_DIR=/tmp/issue-785-storage npm
  --prefix client run test:browser -- --project=cockpit --grep STORE-UI
  --timeout=360000`: initial PASS, 2 tests (1.7 min), three panel repeat comparisons.
- Same focused command with `/tmp/issue-785-final-storage` after the simulation
  continuation assertion: PASS, 2 tests (2.1 min). A further affected rerun after
  the reviewed arrival-checkpoint/target-ownership fix uses `/tmp/issue-785-reviewed-storage`: PASS, 2/2 (2.2 min). Final source/test inputs
  are source-hash bound. The larger local execution budget retains every
  assertion/tolerance; existing CI still uses the unchanged 180-second project budget.
- Existing current-head Navigation CI executes `npm test`, `npm run build`, full
  `npm run test:browser`, navigation-negative, visual repeat, visual-negative and
  baseline approval. Docker smoke is required too. These broad checks run once
  in CI rather than duplicating the parent's unchanged navigation campaign locally.

Linux cloud, Node v24.19.0/npm 11.9.0; system headless Chromium, 1280×800/DPR 1,
en-US/UTC and controlled 2026-11-04 clock. SVG CPU rasterization; existing radar
uses ANGLE SwiftShader (proved by parent production preflight/current-head CI).
Actual Playwright synthetic keyboard/mouse actions exercise the DOM adapter;
focus loss is dispatched through the production window listener. Narrow-screen
sanity uses 390×844; this is no touch/physical-input/hardware qualification.

## Review, originals and limits

New UI originals are review candidates, not promoted references. Existing approved
navigation files, thresholds and masks are unchanged. Initial browser scenarios passed; same-session review then reproduced a location
checkpoint racing a draft confirmation (revision 1 → 2, false stale rejection).
The controller now checkpoints arrival before opening a draft, and a regression
test proves the next autosave leaves the revision stable and confirmation succeeds.
Closing an old interface also preserves a newer target interaction. These fixes
receive fresh affected unit/build/browser checks. No failure was waived; exploratory commands with a wrong working-directory path
failed before running checks and were corrected. The source-bound final checks
below establish acceptance. Parent report retains its separate initial cockpit
timeout and successful rerun; it is not hidden by this child.

The current two-resource/empty-item catalog cannot fill 12 slots in real browser
play. Slot error text and exact shared-budget boundaries are verified by the
production helper with validated synthetic usage; #784's 13-resource ledger
fixture separately proves the actual shared slot rejection and uncapped store.
No new catalog, count-item content, machine buffers, upgrades, source catalog or
backend/multiplayer authority is introduced. Manual/hardware category N/A for
routine browser-local acceptance; no physical/Metal/native release claim.

Canonical resource/client/playable/status/maintenance/architecture guides are
updated. Link checker and `git diff --check` pass. Essential originals, raw save
snapshots/comparator measurements, logs and source/evidence hashes are retained.
Same-session implementing-agent review inspects exact parent/head, criteria,
unique diff, originals and actual check results; final verdict/current-head CI
are published to the PR and source issue. No independent review is claimed.
Human retains merge authority. Merge #793, then #794, then this child; source issues stay
open until delivery reaches main.

## Final local evidence review

Same-session implementing-agent review found and fixed the arrival checkpoint
race and newer-interaction cancellation. All affected checks now pass. Final
browser originals show open/transferred/error/full/save-error/narrow states;
three repeat comparisons pass with zero changed pixels beyond the existing
perceptual threshold, and a wrong-checkpoint negative rejects the comparison.
Original transferred/full/narrow captures were inspected for text, weight/slot
clarity, control reach and overlap; no material visual finding remains.

The final browser run executed at runtime/source `1b669725290b708d305ff157fbaf175d4d053fee`.
Commit `cb576c6` changes only the unit slot fixture to derive real shared usage
from synthetic count-item definitions. Production, browser tests/assets/config
and lockfile bytes are unchanged; `browser-evidence-reuse.txt` records this diff.
The updated controller file passes direct Node execution (7 tests;
`controller-final.log`), and the final TypeScript/build passes. The earlier full
17-file suite is reused for unchanged units; current-head CI reruns the complete
suite. No broad local campaign is repeated solely for report packaging.

Local review: **ACCEPT** for new functionality and UI evidence; final overall
acceptance remains pending the existing full current-head CI checks and is bound
in the PR/issue review comment. No independent execution is claimed.

## Stack refresh

Documentation PR #793 appeared during implementation. Parent #794 now incorporates
its `bf1cb0d` history and targets that branch. This child incorporates updated
parent `8f9a924` non-destructively. Runtime/test/config/assets/lockfile hashes remain
unchanged from the verified source; existing local evidence is reused after hash
comparison, and current-head full CI is required again. The migration register
now identifies playable container transfers and preserves future machine scope.
Final review binds exact refs/checks; prior acceptance is superseded after the
parent change until revalidation completes.

## Full CI finding and shared viewport repair

CI 42033453722 passed the storage, gathering, cockpit and full journey scenarios
(36 browser passes) but failed existing TD-1 byte-equality. Parent CI 42033304505
failed the same gate: both originals differed at exactly eight bottom-left
rounded caption pixels. Original PNGs, exact RGB deltas and failure output are
retained under `failures/ci-42033453722`. No assertion failure is waived.

An exploratory lazy panel-loading hypothesis passed one local TD-1 control but
was disproven by the parent failure; its controlled-clock modal checks failed.
That trial was reverted, not shipped or counted as a successful check. The final
interface retains its previously validated direct loading. The shared fix is
owned by parent #794: give the translucent caption its own 7 px inner corner
curve matching the 8 px stage minus its 1 px border. No art, approved reference,
assertion, mask or threshold changes. Eight replay probes and three unmodified
TD-1 fresh-run repeats passed; parent report retains originals and measurements.

This child incorporates parent `8f9a924`. Only `client/src/topDown.css` changes
from prior verified UI source; gram/controller/browser-test/config/lockfile inputs
remain unchanged. Build/typecheck and affected built storage scenarios are rerun
against the merged renderer fix (`8c9de7985bab03d7a8531254f464829f8b1f0e3b`).
Prior exact-head acceptance is superseded until current-head full CI and active-chat
review pass. The final publication comment binds exact head/base and results.

Final affected checks after shared clipping fix: **PASS**, 2/2 built storage
scenarios (1.7 min), all three repeats and wrong-checkpoint negative, with original
captures/raw saves under `clip-browser`. New build/typecheck passed. Original
transferred/full/error/narrow views retain the same legible UI; only caption corner
coverage is stabilized. Pure domain/controller/Node inputs remain unchanged;
`clip-evidence-reuse.json` binds their earlier passing evidence to the final
source, while current-head CI reruns the full suite. Local re-review has no
remaining material finding; overall acceptance awaits final-head CI.
