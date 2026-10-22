# #496 — balance review and physical route evidence boundary

2026-10-22. Delivered a fresh portable reproduction, explicit findings and an
executable physical-trial evaluator. **Owner budgets/tuning and reference
physical route acceptance remain pending**. No meter, recipe, modifier,
restriction or resource distribution changed.

## Reproduced findings

[Raw balance](balance.json), [availability](availability.json),
[summary with input hashes](review-summary.json), [Rust check log](balance-tests.log).
The six production-controller balance tests passed with the report environment
variables pointing here. Travel/setup remains declared fixture time, with
fixture placement between stages; this is not measured navigation.

| Fixture | First ration, seconds | +40 supply margin | Finding |
| --- | ---: | ---: | --- |
| baseline | 86.304 | +35.2053 | Below proposed comfortable time |
| slow setup | 431.304 | +16.0387 | Above 360-second proposal, below break-even |
| low health/fatigue | 221.384 | +27.7009 | Restrictions exercised with assumed travel |
| long route | 941.304 | -12.2947 | Net-negative per ration; retained failure |

A basic ration supplies 720 active seconds at unchanged 1800-second full hunger
drain; comfortable 360 seconds is still a proposal. Full fatigue remains 64800
seconds; offline fatigue recovery remains 21600 seconds. Do not infer balanced
survival from clamped two-ration hunger values.

Sixteen samples/body within 200m again find no biomass on Moon, Mars, Mercury or
Venus. Mercury/Venus also lack sampled ice; Moon/Mars have sampled ice. These
samples do not prove all global layouts. No food was added to those bodies.

## Physical evidence workflow delivered

[Unfilled trial worksheet](physical-trial-template.json) deliberately contains
nulls and is not valid completed evidence. The [canonical protocol](../../docs/survival-balance.md#physical-route-review-496)
requires the complete exclusive timed route, actual hardware/revision and
recording, poor-layout/obstacle and condition context, fresh gathered inputs
and newly produced/consumed rations. The evaluator rejects fixture travel,
starter funding, missing stages, bad timing, nonfinite data and overwritten
results. It retains failed/no-feedstock trials without a fabricated pass.

`python -m unittest discover -s scripts -p test_survival_route_review.py -v`
passed ([log](route-tests.log)); 360/720 boundaries, net-negative routes,
shortcuts, malformed evidence and scarce-body failures are covered with synthetic
**test inputs**, never represented as physical measurements.

## Pending owner/native acceptance

No Apple M1 reference gameplay/operator was available. Complete physical
navigation, discovery, obstacles, gathering, carrying, intake/crafting/collection
and Q remain unmeasured, including low health/fatigue and poor layouts. Coordinate
#376 HUD/wheel/accessibility evidence. No balance tuning is approved by this PR;
owner must accept/revise budgets and provisional recipes/modifiers/restrictions,
then authorize specific changes. The owner transferred these decisions and
reference route requirements to [follow-up #537](https://github.com/mr-exception/salimon/issues/537),
labeled `native`. [PR #534](https://github.com/mr-exception/salimon/pull/534) closes
#496 on merge for evidence/evaluator delivery. Project Q&A follows #537; no
budget, tuning, physical-route or accessibility acceptance follows from closure.

## Documentation impact and checks

Updated the canonical survival-balance guide with the new CLI/evidence schema.
Runtime/survival/Fabricator/ship authority and their architecture/invariants are
unchanged. Read the existing portable suite before reproducing it; no source
changes to the game. Documentation paths/anchors and whitespace checked.
