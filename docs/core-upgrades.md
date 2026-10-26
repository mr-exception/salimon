# Instant physical-material Core upgrades (#548)

## Balance decision and catalog

On 2026-10-24 the owner explicitly asked Codex to choose balanced tiers/costs,
superseding the earlier undecided-values restriction for this implementation.
These reviewable gameplay values live in `client/runtime/catalogs/core-upgrades.json`:

| Applied tier | Capacity | Iron ore | Silicate rock | Full 10 kg pieces |
| --- | --- | --- | --- | --- |
| Starter | 60 GJ | — | — | — |
| 1 | 90 GJ | 20 kg | 10 kg | 3 |
| 2 | 120 GJ | 30 kg | 10 kg | 4 |
| 3 | 180 GJ | 40 kg | 20 kg | 6 |

Costs are incremental, sequential and paid once per tier. Current default mining
is 2 kg/s with a 10 kg sphere cap: raw extraction is approximately 15/20/30 seconds,
plus real access, carrying and feeding time. This is a design rationale, not a
measured acquisition playtest. Capacity grows without thrust/efficiency changes,
so the upgrade never supplies free charge or removes the normal fuel loop.
All tiers together require 130 kg / 13 standard pieces. Balance remains reviewable;
this owner-authorized Core exception does not approve the normal construction
catalog's proposed economy or unrelated #537 decisions.

## Physical interface

Aim at the existing Core interface. I explicitly switches between **FUEL INTAKE**
and **UPGRADE MATERIALS**. E in Fuel burns the carried fragment through the existing
conversion pipeline. E in Upgrade contributes one required physical fragment to
that next tier's reserved material tray, without fuel conversion. There is no
fallback from a rejected upgrade contribution to burning it. Pending materials
are separate exact receipts and cannot enter fuel intake. Mode survives reload;
context and transient feedback remain visible together, including in blackout.

The prompt identifies current installed tier/capacity, next tier/capacity and each
remaining material cost. Y activates the complete next tier instantly. N recovers
the last pending fragment to the one existing carry slot; clear the hand and
physical exit first. I can return to Fuel without losing pending receipts. No
worksite, construction stages/timer, upgrade-energy charge, auxiliary battery,
Core shutdown/disconnection or commissioning wait is involved. Normal modal,
blueprint, pause, survival, view, nearby interior aim and input admission still apply.

Costs use integer milligrams. Whole original fragments must fit the remaining
material requirement; oversized fragments are rejected unchanged with a prompt.
Mine/release a smaller piece rather than receiving an invented remainder. Quantities
below one milligram or not representable to a whole milligram are rejected;
1e-9 kg numerical tolerance admits the existing f64 extraction noise only. Original
accepted mass/material/volume/source/fragment ID remain intact in each receipt.
No fractional material is silently lost. Pending receipt storage is bounded to
128 pieces; recover partial contributions if that explicit limit is reached.

## Transaction, mass and persistence

Ship `core_upgrades.rs` owns receipt/cost validation and activation; world owns
original physical entities and the guarded reserved-return boundary. Runtime
owns physical controls, catalog loading and the single mass ledger entry for
pending plus installed material. Contributing, recovery and activation preserve
onboard total mass. Applied receipts remain a nonspendable audit of installed
matter, not retrievable fuel or a second inventory. Exact IDs cannot appear in
world/pending/applied source simultaneously; allocator high-water and known-tier
recipes/capacity are checked before restore publication. Future catalog changes
must preserve/migrate already paid tier definitions deliberately.

Activation changes only the existing EnergyCore's capacity field. Its current
fractional stored joules, conversion pipeline, assist reservations, propulsion
configuration, velocity, engine switch and selected throttle remain untouched.
Capacity gain does not fill the new space. Invalid/incomplete/repeated application
leaves configuration and material accounting unchanged. Zero-charge activation
is valid and leaves zero joules. Ordinary positive-charge throttle resume follows
the existing bus rules on later simulation ticks; activation introduces no flight
command or special maintenance blackout behavior.

Whole-session schema 14 adds source mode, pending original receipts and applied
tier/material audit. Older saves migrate to Fuel with no contributions/upgrades;
incompatible older envelopes reject new source. Restore constructs and validates
temporary owners, rebuilds the mass entry once and never ticks. Material/mode/tier
changes are discrete autosave events. World reserved returns retain the allocated
ID, seal it against extraction-tail growth, and reject duplicates/occupied slots.
A blocked physical refund rolls back without deleting the contribution.

See [the dated report](../reports/issue-548/README.md) for checks, native evidence,
input fixtures, balance rationale and human/hardware limitations.
