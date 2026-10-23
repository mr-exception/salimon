# Cloud and human verification migration — 2026-10-23

## Outcome

Owner authorized cloud-first checks and replacement of the native workflow
category with auto-verification and human-verification. Updated the 66 existing
open issues: 62 auto and four human (#537/#557/#561/#563). Created auto issue
[#568](https://github.com/mr-exception/salimon/issues/568) for machine-executable
corpus/experience work transferred from #537. All current native title prefixes
and label assignments are removed; unrelated labels and historical evidence are
preserved. Closed issue history is not rewritten. The native repository label
is retired from active use; this change does not claim to delete its definition.

## Changes

Canonical workflow: [verification policy](../../docs/verification-workflow.md).
Updated agent/completion/validation/design/experience/presentation/corpus/CI
guides and issue/PR templates. Human issues include ordered steps, evidence,
reviewer/context/hash records, return-to-Codex instructions and PR/direct-close
rules. Automated issues require execution, a reviewable PR/report and issue
updates; feature/source/backend dependencies remain. Original verification issue
bodies are preserved as explicitly superseded historical scope where rewritten.
No failures, baseline approvals, data permissions, product decisions or physical
performance targets were granted by migration. Cloud graphics may use software
Vulkan; hardware-specific claims require their actual hardware.

## Issue migration inventory

| Issue | Primary label | Current title |
| --- | --- | --- |
| [#413](https://github.com/mr-exception/salimon/issues/413) | auto-verification | [Universe U12] Publish validated compressed offline universe pack |
| [#414](https://github.com/mr-exception/salimon/issues/414) | auto-verification | [Universe U13] Indexed universe reader and bounded partition cache |
| [#415](https://github.com/mr-exception/salimon/issues/415) | auto-verification | [Universe U14] Time-addressable ephemerides, body frames and motion |
| [#416](https://github.com/mr-exception/salimon/issues/416) | auto-verification | [Universe U15] Dominant-four celestial gravity with stable source selection |
| [#417](https://github.com/mr-exception/salimon/issues/417) | auto-verification | [Universe U16] Gravity-aware ship trajectory integration |
| [#418](https://github.com/mr-exception/salimon/issues/418) | auto-verification | [Universe U17] Real-gravity launch, landing and support-force assists |
| [#419](https://github.com/mr-exception/salimon/issues/419) | auto-verification | [Universe U18] Altitude-dependent atmospheric drag |
| [#420](https://github.com/mr-exception/salimon/issues/420) | auto-verification | [Universe U19] Moving-body swept trajectories and local impact resolution |
| [#421](https://github.com/mr-exception/salimon/issues/421) | auto-verification | [Universe U20] Real-scale navigation trajectory prediction and telemetry |
| [#422](https://github.com/mr-exception/salimon/issues/422) | auto-verification | [Universe U21] Galactic and local-system spatial streaming |
| [#423](https://github.com/mr-exception/salimon/issues/423) | auto-verification | [Universe U22] Hybrid artistic Milky Way background and physical nearby stars |
| [#424](https://github.com/mr-exception/salimon/issues/424) | auto-verification | [Universe U23] Seamless planet terrain LOD and deterministic tile streaming |
| [#425](https://github.com/mr-exception/salimon/issues/425) | auto-verification | [Universe U24] Planet landing/mining/resource integration with new terrain |
| [#426](https://github.com/mr-exception/salimon/issues/426) | auto-verification | [Universe U25] New journey spawns outside landed ship on alien homeworld |
| [#427](https://github.com/mr-exception/salimon/issues/427) | auto-verification | [Universe U26] Retire compressed playable catalog and preserve test fixtures |
| [#428](https://github.com/mr-exception/salimon/issues/428) | auto-verification | [Universe U27] Scanner contacts, expiry and historical discovery model |
| [#429](https://github.com/mr-exception/salimon/issues/429) | auto-verification | [Universe U28] Five-second scanner pulses, power and offline detection |
| [#430](https://github.com/mr-exception/salimon/issues/430) | auto-verification | [Universe U29] 3D holographic cockpit radar module |
| [#431](https://github.com/mr-exception/salimon/issues/431) | auto-verification | [Universe U30] Persist scanner contacts/history through saves |
| [#432](https://github.com/mr-exception/salimon/issues/432) | auto-verification | [Universe U31] Gravity-only closed-game trajectory and event propagation |
| [#433](https://github.com/mr-exception/salimon/issues/433) | auto-verification | [Universe U32] Offline timed systems and scanner catch-up |
| [#434](https://github.com/mr-exception/salimon/issues/434) | auto-verification | [Universe U33] Main-menu offline propagation progress and verified resume |
| [#435](https://github.com/mr-exception/salimon/issues/435) | auto-verification | [Universe U34] End-to-end universe physics, catalog and 60FPS benchmarks |
| [#436](https://github.com/mr-exception/salimon/issues/436) | auto-verification | [Universe] Verify real-scale rendering and planetary streaming in cloud |
| [#437](https://github.com/mr-exception/salimon/issues/437) | auto-verification | [Universe] Verify orbital flight, collision and cockpit controls in cloud |
| [#438](https://github.com/mr-exception/salimon/issues/438) | auto-verification | [Universe] Verify holographic scanner and cockpit integration in cloud |
| [#439](https://github.com/mr-exception/salimon/issues/439) | auto-verification | [Universe] Verify new-game and offline restoration end-to-end in cloud |
| [#441](https://github.com/mr-exception/salimon/issues/441) | auto-verification | [Universe U36] Final code-to-documentation consistency audit |
| [#469](https://github.com/mr-exception/salimon/issues/469) | auto-verification | [Exploration X01] Player-known star chart, unknown Earth and coordinate provenance |
| [#470](https://github.com/mr-exception/salimon/issues/470) | auto-verification | [Narrative N03] Recoverable distress and warning transmissions with investigation leads |
| [#471](https://github.com/mr-exception/salimon/issues/471) | auto-verification | [Experience E05] Waking outside Scout on alien world and first-hour experiential slice |
| [#472](https://github.com/mr-exception/salimon/issues/472) | auto-verification | [Absenat A01] Endgame gate and primordial circular Hall spatial prototype |
| [#473](https://github.com/mr-exception/salimon/issues/473) | auto-verification | [Absenat A02] Completion ceremony, one blank inner-ring inscription and hidden main-menu unlock |
| [#474](https://github.com/mr-exception/salimon/issues/474) | auto-verification | [Absenat A03] Backend endgame identity, moderated inscriptions and distance leaderboard |
| [#475](https://github.com/mr-exception/salimon/issues/475) | auto-verification | [Absenat A04] Hall returning-visitor streaming, snapshots and account access integration |
| [#483](https://github.com/mr-exception/salimon/issues/483) | auto-verification | Verify Scout lighting evidence and capture tooling in cloud |
| [#485](https://github.com/mr-exception/salimon/issues/485) | auto-verification | Verify Scout audio synthesis, routing and playback lifecycle in cloud |
| [#489](https://github.com/mr-exception/salimon/issues/489) | auto-verification | Verify Scout machinery feedback and interaction walkthrough in cloud |
| [#498](https://github.com/mr-exception/salimon/issues/498) | auto-verification | Verify funded-impulse engine handling and assist handoff in cloud |
| [#502](https://github.com/mr-exception/salimon/issues/502) | auto-verification | Verify Scout UI, physical screens and interaction cues in cloud |
| [#503](https://github.com/mr-exception/salimon/issues/503) | auto-verification | [UI] Apply Scout readability contract to the future scanner hologram |
| [#505](https://github.com/mr-exception/salimon/issues/505) | auto-verification | Verify Scout Archive interactions, readability evidence and persistence in cloud |
| [#515](https://github.com/mr-exception/salimon/issues/515) | auto-verification | Verify Recovery Module installation and Archive progress in cloud |
| [#521](https://github.com/mr-exception/salimon/issues/521) | auto-verification | [Universe] Run U03 scientific reference workloads in cloud |
| [#524](https://github.com/mr-exception/salimon/issues/524) | auto-verification | [Universe U07] Validate approved Gaia release cross-matches and full-data resource budget |
| [#526](https://github.com/mr-exception/salimon/issues/526) | auto-verification | [Universe U08] Validate full reviewed Solar System and exoplanet source coverage before pack publication |
| [#537](https://github.com/mr-exception/salimon/issues/537) | human-verification | Review source permissions, survival balance and full-experience decisions |
| [#538](https://github.com/mr-exception/salimon/issues/538) | auto-verification | [Scout S01] Redesign starter hull layout for separated cargo bay and integrated habitation |
| [#539](https://github.com/mr-exception/salimon/issues/539) | auto-verification | [Scout S02] Implement physical cargo bay storage, bulkhead door and zero-G containment |
| [#540](https://github.com/mr-exception/salimon/issues/540) | auto-verification | [Scout S03] Add practical fold-away bunk, galley and hygiene to starter cabin |
| [#541](https://github.com/mr-exception/salimon/issues/541) | auto-verification | [Building B01] Introduce ship-local 3D structural grid and Core-rooted connectivity model |
| [#542](https://github.com/mr-exception/salimon/issues/542) | auto-verification | [Building B02] Create blueprint placement tool and physical in-world placeholders |
| [#543](https://github.com/mr-exception/salimon/issues/543) | auto-verification | [Building B03] Feed real materials to placeholders and implement proportional physical refunds |
| [#544](https://github.com/mr-exception/salimon/issues/544) | auto-verification | [Building B04] Automatically construct funded placeholders with brief animation and Core-powered progress |
| [#545](https://github.com/mr-exception/salimon/issues/545) | auto-verification | [Building B05] Complete multi-cell rooms, decks and modular hull geometry from one placeholder |
| [#546](https://github.com/mr-exception/salimon/issues/546) | auto-verification | [Building B06] Enforce exterior construction takeoff interlocks and safe stop/cancel |
| [#547](https://github.com/mr-exception/salimon/issues/547) | auto-verification | [Building B07] Install and upgrade non-Core ship modules through physical placeholders |
| [#548](https://github.com/mr-exception/salimon/issues/548) | auto-verification | [Core C01] Add instantaneous direct-material Energy Core upgrades without construction sites |
| [#549](https://github.com/mr-exception/salimon/issues/549) | auto-verification | [Building B08] Handle construction damage, physical salvage and safe structural detachment |
| [#550](https://github.com/mr-exception/salimon/issues/550) | auto-verification | [Building B09] Persist construction projects and simulate eligible work offline |
| [#551](https://github.com/mr-exception/salimon/issues/551) | auto-verification | [Building B10] Verify starter Scout and modular construction end-to-end in cloud |
| [#557](https://github.com/mr-exception/salimon/issues/557) | human-verification | Verify physical input, display and OS sleep/wake on an available desktop |
| [#559](https://github.com/mr-exception/salimon/issues/559) | auto-verification | [physics] Resolve mixed-sphere grid-query stress with cloud profiling |
| [#561](https://github.com/mr-exception/salimon/issues/561) | human-verification | Review cloud and desktop visual, audio and gameplay presentation evidence |
| [#563](https://github.com/mr-exception/salimon/issues/563) | human-verification | Decide hardware performance budgets and retained stress limitations |
| [#566](https://github.com/mr-exception/salimon/issues/566) | auto-verification | Collect matched Scout lighting timings and regression evidence in cloud |
| [#568](https://github.com/mr-exception/salimon/issues/568) | auto-verification | Automated corpus/experience follow-up from #537 |

## Validation and limitations

Passed: `python3 scripts/check_docs.py` and
`python3 scripts/check_docs.py --base origin/main` (zero failures), expanded
local-link/anchor checks including new report/visual/PR guides (zero failures),
PyYAML parsing/label/field checks for all three issue forms, migration inventory
checks (66 unique issues, correct labels/title prefixes/human handoff fields),
and `git diff --check`.

All 66 GitHub issue updates succeeded. Live read-back confirms 63 open
`auto-verification` issues including #568, four `human-verification` issues
(#537/#557/#561/#563), and zero open `native` label assignments. Read-back
confirmed migrated body/title policy; #541's concurrent implementation delivery
links to PR #569 were retained after migration, and #413's current source-review
label reference was corrected. No source issue was closed by this migration.
No runtime code, renderer policy, physics, thresholds or baseline image bytes
changed. Runtime/gameplay tests are not required for this documentation migration.
Existing native terminology still describes the actual application/backend and
dated evidence; it is no longer a workflow label or blanket M1 completion gate.
Repository label definitions were created through GitHub issue-label mutation;
no label-description/color editing API was available in the connected tools.
