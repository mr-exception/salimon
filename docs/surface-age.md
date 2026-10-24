# Persistent Scout surface age (#581)

Ship `src/surface_age.rs` owns integer cosmetic exposure keyed by exact physical
identity. Runtime `session/surface_age.rs` admits installed identities and maps
actual successful use. Renderer `surface_age.wgsl` owns the fixed localized contact
pattern shared by ship, fixture and handheld pipelines. Aging never changes mass,
collision, resources, energy, durability, capability or physical condition.

Passive exposure contributes at most 0.12 over 120 days. Use contributes at most
0.32 over 1,200 equivalent contact seconds; bunk/seat fabric contact is weighted
1/60. A complete manual traverse contributes one equivalent second, quantized to
millionths of travel. Reversing counts only distance actually traveled; blocked
motion and failed clamp actions add nothing. Fabricator exposure counts only its
funded work duration, mining-tool exposure only ticks that extract material,
galley exposure admitted handling, floor exposure actual ship-local walking, and
mount/rail contact successful securing/releasing/moving operations. Flight transport,
held keys without productive work and save/load are not use sources.

Each surface records its birth in one passive clock plus saturating integer use.
The validated clock/use range is 100 years; intensity always stays at or below
0.44, including after very long verified absence. There is no per-frame random
scratch generation, texture allocation, cleaning mechanic or age-driven failure.
New installed physical IDs start at the current clock with zero use. Recoloring
and repairing never write exposure. The registry retains exact lost-cell/project
tombstones, so destruction or detachment cannot transfer old age to a replacement.
Room partition edits and mount relocation do not change physical identity.

## Material and condition mapping

Fixed object-local contact bands provide painted scuffs, metal contact polishing,
fabric fading, floor grip wear and Fabricator heat discoloration. The ship importer
explicitly tags deck/runners, galley/hygiene, pilot upholstery and nonfunctional
Fabricator casing/input/tray; independent bunk, seat, table, screen, shutter,
locker, shelves and rails use stable fixture IDs. Moving a fixture moves its pattern.
Harness colors, displays, functional lamps, status bands, warnings and glass remain
excluded. Wear composes after cosmetic tint and before lighting, including blackout.
It does not darken glass or change emission/power admission.

Completed modular cells and installed constructed objects carry a shared service
cover. `PanelDamaged` replaces `Panel` only for the authoritative #549 damaged
cell/work flag. A clean repair restores the exact normal variant and removes all
scar geometry; exposure remains byte-identical. Lost/destroyed identities submit
no attached cover. Covers are presentation layers on existing physical geometry,
never an additional rigid body, collider, salvage item or detachment authority.
The authored starter hull/fixtures have no independent damage authority in #549;
this implementation does not invent one or manufacture damage from age.

## Persistence and offline reconciliation

Whole-session schema 23 requires the validated exposure source. Versions 1–22
migrate to pristine source without historical age credit; nonempty age smuggled
into an older envelope rejects. Missing current source, unknown physical IDs,
invalid birth times and excessive populations reject before publication.
Ordinary restore preserves exact state and performs no catch-up. The existing
[verified launch clock](construction-offline.md) alone admits closed-game time.
Passive exposure and actually funded Fabricator work settle within that same
transaction, including partial ticks, rollback/high-water handling and publication
failure/retry. Closed-game posture never adds occupant contact. In-game pause and
terminal state freeze age along with other live simulation.

## Bounded rendering and verification

The ship keeps its 80-byte vertex stride and uses eight fixed age floats in its
608-byte uniform (32 additional bytes). Fixture records grow from 64 to 80 bytes,
sharing the same meshes, material responses and mesh batches. Handheld wear uses
an existing spare uniform lane. Aging creates no textures/materials, and fixture
population remains behind the existing explicit 16,384-record renderer guard.
Normal/damaged covers use two shared template batches, independent of population.
Software-renderer timing is evidence for the tested workload, not hardware approval.

Run `cargo test -p salimon-ship --locked surface_age` and
`cargo test -p salimon-client --locked surface_age`, workspace/build/Clippy/fmt and
save/domain regression gates. Tests cover real repeated latch travel, moderate
idle/use, exact restoration, pause/recolor/blackout, new replacement identity,
authoritative damage/clean geometry restoration, lost-cell retention, schema
migration/rejection and partitioned verified offline settlement.

`scenarios/evidence/surface-age-*.json` capture original production game views.
The used locker seed performs 600 real open/close cycles through production
clearance/motion; idle seeds one day of passive exposure without advancing world
survival. Damage/repair seeds a funded completed project and a declared #549
impact; repair uses the production construction command. Replacement dismantles a
completed stool, seeds the exact recovered fragment into the carry slot, then uses
production placement/intake/completion for a fresh physical project ID. Its declared
camera clears the existing conservative dismantling envelope. The offline route
submits a one-day interval to the verified-host reconciliation boundary, retaining
the resulting Core depletion and source state. These explicit initial
conditions do not claim an organically elapsed 120-day playthrough.

`SALIMON_FIXTURE_EVIDENCE_DIR=/absolute/output cargo test -p salimon-renderer
--locked modular_fixture_gpu_evidence -- --ignored --nocapture` compares fresh,
aged, damaged, repaired and blackout output, plus 500 independent fresh/aged
fixtures, with repeated frozen frames and raw resource/timing rows.
See [results, captures and limitations](../reports/issue-581/README.md).
Human appearance remains #561; physical platform checks #557; real GPU/Metal
budget qualification #563.
