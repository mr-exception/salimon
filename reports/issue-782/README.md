# #782 — Browser-first documentation reconciliation

Delivered documentation-only routing and supersession markers. Parent: `main`
`e4a9548` (merged #781); no active PRs or prerequisites. Source audit: this
parent's `client/src`, unchanged by this PR. Linux cloud, Python 3, Git; no runtime
or approved gameplay-number changes.

## Evidence and review

| Criterion | Result / evidence |
| --- | --- |
| DOC-1 | PASS: root and docs index → current-client-status → playable journey, client setup, active owners and pinned native archive |
| DOC-2 | PASS: `python reports/issue-782/check-links.py`; all changed guide links resolve locally or to verified `git cat-file` historical paths. Broken reports index restored; five removed models links point explicitly to the existing pinned historical tree |
| DOC-3 | PASS: reconciliation table in current-client-status maps all eleven requested systems to current source owners and separates delivered, legacy, approved future and pending decisions. Inspected resource session/inventory, TopDownView, traversal, cockpit, layout, simulation, scanner, sample universe and site owners |
| DOC-4 | PASS: original historical detail preserved; explicit supersession at native sections and localized damage. Retains 10 kg / 12 slots, 1 kg/s, 50 kg proposed raw buffers, ship-wide damage and future 200 HP design |
| DOC-5 | PASS: current-client-status and validation route Node/browser/cloud checks; native/M1 context explicitly historical/separately owned |

First link-check run found one missing historical report anchor; added the archive
section and rechecked. Exact final link count is in `links.txt`.
`git diff --check` passed. E2E, visual and hardware: N/A, no runtime/UI changes.
The offline checker verifies pinned historical file paths, not live external URL
availability or every historical heading. Semantic accuracy is assessed by the
source reconciliation, not inferred from link existence.

Same implementing agent reviewed criteria, unique diff and actual link results in
the active chat: **ACCEPT**. Publication comment binds this review to final head
and base SHAs. CI path filters do not require gameplay campaigns for this docs change.
No independent review or hardware execution is claimed. Human retains merge authority.
