# #513 — private-development source decision and production gate

2026-10-22. The owner explicitly chose **private development only** in this task
conversation. [Recorded decision/input inventory](private-development-decision.json)
pins the exact manifest SHA-256 and all source paths/sizes/hashes. No Gaia-derived
production release, commercial use, scientific-source replacement, permission
request or rights approval was performed.

## Delivered

Keep Gaia's synthetic fixture and actual small JPL Earth / NASA PS Proxima
snapshots unchanged. Gaia remains restricted, JPL/PS unreviewed. Historical source
policy links and credit guidance remain source-specific; their text is not
invented exact approved attribution. The owner scope decision is not provider
permission or legal approval, including for contributed data.

Close an importer gap: toggling reuse/origin flags alone no longer allows
production staging. `--production` additionally requires a human review sidecar
bound to the exact manifest, source IDs, intended use/distribution, full terms
and credits, permission where required, full selected snapshot scope and
acquisition/epoch/frame. Output preserves the exact manifest/review and verifies
again before atomic publication. Development import remains available offline.
See the [canonical record contract](../../docs/universe-sources.md#private-development-decision-and-production-evidence-513).

## Validation

- `python -m unittest discover -s scripts -p 'test_*source*.py' -v`: passed
  ([log](source-tests.log)); socket-free development builds, byte tampering,
  duplicate/unsafe/reserved paths, production flags without review, changed
  manifest binding, incomplete terms/credit/permission and fixture scope reject.
- Actual default development staging passed; actual `--production` rejected the
  unchanged restricted manifest before publication.
- Documentation paths/anchors and whitespace checked. Rust metadata/schema,
  scientific adapters and input bytes are unchanged; owning
  [architecture](../../client/universe/architecture.md#source-manifests--u06-407)
  and [invariant 12](../../client/universe/invariants.md) remain applicable.

## Pending #513 acceptance

Full current provider terms were not fetched: the configured network allows
package hosts, not ESA/JPL/IPAC. The prior 2026-10-21 full Cosmos fetch returned
451; that historical observation is not a current fetch. No exact approved
credits, new permission record, reviewed full production inputs or production
eligibility are claimed. U07/U12 production remains blocked. The production
record validates completeness/binding, not legal truth or actual survey coverage.

This PR stays draft while #513's full acceptance is pending. The current private
scope permits continued fixture development; before broadening use, review the
actual full terms and acquire/freeze full selected snapshots and permission
as required. #513 retains those requirements; no pending gate becomes a pass.
