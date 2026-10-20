# Issue #371 — active survival, full saves and terminal restart

Implemented against `main` `fb2b5d4` after PR #397 merged #370. All implementation
prerequisites are present; neither blocked draft #387 nor #395 was merged here.

## Delivered

- Production session survival ticking on admitted active intervals, including
  ordinary menus. Hunger/fatigue, fractional starvation/regeneration, wounds,
  damage/healing/status rules/stacks/remaining durations, terminal cause and
  survival active time now share the actual gameplay clock.
- Schema-2 whole-game source includes survival, consumable definitions, restriction
  tuning and typed run/journey source with player, ship, Core, world, fragments,
  Fabricator and consumed inventory. Validation checks captured effect/catalog
  compatibility. Physical/domain restoration is transactional, clears stale input/
  UI/view/caches, repairs derived restrictions and never advances simulation.
- Explicit schema-1 migration preserves all old physical/Fabricator/equipment
  source and initializes only the new survival state (full meters, zero active
  survival time, no effects). Schema 2 requires its complete survival payload;
  unknown versions and invalid/mismatched source reject before publication.
- Native launch restores into a temporary candidate, credits ONLY offline fatigue
  with the domain's cap/rollback/high-watermark policy, verifies a checkpoint and
  then exposes it. Failed publication leaves live source unchanged. Direct save/
  restore, focus/resume and renderer resets never grant offline recovery.
- A lethal tick advances physical domains only to its admitted lethal boundary;
  later ticks/commands freeze all authoritative domains and discard catch-up.
  Held movement/mining/crafting/piloting clear. Terminal state is checkpointed
  into both generations; failed storage is reported rather than claimed durable.
- Minimal native terminal notice and initial real focused R restart path. Restart
  prepares a deterministic fresh world with full meters, empty old world/inventory/
  jobs/effects, reset inputs and a new run serial. The save worker writes terminal
  generations, an immutable idempotent archive summary, and BOTH fresh fallback
  generations before exposing new play. Successful restart cannot fall back to
  the previous run. Failure/retry retains the terminal live source.
- Host-time terminal save retry, because frozen gameplay time cannot drive retry.
  Archive allocation avoids reusing historical serials after missing/corrupt saves.

[Canonical save/lifecycle contract](../../docs/save-contract.md) records schema,
clock, migration, crash and IO guarantees. In particular, if the very first death
write fails, only the prior verified checkpoint can be guaranteed; the PR does
not claim impossible durability on failed storage. Once the terminal pair or fresh
pair is verified, its backup cannot revive an older living run.

## Validation

Linux x86_64, Rust stable 1.99.1. **633 workspace tests passed; 22 existing opt-in
checks ignored.** Workspace build, all-target Clippy with warnings denied,
formatting, 93 Python tooling tests and changed-link checks passed. See
[validation.txt](validation.txt) for exact suite summaries and new test names.

Thirteen new tests cover lethal sub-tick truncation/frozen commands, full-game
partial-healing continuation, consumed crafted item + effect atomic save, schema-1
migration/schema-2 rejection, menu clocks/transient reset, fresh world identity,
only-fatigue launch credit/rollback/idempotence, native filesystem restart/fallback,
archive collision/idempotence, failed launch publication with no candidate exposure,
terminal retry with a frozen gameplay clock, and write/archive fault injection
at every restart stage. Existing moving-ship/EVA/500-fragment, corruption/fallback,
input, Core/Fabricator and held-control regressions passed.

The required normal native smoke was **attempted but blocked before creating a
window**: winit could not connect to an X server. Display dependencies were
prepared, but this execution environment could not expose the X11 server to the
client. No native UI, keyboard, screenshots or Metal behavior is claimed passed.
See [native smoke result](native-smoke-result.json) and [client log](native-smoke-stderr.log).
Existing [#376](https://github.com/mr-exception/salimon/issues/376) retains full-save/
force-close/relaunch, terminal/restart and reference-M1 signoff; #312 retains native
input/lifecycle regression evidence.

## Remaining feature owners and review implications

- #372 samples distance/coordinate discontinuities and enriches journey location/
  report semantics. Its typed distance counter source is persisted/archived now;
  until its sampler lands the default distance is zero, not a measured journey.
- #374 owns the full cinematic Journey Ended screen, layout and Restart Game UI;
  this PR provides the durable lifecycle and a minimal native R notice.
- #373/#377/#379 own the HUD, native Q-wheel effect dispatch and starter supplies.
  This PR activates survival before that presentation/usability wave; native food
  consumption is still pending #377. Do not treat this intermediate build as the
  completed survival experience. #378/#384 own expanded food effects/balance.
- #381/#382 own full pause and verified manual-save UI. Ordinary menus still run
  simulation; lifecycle exclusions do not become offline recovery.

These existing owners are not prerequisites for this lifecycle implementation,
and no native reference signoff is waived by closing #371 on merge.
