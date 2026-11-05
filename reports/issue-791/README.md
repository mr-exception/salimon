# Issue #791 — full 3D-to-2D documentation migration

**Scope:** documentation-only. Baseline: `main` at `cf46092f111061efb80d1d608396ad18fc71ea7f` (merged #789 browser status and #790 resource ledger). Branch: `issue-791-2d-doc-corpus`. The parent stack merged during this audit; the new work was committed on that exact file tree with no rewrite of the older PR branches.

## What changed

- Audited **106 existing `docs/*.md` files** and **40 other repository Markdown files**, including root/client guides, templates, visual baseline guides and preserved issue-report evidence. The new `docs/2d-migration-register.md` includes **107 file-by-file rows**: 6 active browser owner/contracts, 12 current guides (including this register), 22 future design guides, 61 historical native implementations and 6 archives/evaluations.
- Marked historical Rust/wgpu, first-person/mesh, scientific catalog and cargo/ship module implementations clearly as **revision-specific native evidence**, not shipped React code. Kept the underlying approved numerical/gameplay and narrative contracts, changed no app logic or source data, and retained historical reports.
- Defined the mapping from a physically real/canonical universe to **strict illustrated top-down Scout/planet gameplay** plus **cockpit-only 3D radar**. Added actionable boundaries for fixed footprints/occlusion, ship/cargo interactions, integer-gram inventory, tierless suits, 200 HP hazards, ship-wide 2D damage, legacy saves, and unported power, Fabricator, planetary world and multiplayer.
- Updated root/client/docs entrypoints, coding conventions, phases, current status and eight core design files. Corrected the radar guide's stale pre-integration wording.
- Repaired removed-source Markdown links across the document corpus to verified file paths in historical commit `8f112bb6e6be9f813174efb47d706a8601930d04`; did not invent current paths or replace archived source references with nonexistent TS files.

## Source-to-document checks

| Area | Current checked-in browser owner / scope | Result |
| --- | --- | --- |
| 2D world, frames, cutaway and interactions | `client/src/topDownRuntime.ts`, `topDownFrames.ts`, `topDownTraversal.ts`, `topDownInteraction.ts`, `TopDownView.tsx` | Current delivered; not the native first-person world |
| Radar cockpit | `client/src/cockpit.ts`, `navigationRuntime.ts`, `App.tsx` | Local seat and 3D radar delivered; power fixture only, no online pilot |
| Raw sources, inventory and save | `client/src/resourceInventory.ts`, `resourceLedger.ts`, `resourceSession.ts`, `planetarySite.ts` | Gram ledger v3, 10 kg / 12 slots, sample mining; no playable cargo store UI |
| Suits, hazards, HP, Core/Fabricator | Approved `docs/` contracts, visible machine inspection placeholders | Not shipped; preserve 200 HP, suit tiers/slots and raw-buffer rules without invented weights/recipes |
| Canonical planet data and multiplayer | `sampleUniverse.ts` and authored landing site only | Sample radar/landing area; native canonical streaming, server authority and reconnect not migrated |

## Verification record

| Criterion | Actual evidence | Verdict |
| --- | --- | --- |
| MIG-1: every docs file covered | Read 106 original docs; new register accounts for **all 107 `docs/*.md` entries** with no missing/extra table rows on final Git tree | PASS |
| MIG-2: legacy implementation explicitly scoped | Added document-level native/approved-future status markers where needed; 61 native implementation notes separate archived Rust and 2D runtime | PASS |
| MIG-3: approved 2D mechanics preserved | Source/owner check above; current contract tables retain 10 kg, 12 slots, no volume, 50 kg proposed raw container, 200 HP, suit slots, 3D radar isolation and ship-wide damage | PASS |
| MIG-4: link reconciliation | **1,251** local or pinned-archive Markdown destinations checked across 107 docs against current and archived Git trees. Initial pass found 4 legacy links in coding-conventions; repaired and rechecked. **Zero unresolved destinations**. Additional root/client/entrypoint check: zero unresolved | PASS |
| MIG-5: documentation-only, reviewable diff | Compare against the merged `main` tree; source/gameplay/tests unchanged. Semantic source-owner check plus same-chat implementing-agent review | PASS for documentation; CI not yet known |

**Verification method:** Through the connected GitHub repository, enumerate the exact current and pinned historic recursive Git trees and read every Markdown file. Parse local inline link destinations, resolve them relative to each owning document, validate against the current tree; for historical pinned `blob/tree` links, validate the path against the archived tree. Validate each current `docs/*.md` path has a register row. This is a deterministic connector-backed repository audit, not a locally run `git diff --check` or a claim that external URLs/fragment headings were opened. Two pre-existing trailing-whitespace lines in the historical `phase-0-evaluation.md` remain untouched; they are not added diff lines.

**Not run / N/A:** Node unit/build/Playwright, visual capture/negative/repeat and hardware/M1 tests were not rerun because no runtime/test code or rendering changed; any GitHub CI triggered by README paths must be evaluated on the final PR head. No local checkout is available in this session, so shell `git diff --check` was not executed. External website availability, anchor correctness and reference-style Markdown links were not exhaustively validated.

## Same-session review and limitations

**Review mode:** same implementing assistant in this conversation; no independent reviewer or reviewer service. Reviewed the unique scope/diff, full migration register, archival link validity, actual browser source owners and status-supersession precedence. **Verdict: ACCEPT for documentation scope**, subject to any required final-head CI. The repository owner reviews and merges; no automatic merge or issue closure was performed.

This PR **does not implement** the native features in TypeScript or finish decisions for full planetary streaming, gear weights/recipes, Core power, ship damage physics, multiplayer or endgame. Detailed historical instructions remain deliberately readable under supersession notes, not silently rewritten into invented current mechanics. Their future 2D implementation requires separately scoped issues and production acceptance.
