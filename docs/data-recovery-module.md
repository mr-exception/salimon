# Scout Data Recovery Module — #468

The physical Fabricator can produce one `data-recovery-module` controller package.
Collecting that tray item with E installs its circuitry into the existing Archive
terminal housing opposite the Fabricator. This deliberate first implementation
uses the existing anchored package/housing; it adds no loose carried equipment,
food/medical slot, automatic resource grant or new world collider. The same
(job, unit) moves from tray source to durable installed source, retaining physical
mass exactly once. One pending/tray/installed module is permitted. Duplicate or
batch requests reject before reserving ingredients; cancellation never refunds.

## Configured recipe and work

[Recipe data](../client/fabrication/config/recovery-module.json) is validated through
`RecipeBook`: 1 kg iron ore + 0.5 kg silicate rock, 60 seconds at 1 MW, output
0.75 kg. These are explicit provisional implementation defaults, subject to owner
balance review in [#537](https://github.com/mr-exception/salimon/issues/537).
They are not an approved progression balance. Existing 20 kg bins, physical raw
mining/carry/intake, FIFO head, one tray, Core funding and no-refund rules apply.

[Recovery data](../client/runtime/config/data-recovery.json) supplies the captured
per-run configuration: 5 MW maximum work, 3/5 GJ for the two existing damaged
records and a 2 GJ emergency floor. Full nominal work is 600/1,000 seconds;
resource gathering/installation and insufficient power can extend real elapsed
time. JSON data is compiled into the initial definition; changes require a build
and apply to new runs, while saves retain the original validated configuration.
No arbitrary instantaneous recovery timer or second energy currency exists.

## Core funding and reserve

On each shared fixed tick, ordinary ship loads, conversion and Fabricator settle
first. Recovery then requests at most watts × tick seconds and remaining record
cost. `ShipController::fund_auxiliary_work` debits only actual stored unreserved
joules above the larger of configured reserve and twelve seconds of current
full forward/idle demand (covering the longest existing crash recovery assist).
Existing isolated assist reservations are already outside stored bus energy.
Recovery cannot consume either reserve, and recovery joules never create thrust.
Propulsion/crafting can independently consume ordinary energy; this is a floor
on voluntary data work, not a guarantee that every future maneuver is affordable.

Paid energy alone increments reconstruction progress, capped at the record cost.
No power or a reached floor means no work, no debit and no banked time; the same
active job resumes after ordinary conversion restores a surplus. Core overfill
retains the existing clipping/waste policy. Low-priority work is settled after
that ordinary transaction. Actual last-tick auxiliary watts enter Core total-draw
telemetry without a second debit; predictive depletion assumes unchanged draw.

## Archive interaction and information

Open Archive with normal aimed/inside/powered E interaction. Select a damaged
record and press R to start/stop its job. One record runs at a time; switching
retains both records' paid work. Stop/abort retains work and never refunds Core
energy. Reading/closing the terminal does not stop a machine already running.
Pause freezes simulation; closed-app recovery is zero. No retrospective work is
credited at load. Future universe offline machinery remains #433.

The accessible text shows installation, working/stopped/reserve-limited status,
requested MW, protected GJ, per-record percentage and remaining nominal work.
Intact, unknown damaged, partially reconstructed, unverified recovered fragments
and permanently missing second-record segments stay distinct. Record identities,
sender/source and unknown timestamps persist; recovered text is an unverified
fragment, never proof of unseen history. Full distress/warning location leads
and their authored destination integration remain [#470](https://github.com/mr-exception/salimon/issues/470)
after charting/world prerequisites. Recovery cannot invent coordinates, expose
Earth/Hall destinations or reconstruct missing bytes into certainty.

## Persistence and verification

Current runtime save schema 7 requires version-1 Recovery source: captured config,
installed item identity/mass, paid joules and active record. The transient reader,
inputs and auxiliary telemetry remain excluded. Invalid progress/config, missing
current source, duplicate installed/tray/pending identities or unknown recipes
reject before session publication. Mass rebuild includes the installed module.

Explicit v1–v6 migration starts without module/progress and adds the new blueprint
when absent, retaining old recipe definitions, jobs, supplies and Core state.
A fresh run resets hardware/progress. Installation/start/stop/completion enter
discrete checkpoint events; periodic/exit/Save Now checkpoints include partial
work. Existing verified fallback/atomic publication applies to the complete save.

Tests compose real domain mining, carrying, Fabricator/Core conversion and Archive
work using declared synthetic deposits and supplied placement. This is not a
measured walking/gathering route. CPU raster evidence verifies bounded text;
[cloud verification #515](https://github.com/mr-exception/salimon/issues/515)
adds production native-window intake/craft/collect/walk/R captures and exact
concurrent-funding, migration and Save Now/relaunch/restart regressions. Its
declared initial materials do not measure acquisition or approve recipe/energy
balance. Human presentation is #561, physical OS/input/DPI is #557, hardware
budgets are #563, and route/balance decisions are #537 under the
[verification workflow](verification-workflow.md).
See [runtime ownership](../client/runtime/architecture.md#data-recovery-module-468),
[Fabricator ownership](../client/fabrication/architecture.md#recovery-controller-component-468)
and [ship energy boundary](../client/ship/architecture.md#low-priority-data-recovery-energy-468).
