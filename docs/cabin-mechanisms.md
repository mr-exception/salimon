# Scout cabin mechanisms (#577)

Runtime `session/cabin.rs` owns five manually reversible source mechanisms, supported
postures and restraints. Ship `mechanism.rs` supplies the bounded smoothstep motion
primitive. Renderer fixture nodes are presentation only. Character owns admitted
supported poses and normal collision; loose-object contacts and pickup rays consume
the same runtime physical bounds. The editable Blender library and checked runtime
geometry live in `models/assets/fixtures/scout-habitation-kit/` and
`client/assets/fixtures/scout-habitation-kit/` respectively.

Aim at a nearby physical fixture and press E to deploy/stow the table or padded
seat, slide the three-panel privacy screen, raise/lower the five-panel observation
shutter or open/close the hollow locker. Press again to reverse. Motion halts with
an obstruction prompt if its swept bounds encounter the player, another fixture
or a physical resource. No contents are teleported, deleted or converted to slots.
G places carried objects on sufficiently large clear upward surfaces, including
the table. Moving mechanisms wake loose-object contact history. Carrying into a
closed fixture retains the last clear pose, including its moving ship frame;
blocked carried objects cannot enter the Core or Fabricator remotely.

X sits in a fully deployed seat or lies on the deployed bunk, with a 0.35 s physical
transition and retained free look. X/E gets up. U explicitly secures/releases the
visible corresponding harness; release it before getting up. Open the privacy
screen before leaving the bunk. Screen/shutter controls remain usable while
resting. Occupied seats/bunks cannot fold. Supported rest uses a seated/lying body
proxy; ordinary standing collision resumes after the return transition. Unrestrained
support in zero-G starts a safe exit transition; restraints retain the body through
blackout. Unrestrained zero-G entry is rejected. There is no time skip, survival
bonus, hygiene/water meter, implicit flight command or extra inventory.

All five mechanisms and restraints are manual and work without Core power. Galley
continues to use the existing exact-identity Q-wheel; hygiene remains inspectable.
Room light controls/preferences belong to #580; mechanical sound presentation to
#583. Pause/modal gates and the single physical carry slot remain authoritative.

## Save and verification

Whole-session schema 12 adds optional `habitation.cabin`, including finite motion
fractions/targets, restraints and exact posture transition source. Older saves
retain their original layout without new colliders. Restoration validates source,
occupied supports and interpolated player pose atomically; no elapsed offline work
is introduced. Autosave checkpoints observe discrete targets/posture changes,
not every animation fraction. Normal explicit saves retain intermediate motion.

`session/cabin/tests.rs` covers free look, blackout/restraints, safe exit, reversals,
occupied/obstructed mechanisms, exact save restoration, corrupt-state rejection,
legacy layout and authored geometry/sweep agreement. `scenarios/evidence/habitation-use.json`
uses real native inputs after an explicit initial camera setup. Native appearance
and comfort still require #561; hardware budgets remain #563. See the dated
[implementation report](https://github.com/mr-exception/salimon/blob/d003bd58d506e04e3df97744f29f68449b6e2b3e/reports/issue-577/README.md) for actual results.
