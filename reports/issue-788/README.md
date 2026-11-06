# Issue #788 — deterministic hazard domain

Implemented the specified temperature/radiation/poison subset in the production
200 HP survival/session/save path. Normal gameplay declares no sources or modules.
No wearable equipment, unapproved item masses, planetary strengths or fire rules.

- Base: `main` / `73382e8408f0b7aa1d932d686ed85560845da4da` (merged #800 / prerequisite #787; #783 also merged). No active PR stack existed at discovery or publication preflight.
- Source branch: `issue-788-hazard-domain`; tested source: `c5fcb6a6e7d067ff4ce57e6825bbf04481134664`.
- Final PR head/base and current CI/review are bound in the PR review comment. Report-only commits reuse unchanged runtime/test/configuration inputs from [hash manifest](verified-inputs.sha256).
- Environment: managed Linux 6.18.57 x86_64, Node 24.19.0, npm locked install, Playwright 1.64.1, Chromium 156.0.8078.4. Browser fixture is a separate production Vite build on 4275; ordinary built game on 4175. 1280×800, DPR 1, UTC controlled clock starting 2026-11-04; Playwright synthetic buttons/keyboard. SVG fixture uses CPU rasterization; no hazard rendering or physical-device claim.

| Criterion | Result and evidence |
| --- | --- |
| HAZ-1 | PASS: rational per-source decimal addition, protection once per type, fractional HP, independent types, source permutation, module ID allocation and tick partition tests |
| HAZ-2 | PASS: 4/24/44°C safe with 2 HP/s; 44.1/23.9°C fractions; mid-tick 100-point threshold; independent 5+protection recovery; residual reentry; exit stops damage |
| HAZ-3 | PASS: exact 1,000 HP exhaustion, one debit at full exposure, partial-tick destruction, no later recovery bonus; destroyed IDs survive reload/reset |
| HAZ-4 | PASS locally: one survival clock, exact health/exposure/budget schema v7, v1–v6 migrations, duplicate/older tick guard, pause/reload, lethal ordering, failed/stale writes freeze at committed revision |
| HAZ-5 | PASS: explicit synthetic declarations only; separate fixture build and normal-build absence/query test; no production source/module defaults |

| Category | Exact command/procedure | Actual result |
| --- | --- | --- |
| Unit + integration (mandatory) | `cd client && npm test` | PASS: 124/124; [original log](unit.log). Includes real ResourceSession + storage failure/stale injection, exact reload, migration and terminal reset |
| Build (mandatory) | `cd client && npm run build` | PASS: TypeScript + Vite; [original log](build.log). Existing large-bundle advisory remains |
| Focused built E2E | `cd client && npm run test:browser -- --project=cockpit hazards.spec.ts` | PASS: 2/2; [log](browser-focused.log), [original results](browser-focused-results.json), [original numerical records](hazard-records.json). Build uses same runtime inputs as tested source; later added tombstone unit test changes no browser inputs |
| Full browser / existing CI gates (mandatory) | Navigation client workflow: full `npm run test:browser`, navigation negative, visual repeat/negative, baseline approval | Pending at initial publication; current results linked in PR review comment |
| Visual | N/A to new domain: no hazard presentation. Existing navigation visual/repeat/negative gates retained in CI | No baseline promotion; automatic final captures retained as [fixture](fixture-final.png) / [normal game](normal-game.png), not approved baselines |
| Manual/hardware | N/A to deterministic browser-local domain; real planetary sampling, balancing and hardware performance outside this issue | No qualification claimed |

Initial iteration found an unconnected damage callback, old-schema test fixtures
retaining the new hazards key, strict TypeScript errors and two incorrect test
expectations. Those were corrected before the passing source checks above.
The first shell test attempts also used the repository root instead of `client`;
they exercised no code. No failure was waived or baseline changed. The branch
protection API returned HTTP 403 (integration lacks access); the existing workflow
checks remain the observable acceptance gates, not a claim to have audited hidden
repository rules.

Changed owners: `exact.ts`, `hazards.ts`, `survival.ts`, `resourceSession.ts`;
colocated tests and the separately built `tests/hazards.*` fixture/config. Save
schema v7 retains the existing key and ledger. Hazard progress saves before
publication, including partial seconds; storage failure cannot publish a free
shield interval or erase committed exposure. It currently saves once per admitted
hazard advance; future optimization must preserve that guarantee. Destroyed
module tombstones remain through explicit character reset. Sampled live sources
are not restored as planetary facts.

Canonical contracts: [browser hazards](../../docs/browser-hazards.md), updated
browser survival/design status, maintenance routing and client usage. Deferred
scope remains fire/acid unspecified rules, pressure/oxygen/gravity, equipment UI,
bench/crafting/storage weight caps, multiplayer and planetary sampling.

Review: implementing Codex agent in the same active chat; no independent agent or
execution claimed. Local criterion/source/evidence review found no remaining
blocking implementation finding. Final ACCEPT requires current-head mandatory CI;
the final revision-bound verdict is published in the PR comment. Issue stays open
until owner merge. No merge/deploy/release authorized.
