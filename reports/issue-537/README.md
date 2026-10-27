# #537 — source review preparation and deferred acceptance

**Delivered preparation; human acceptance explicitly deferred by the owner on
2026-10-27.** No production source permission, full snapshot, balance setting,
physical trial or future experience approval is granted. Base: `ecbf59e21cd8405b940ff6593061c06978a7bc7d`
(main after #657). This work has no active predecessor PR.

## Owner-approved delivery and scope transfer — 2026-10-27

The owner instructed: “can you just approve results and proceeed? I'll verify them
later”. This supersedes the earlier draft-only delivery restriction. PR #658 may
close #537 for its verified tooling/evidence preparation; it does not claim the
unperformed human reviews passed. Acceptance is deferred explicitly as follows:

| Requirement | Actual state | Continuing owner |
| --- | --- | --- |
| Source-use/permissions, selected full inputs and exact snapshot review | Pending; linked full legal code returned 403; current inputs remain development fixtures | [#659](https://github.com/mr-exception/salimon/issues/659) |
| Physical survival/recovery routes and tuning decisions | Unmeasured; three blank forms; slow-setup comfort miss and net-negative stress retained | [#660](https://github.com/mr-exception/salimon/issues/660) |
| Full opening/exploration/Hall/backend experience | 12 portable cases passed; 14 blocked; implementations/backend unfinished | [#661](https://github.com/mr-exception/salimon/issues/661) |
| Human images/audio/corpus, exact context and mask/tolerance approval | Pending; candidates remain unapproved | Existing #561; physical platform #557; hardware budgets #563 |

Source-dependent work #413/#524/#526 follows #659. Active guides and new generated
packet ownership now point to these follow-ups; the existing frozen packet and
original logs/hashes remain historical evidence at their recorded revision.
Current-head [CI run 40487057804](https://github.com/mr-exception/salimon/actions/runs/40487057804)
passed before this ownership-only update. All 134 Python tests pass again in
[the ownership recheck log](ownership-recheck.log); documentation validation
against main and whitespace checks pass. Fresh packet generation points to #659
and production staging still rejects the pending review/current fixtures. No
source eligibility, approved gameplay parameter, visual baseline or hardware
claim is changed by this delivery decision.

## Finalization check before owner deferral — 2026-10-27

PR #658 head `418e8be27ef8dadb0465f30977987fc4bf92f8f4` is mergeable, with
no submitted reviews or review threads. [Source build run 40456973353](https://github.com/mr-exception/salimon/actions/runs/40456973353)
passed Ubuntu workspace compilation/tests and Windows/macOS publication goldens.
A fresh local run of all 134 Python tests passed; [original log](python-recheck.log).
Documentation validation against `origin/main` and whitespace checks also pass;
all six archived response hashes and byte counts were independently verified.
These checks verify the preparation tooling, not the missing human criteria.

The current environment reports an enforced unrestricted HTTP policy. With
network enabled for the execution command, all three original provider URLs now
return HTTP 200. The earlier revision-3 allowlist failures below are historical;
they are not the current source-access diagnosis. Six official pages are archived
with URLs, timestamps, response-byte hashes and lossless compressed originals in
the [current source evidence](network/current/README.md). The linked Creative
Commons full legal-code request still returns HTTP 403. Successful page retrieval
does not establish complete source-specific review, production selection or reuse
approval. No source status, production eligibility or owner decision changed.

The remaining completion blockers are the actual source-use/snapshot decisions,
complete selected production inputs, measured survival/recovery routes and
explicit balance decisions, and the unimplemented full-experience/backend
prerequisites. Three route forms remain blank and 14 matrix cases remain blocked.
The [decision checklist](decisions.md) records the exact next human actions.
Those source requirements remain unfinished in the linked follow-ups; the owner
subsequently authorized preparation delivery as recorded above.

## Delivered work

The new offline `scripts/prepare_source_review.py` freezes validated manifest and
raw inputs in a fresh atomic directory. It derives all inventory/form fields from
that frozen copy and rechecks it before publication. Existing destinations survive
reruns, corrupt input leaves no published partial packet, and generation is
deterministic. The generated form is intentionally incomplete and production
validation rejects it. Runtime, scientific models, source bytes/statuses, approved
meter durations, recipes and prior owner decisions remain unchanged.

- [Frozen source inventory](source-review/inventory.json),
  [exact original manifest](source-review/inputs/manifest-v1.json),
  [pending human review form](source-review/production-review.pending.json).
- [Fresh balance decision evidence](balance-review.json), raw
  [balance](balance.json) and [availability](availability.json).
- [Fresh experience results](experience/results.json): 12 passed, zero failed,
  14 explicitly blocked. These are portable Scout contracts, not the unimplemented
  opening/endgame/backend experience.
- [Owner decision and physical-trial checklist](decisions.md), with three blank
  normal/impaired/poor-layout forms. Empty values are unrun, never approved.

## Original preparation source evidence (before the finalization check)

The exact input-manifest SHA-256 is
`eaa6c4ff059ae06464d72744302cc2b7696fed7cc965759ec9f43af09f79ee3d`.
The owner-approved scope remains private development only, from the
[#513 decision](../issue-513/private-development-decision.json).
No commercial/distribution or full production-source decision is inferred.

| Source | Frozen bytes / coverage | Reuse state / remaining requirement |
| --- | --- | --- |
| Gaia DR3 | 185-byte synthetic two-row CSV; no real Gaia release | Restricted; actual full current terms/credits, intended use and required permission plus selected real inputs need human review |
| JPL Horizons Earth 399 | 6,924-byte observed response, two J2000 dates; ICRF/SSB KM-S/TDB | Unreviewed; source-specific rights/credits/model and a complete selected target/epoch set remain unapproved |
| NASA Exoplanet Archive PS | 438-byte Proxima Cen b default-row CSV | Unreviewed; archive/original-reference rights and credits plus selected full snapshot remain unapproved |

Current retrieval attempts on 2026-10-27 used inherited proxy/TLS settings. All
three official provider requests failed at proxy tunneling with **403 Forbidden**,
zero response-body bytes, before receiving provider content. These are environment
access failures, not a provider refusal of data rights. Exact URLs, UTC attempts
and errors are retained in [network](network/gaia-license.json),
[JPL](network/jpl-horizons-manual.json) and
[Exoplanet Archive](network/exoplanet-acknowledgments.json).
No terms were fetched or reviewed; historical attribution strings are not current
license evidence. This managed environment has an enforced restricted allowlist
that excludes those provider hosts. No configuration-write tool is available here.

To continue, environment access is needed for `www.cosmos.esa.int`,
`ssd.jpl.nasa.gov` and `exoplanetarchive.ipac.caltech.edu`; related official
metadata links also use `ssd-api.jpl.nasa.gov`. Approved data acquisition later
may need additional explicitly selected endpoints. No alternative proxy, mirror,
identity or bypass was attempted. Record/archive actual full terms and credits
before asking the owner for a source-specific production decision. Changing a
network rule itself would not approve reuse or full selected snapshots.

## Retry after provider-access confirmation

After the owner reported enabling provider access, all three requests were retried.
They still failed before receiving any provider content with proxy-tunnel
`403 Forbidden`; [retry URLs, times and errors](network/retry/gaia-license.json),
[JPL retry](network/retry/jpl-manual.json) and
[Exoplanet retry](network/retry/exoplanet-acknowledge.json) preserve the results.

The [current runtime policy observation](network/retry-policy.json) reports
specification revision **3**, matching observed revision 3, current observations
and an enforced restricted policy. Its only preset is `package_managers`;
`allowed_hosts` is empty and none of the four requested provider domains appears
in the effective rules. Thus the running policy still excludes those hosts.
This records the running instance, not a claim about what the owner saved in
another configuration draft. No configuration-write capability is exposed here.
The next action is to apply the four provider hostnames as allowed destinations
to this chat's active environment; enabling a package-manager preset alone does
not allow them. No additional permission to research the sources is needed.

## Refreshed balance and experience limits

All six current portable survival-balance tests pass. Their assumed travel/setup
budgets are **fixture assumptions**, not measured acquisition. First ration:
baseline 86.304 s, slow setup 431.304 s (over proposed 360 s comfort), impaired
221.384 s, stress 941.304 s with a -12.2947 unclamped single-ration margin.
The 720 s break-even budget and 360 s comfort budget remain proposals. No recipe,
biomass distribution or meter duration was changed. Resource availability covers
16 seeds within 200 m per body; it is not a global guarantee. See raw records
for null/unavailable resources rather than treating them as zero travel distance.

All #469–#475 implementation tickets remain open; #568 automation is closed.
The matrix retains 14 blocked cases and no native/backend signoff. Actual source
rights can be resolved before unrelated future-experience or balance acceptance;
closing #537 alone must never be used to infer those source rights.
Human image/audio/baseline review is separately owned by #561, physical platform
checks by #557 and hardware/M1/Metal budgets by #563. Their prior evidence is not
re-certified by this packet.

## Validation and reproduction

Use `. /workspace/.salimon-tools/env.sh` and
`PKG_CONFIG_SYSROOT_DIR=/workspace/.salimon-tools/sysroot` for the Rust commands.
Python 3.12.14; Rust/Cargo 1.99.1; Linux x86_64, inherited managed runtime.
The balance test log is losslessly gzip-compressed; other logs retain original bytes.

| Command | Actual result |
| --- | --- |
| `python3 scripts/prepare_source_review.py --output artifacts/issue-537/source-review` | Three source artifacts frozen; same manifest hash as the prior owner decision; production_eligible false |
| `python3 -m unittest discover -s scripts -p 'test_*source*.py' -v` | 13 passed, including exact binding, corruption/atomic publication, deterministic generation and rejection of pending production review |
| `python3 -m unittest discover -s scripts -p 'test_*.py'` | 134 passed |
| `SALIMON_BALANCE_REPORT=/absolute/balance.json SALIMON_AVAILABILITY_REPORT=/absolute/availability.json cargo test -p salimon-client --locked --offline survival_balance -- --nocapture` | Six passed; original [log](balance-tests.log.gz) and actual JSON retained |
| `python3 scripts/experience_suite.py --output artifacts/issue-537/experience` | Exit 3 (pending): 12 passed, zero failed, 14 blocked; per-case commands/logs retained |
| Development source staging | Passed through the existing verifier |
| Production review/staging and blank physical forms | Rejected as expected; no output published, no approval granted |
| Documentation/whitespace | Results recorded in docs-checks.log before publication |

No Rust production code, asset, shader or numerical contract changed. Full
workspace rebuild, graphics captures, hardware measurements and human gameplay
were not rerun for this tooling/review preparation; focused Rust checks above
refresh the relevant existing evidence. New source-review tooling tests and
[maintained documentation](../../docs/universe-sources.md#preparing-the-human-source-review-537)
cover the change. The owner subsequently authorized merging this preparation and closing #537
with exact residual criteria transferred above. No pending human criterion
becomes a pass.
