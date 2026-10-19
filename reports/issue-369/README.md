# Issue #369 — survival movement and interaction authority

Implemented against `main` `2ecc54b` (includes merged #393/#394, providing #367/#368).

## Delivered

- Exact fractional threshold intervals, multiplicative movement modifiers,
  configurable minimum, exact basis-point DTOs and typed permission policy in
  `salimon-survival::restrictions`.
- Shared native/automation session command and tick enforcement. Cabin, ramp,
  surface, EVA and zero-G voluntary motion scale without changing gravity,
  collision, jump impulse, inherited momentum or authoritative simulation time.
- At zero fatigue, interrupt and detach mining without losing extracted mass;
  close/reject machine operations, carrying and Core feeding; allow slow movement,
  emergency consumption permissions, door operation, safe cockpit exit and menus.
- Explicit ship pilot revocation zeros manual thrust/steering/held input while
  preserving heading, world momentum and committed assists. Recovery requires
  new cockpit entry/input and never replays stale controls.
- Configurable slower mining work and active-time recovery between nonessential
  E/F/crafting mutations. Emergency actions bypass the recovery timer. Ongoing
  Fabricator jobs/Core/world physics receive full admitted time.
- Permission-consistent cockpit/machine prompts and automation inspection of exact
  penalties through `survival_restrictions`; authoritative docs/maintenance updated.

The [canonical restriction contract](../../client/survival/architecture.md#movement-and-interaction-restrictions-369)
records continuous fractional boundaries, all action choices and explicit numeric
provisional tuning (10% minimum, 75% low-condition interaction factors, 250 ms
recovery base). These are configurable reviewable defaults; #384 owns later
balance measurement. Approved meter depletion durations were not changed.

## Integration boundaries

As established by #367/#368, #371 owns production survival ticking, full-save
schema/migration, offline recovery publication, terminal world freeze and restart.
This change composes the typed owner and physical consumers without starting an
unsaved survival clock; default playable vitals remain full until #371 lands.
#370 owns consumable effects; #377/PR #387 owns final Q-wheel dispatch and #373
owns persistent HUD labels. This PR supplies and tests their action/penalty APIs;
it neither depends on nor claims delivery of those later issues. No save schema
is silently changed. Derived movement scale and cooldown reset on restore.

## Verification

Linux x86_64 cloud session, Rust stable 1.99.1. Final gate results and relevant
suite summaries are in [validation.txt](validation.txt).

Focused runtime cases exercise cabin/surface/EVA movement, exhausted held
throttle/steering, legacy automation throttle rejection, fresh-input recovery,
actual E-door operation, attached mining interruption with mass preservation,
crafting denial, unchanged powered machine progression and committed assist
completion. Character tests check inherited EVA drift, zero-G acceleration and
save reset; ship tests check blackout/recharge revocation without braking.
Survival tests cover exact fractional boundaries, tuning validation/minimum,
multiplicative factors, emergency/terminal permissions and recovery duration.

No native GUI, OS-input or reference Apple M1/Metal checks were run. This PR does
not establish visual smoothness or hardware performance. Existing #376 tracks
full survival native verification after the integration chain; #312 tracks native
input/lifecycle regression evidence. No acceptance has been silently waived.
