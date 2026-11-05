# Issue #786 — weighted count items

Outcome: generic item registration, atomic item add/remove and unified field
inventory are implemented. Production `ITEM_CATALOG` remains empty: no gear,
weights, grants, auto-equip or extra suit slots are introduced.

## Revisions and scope

- Tested source: `f265b9d2d183132bf015fab607e993337a0bbbfa`.
- Branch: `issue-786-count-items`; base `main` at
  `981c510b6c600584f17cd2fa05f217040f19f426`.
- No active PR existed at branching. #783 is closed/delivered by #790;
  source/store conservation is present. #784/#785 are also merged (#794/#795).
  The formal blocked-by endpoint returned no edges; #783 is the explicit textual
  prerequisite. Current main is a verified ancestor, with one unique issue diff.
- `resourceInventory.ts` validates/freeze-copies all trusted item definitions;
  `resourceSession.ts` routes revision-bound exact add/remove through the existing
  atomic save boundary. Schema v5 binds saved definitions to trusted metadata.
  Valid v1–v4 empty-item saves migrate losslessly; corrupt/unknown definitions,
  unknown IDs, invalid quantities and unsupported migrations preserve original
  bytes and block writes. Changed authored catalogs need an explicit migration.
- `TopDownView.tsx` displays counts, per-type item mass/slots, raw stacks, combined
  kg/slots and the empty item state. Gathering, ship payload and Resource Container
  capacity preview/panel use the same session catalog.
- `tests/items.html` is an explicit non-shipping fixture; its synthetic definitions
  and controls are outside the production entry. The built bundle contains none
  of `fixture-single`, `Single fixture`, `Add two singles` or `items.tsx`.
- Canonical inventory, current-client, architecture, maintenance, playable-journey and client guides
  were updated. Core, crafting, world item spawns, equipment and consumables remain
  outside this issue.

## Verification matrix

| Category / criteria | Command / observable assertion | Result | Evidence |
| --- | --- | --- | --- |
| Unit — ITEM-1–3, ITEM-5 | `cd client && npm test`: count/stack=1 and >1, exact mixed grams, shared slots, invalid registration/request, overflow/removal, migration and no failed source debit | PASS, 106 tests | [unit.log](evidence/unit.log), `resourceInventory.test.ts`, `countItemSession.test.ts` |
| Integration — ITEM-3–6 | Same test command: real `ResourceSession`, atomic quota/stale failures, definition/count corruption, repeated reload, migrations and onboard item payload | PASS | [unit.log](evidence/unit.log), `countItemSession.test.ts`, existing session/store tests |
| E2E — ITEM-4–6 | `cd client && npm run build && npm run test:browser`: real session/HUD under Web Lock, fixture add/remove, exact reload, rejected overweight bytes, normal built gathering and cockpit | PASS, 39 browser tests | [build.log](evidence/build.log), [browser.log](evidence/browser.log), [browser-summary.json](evidence/browser-summary.json) |
| Visual — ITEM-4, ITEM-6 | `items.spec.ts`: empty, raw, mixed and overfill originals; two replays under existing pixel policy; wrong-checkpoint negative; narrow 390×844 layout | PASS | [original PNGs](evidence/items-mixed-0.png), [ordinary empty](evidence/ordinary-empty.png), [journey state](evidence/items-journeys.json), repeat JSONs in `evidence/` |
| Manual/hardware | N/A: no gear selection, wearable physics or physical-input claim | N/A | Synthetic Playwright keyboard/mouse; software rendering only |

Required Navigation client CI, negative mutations, visual repeat and unchanged
baseline approval are bound to the published head in the final PR review comment.
No approved baseline is changed or promoted. Pending checks are not passes.

## Criterion review

| Criterion | Result | Evidence inspected |
| --- | --- | --- |
| ITEM-1 | PASS | Stack=1 copies, `ceil(count/4)`, raw+item shared slots; exact freed slot on 5→4 removal in unit and browser tests |
| ITEM-2 | PASS | 1,000 raw g + 250 single g + 1,250 stacked g = 2.5 kg; zero/fractional/overflow definition and overfill rejection |
| ITEM-3 | PASS | Exact add/remove/reload; v1/v2/v3/v4 migrations; quota/stale/corrupt-save byte preservation in session tests |
| ITEM-4 | PASS | Original empty/raw/mixed/narrow captures and real HUD count/kg/slot assertions |
| ITEM-5 | PASS | Combined slot and mass gathering boundaries (including exact 250 g remaining credit); failed adds leave source untouched; built gathering regressions |
| ITEM-6 | PASS | Non-shipping HTML mounts production session/HUD; built ordinary entry has no items/controls/query backdoor; fixture strings absent from bundle |

## Environment and evidence

Linux 6.18.57 cloud; Node v24.19.0; locked dependencies installed with `npm ci`;
Playwright Chromium 156.0.8078.4. 2D SVG/sprite captures use Chromium CPU
rasterization (`--disable-gpu`); cockpit/radar use ANGLE SwiftShader software
WebGL2. No hardware GPU or physical input qualification is claimed. Main viewport
1280×800/DPR 1; narrow 390×844/DPR 1; locale en-US, UTC browser timezone, dark,
reduced motion; vendored reference fonts. The world journey clock starts at
2026-11-04T00:00:00Z and advances fixed ticks through the production input path.
Existing seeded navigation scenarios keep their own authored seed/clock.

`evidence/source.sha256` binds source/test/assets/configuration to the tested source.
The final documentation/evidence-only commit reuses those unchanged runtime/test inputs. Final head/base,
CI links and same-session review are recorded after publication.

Initial typechecking identified a test-storage helper call signature mismatch;
it was corrected before final verification. No final failures are waived.

## Review in this chat

Reviewer: implementing Codex agent, same active chat; not an independent agent.
Final diff, original captures and actual criteria/check evidence are reviewed
before acceptance. Local criterion review: PASS for ITEM-1–6; no blocking code/visual findings.
Full local browser suite: 39/39 PASS. Final ACCEPT remains pending required
current-head CI and is recorded in the published PR review comment. The issue stays open
until owner-authorized merge; no merge, deployment or baseline approval is made.
