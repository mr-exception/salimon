# Survival balance evidence (#384)

The portable `runtime/src/survival_balance.rs` suite exercises new-run supplies,
generated biomass/ice mining, F pickup, physical E intake, Core-funded crafting,
E tray collection and explicit Q consumption. It covers blocked/full slots and
tray, blackout, low health/fatigue, exhaustion gates, active Q versus full Escape
pause, exact ten-second starvation, offline fatigue-only recovery, save/reload
without regrants, locked advanced recipes and the existing moving-EVA return route.

Travel/setup are explicit **fixture time budgets**. The avatar is placed at each
stage after that budget, through validated character fixture state. These are
not measured obstacle navigation, a pathfinder, player discovery time or native
subjective evidence. Mining/F/E/Q and Core funding use production controllers.
The availability scan uses actual deterministic body profiles at sixteen seeds
per solid body near +Y within 200 m; it cannot establish a global worst case.

## Current numerical policy and proposed review budgets

- Hunger remains 100 points per 1800 active seconds; fatigue 64800 seconds and
  offline full recovery 21600 seconds. Basic ration +40 means 720 seconds of
  hunger supply; a repeatable one-ration route must finish inside 720 seconds.
- Proposed comfortable target: one ration within 360 seconds including travel,
  search/setup, gathering, Core setup, crafting and collection. Proposed stress
  ceiling: 720 seconds. These are measurable proposals, not approved difficulty.
- Basic recipe: 0.4 kg biomass + 0.1 kg ice, 10 seconds at 10 kW (100 kJ of
  crafting energy). Ordinary cabin/cockpit idle is 110 kW, so total run debit
  includes idle, blocked/setup and travel time. Mining is 2 kg/s before restrictions.
- New runs have two rations, one medkit and reserves for exactly one ration.
  With no waste/other food, three +40 rations add 36 minutes to the initial
  30-minute hunger supply. At zero hunger, full health lasts another 1000 seconds
  under unmodified starvation. This is an arithmetic bound, not a food guarantee.
- Default Core resource yields and conversion rate remain catalog-owned; the
  suite neither makes fuel free nor changes them. See the [ship contract](../client/ship/architecture.md).

[Raw balance rows](../reports/issue-384/balance.json) report actual first-ration
net hunger, total two-ration cycle, Core debit and declared time budgets. The
900-second travel +30-second setup case is net-negative per ration; this is an
expected sensitivity finding. Do not interpret its test pass as balance approval.
The four selected rows are scenarios, not a probabilistic player failure rate.

[Availability samples](../reports/issue-384/availability.json) find both required
feedstocks in all sixteen Earth samples; largest sampled nearest distances are
about 30 m biomass and 166 m ice. Current Mercury/Venus/Moon/Mars profiles lack
biomass entirely, so their samples cannot sustain basic-ration production. No
new world resources or approved meter tuning are silently introduced.

Owner budget/tuning acceptance, measured physical routes and subjective native
signoff remain #496 delivery follow-up [#660](https://github.com/mr-exception/salimon/issues/660), alongside
existing native #376. Advanced food/restriction provisional tuning remains
provisional until that review.

## Reproduce

Run from the repository root; paths are absolute because Cargo sets the test
working directory to the owning crate:

```sh
SALIMON_BALANCE_REPORT="$PWD/reports/issue-384/balance.json" \
SALIMON_AVAILABILITY_REPORT="$PWD/reports/issue-384/availability.json" \
cargo test -p salimon-client survival_balance --locked
```

These six bounded portable tests run in ordinary workspace PR CI. No window,
GPU, audio device, native baseline approval or network source is needed.

## Physical route review (#496)

[Current reproduction and review packet](../reports/issue-496/README.md) retains
fresh portable JSON, explicit failures/assumptions and an unfilled native trial
worksheet. No measured native route or owner balance approval is implied.

Copy [the trial template](../reports/issue-496/physical-trial-template.json) to a
new file; fill actual revision, reference hardware/OS/backend, body/seed,
health/fatigue/hunger, layout/obstacles, operator/date and recording/log evidence.
Time the full route through discovery, setup, outbound navigation, gathering,
return, physical intake, Fabricator, collection and Q consumption. These are
**exclusive active-time intervals**; their sum must match an independent active
stopwatch within 0.01 seconds. Pause stops both clocks. Count crafting overlap
once, as part of the interval actually occupied, never add a whole parallel job
duration twice. Preserve actual state before/after and elapsed production ticks
in the linked evidence. No fixture placements or starter material reserves can
prove the repeatable route. Record newly gathered biomass/ice and the number of
newly crafted/consumed rations, excluding starter rations.

```sh
python scripts/survival_route_review.py artifacts/my-physical-trial.json \
  --output artifacts/my-physical-trial-result.json
python -m unittest discover -s scripts -p test_survival_route_review.py -v
```

Use `outcome: completed` only with every stage measured. Failed/no-feedstock
trials use `outcome: failed` or `no_feedstock`, `failure_reason` and `null` for
unrun stages; these are retained without supply passes. The CLI exits 1 for
those outcomes, malformed evidence or a net-negative completed route, and 0 for
a completed route meeting break-even. Outputs never overwrite earlier evidence.
`comfortable_proposal_met` compares with the **unapproved** 360-second proposal.
`unclamped_hunger_supply_margin = 40 * rations - active_seconds * 100 / 1800`
is an arithmetic supply margin before meter caps/modifiers, not measured hunger
recovery. Actual meter observations remain in evidence. No result grants owner
approval or native signoff.

Reference trials must include baseline, low health/fatigue, discovery/setup,
obstacle detours and poor resource layouts. Retain failures on Moon/Mars/Mercury/
Venus; missing biomass is a design constraint, not permission to add it everywhere.
The 16-seed/200m sample cannot establish global availability. Owner acceptance of
budgets and provisional recipes/modifiers/restrictions remains #660 after #496 delivery; coordinate
HUD/wheel/visual accessibility with #376 before native acceptance.
