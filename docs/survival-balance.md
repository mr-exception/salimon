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
signoff remain [#496](https://github.com/mr-exception/salimon/issues/496), alongside
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
