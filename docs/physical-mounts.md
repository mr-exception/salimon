# Physical placement and mechanical mounts (#576)

New journeys have a portable sample shelf, a heavier fixed cargo restraint and
three compatible fine habitation rail positions. These reuse the authored #575
Shelf template; they are physical geometry rather than inventory slots. Shelves
and clamps are not automatically fabricated by blueprint reservations.

Carry a fragment with F, aim downward within 2.5 m and press G to place it on a
clear deck or nearby shelf. Invalid/occupied placements leave the original object
in hand. F retains ordinary physical drop. Placing is **loose**: aim at the mount
and press K to explicitly secure a fitting object; K releases it again. F cannot
pick up a secured object. Mechanical locks work without Core power. Released
objects return to normal contact/gravity/zero-G motion; open cargo bins still
have real walls/open tops, with no protected contents or virtual inventory.

J unfastens an empty portable shelf. Occupied shelves and the heavier fixed
restraint refuse movement. The shelf occupies the same `WorldObjectId` carry
slot as a fragment; tools and fragment pickup cannot bypass it. Aim at a compatible
habitation rail and press G to reattach the existing identity without paying a
second fabrication cost. A carried shelf stays in hand until reattached, including
outside the ship; its contextual prompt explains the required rail action.

`salimon-ship::attachments` owns stable mount/rail/item references and compatibility;
world owns the single carry slot and original fragment identity/material/mass/pose.
Runtime composes aim, real hull/door/occupancy checks, fixed contacts, character
obstacles, and render-only fixture observations. Securing changes neither material
nor object ID and adds no second mass contribution. Secured fragments retain their
ship-local pose and orientation through translation/rotation. The existing portable
shelf's registered 2 kg is included in dry mass: carrying it outside removes that
same dry component once; returning restores it, without adding cargo mass. This
physical equipment parameter is a reviewable source default, not a measured material
density result, construction resource cost or duration.

Schema 11 persists the attachment source, carried accessory position and exact
child references. Restore reconstructs the carry slot and collision caches in an
isolated candidate and rejects duplicate/missing children, carried+secured conflict,
invalid rails/geometry or a second carried object before publication. Old schemas
migrate with no new installed shelves, avoiding surprise collisions with retained
cargo or player poses. New-run starter fixtures are not retroactively granted.
Continuous hand motion is not a discrete autosave event. Checkpoints preserve the
actual source without advancing clocks or fabricating resources.

Rail removal returns its exact secured children for explicit physical release by
the removing owner; it cannot silently consume a resource. New room construction,
new fixture fabrication and actual structural damage remain owned by #542–#545/
#549; those owners must supply their real geometry and release transactions before
claiming integration. There is no new damage, pressure or survival simulation.
Tactile acoustic events remain #583; human appearance approval remains #561 and
hardware qualification #557/#563.

## Verification

`cargo test -p salimon-client --locked session::mounts` exercises G/K/J via the
same typed command path as native keys, blackout, exact mass/IDs, current restore,
atomic corrupt-restore rejection and legacy migration. Ship attachment tests cover
compatibility, uniqueness and explicit parent removal; world carry tests retain the
one-object rule. Character installed obstacles use ordinary walking and zero-G
collision admission, and loose cargo uses the existing physics solver.

The opt-in `scout-mounts` initial condition deliberately seeds one 32 g iron sphere
through production spherical extraction and places the player at the shelf. It is
not evidence of naturally mined/gathered travel. Run:

```sh
python3 scripts/salimon-test run scenarios/evidence/physical-mounts.json --binary target/debug/salimon-client --screenshot-command '["native"]'
```

Captures come directly from production wgpu; no desktop pixels or baseline approval
are implied. See the [delivery report](https://github.com/mr-exception/salimon/blob/d003bd58d506e04e3df97744f29f68449b6e2b3e/reports/issue-576/README.md) for exact
results, original images and environment.
