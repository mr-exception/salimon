# Biological harvesting and source deltas (#621)

World `environment::harvest` publishes explicit yield designations against the
exact immutable physical Population. Each designation names one stable organism /
part key, method (Detach or Embedded) and existing raw-material key. Life, palette,
cosmetic cover and a scanner result never grant a yield. Root part 0 cannot be
harvested; detachable parts must be terminal anatomy. Catalog identity binds the
population, body and sorted designations. Hash-derived deposit handles are checked
for collisions within the publication; stable part identity remains separate
from the allocated physical fragment handle.

Vesper's frozen publication remains sterile, with an empty yield catalog. The
explicit `biological-harvest` / `biological-mining` fixtures publish one fictional
frond's biomass appendage and a separate medicinal reserve against representative
Moon geometry. This is declared fictional authoring, not life or medicine on the
observed Moon or Vesper, a catalog-body adoption, or a new world selector.

## Input, transfer and conservation

Empty-handed F on a reachable designated attached part grips, stows the tool,
and shows a 250 ms separation/tension animation. The target must remain the first
unobstructed physical anatomy within the existing 3 m pickup range. Hull, terrain,
construction, other anatomy and loose objects occlude. Focus/input reset, pause,
view change, lost aim/reach or an occupied carry slot cancels before commit.
Pending grips are transient; saving/loading them restores an intact source.
Tool selection is locked while gripping and while carrying. F while carrying
continues to drop that physical entity; no extra equipment or inventory system.

The commit preflights method, revision, carry slot, geometry, size, mass and
allocator before source removal and object publication in the same world owner.
Original source/part identity, tapered endpoints/radii, color, species material,
raw material and mass survive in `ResourceFragment::natural_shape`. Its session
fragment handle is allocated once and retained through carrying, dropping,
streaming, consumption and checkpoint reopen. No additional sphere is drawn for
an intact biological part. This is transfer of the one named anatomy entity,
not a duplicate resource grant alongside a still-attached source.

Natural mass is the six-sided tapered-prism volume times the explicitly designated
material's effective density. The legacy enlarged mineral-sphere appearance does
not enlarge intact anatomy. Provisional natural-part carrying bounds are 0.75 m
circumsphere radius / 20 kg; larger parts reject without debit or allocation.
Ordinary mineral carrying rules remain as before. Natural anatomy supplies real
conservative bounds to carrying and loose physics; terrain/deck and object contact
retain the existing solver. Static organism contacts use transformed tapered-part
bounds; moving-object/terrain support uses a conservative circumsphere. Narrow
clearances can therefore reject; exact convex tissue contact/soft bodies are not
claimed. Held objects require a clear end pose and sight path, as the existing
carry adapter does for ship obstructions.

Embedded reserves use the existing equipped mining tool and left click, spherical
capacity/rate/interrupt policy, reserved/released object IDs and source-depletion
storage. `MiningSession::mine_biological` composes the debit and the original
spherical operation under an expected source revision. Zero time, rejected or
stale requests mint nothing. Removed mass equals reserved plus released material
within the existing numerical tolerances; interruption releases existing physical
output and never refunds or recreates source mass. Separate organ material keys
are authored only where the source explicitly yields them.

## Persistent and shared consumers

Journal entries bind the catalog and key, positive revision, remaining mass and
detached physical handle. Compare-and-commit revisions reject stale future-authority
requests. This is a portable transaction boundary, not a networking implementation.
No regrowth is scheduled. Missing parts disappear; partial reserves shrink their
radii by the square root of remaining fraction, retaining mass-proportional
volume. Collider envelopes, weather flex, sound eligibility and personal physical
site inspection all use current anatomy. Renderer quality, cache eviction and
streaming cannot restore removed parts or material.

Schema **29** captures the publication-bound journal and explicit fixture/world
context. Versions 1–28 migrate to empty biological deltas; they cannot carry
new biological state. Unknown publication, duplicate keys/handles, invalid shape,
material/mass/allocator mismatch, uncommitted natural objects and inconsistent
embedded mining debit reject before replacing the live session. Existing physical
carry/motion, crafting, mounted cargo and exact canonical world validation remain
in force. Consumed detached handles remain historical tombstones, so source
anatomy cannot be minted again after Core/Fabricator use. Pending animations,
audio contacts and collision caches remain excluded from authority/save payloads.

Sound follows [the environmental audio contract](planetary-audio.md): actual
successful separation emits a restrained material contact observation, not held
input or failed/replayed commands. Missing anatomy no longer rustles. The explicit
fictional fixture samples its own climate and translated body-fixed listener;
normal Vesper retains exact atmospheric silence. No fauna, damage, terrain
destruction, scientific identification, new scanner timing or automatic objective.

## Verification

World harvest tests cover atomic transfer, stable source/part identity, stale
revisions, occupied/too-large rejection, spherical rate/cap/depletion, interrupt,
conservation and reopening. Runtime tests exercise real F/tool/controller gates,
focus/pause/aim/range, mid-grip save, natural shape rendering and malformed atomic
restore. Native evidence scenarios use the same commands and production wgpu
under Xvfb/Mesa. An opt-in actual-separation audio export passes these observations
to the production continuous callback. See [completion evidence](https://github.com/mr-exception/salimon/blob/f1cd0b3e878fac487ce898547a0c8c35aa40519b/reports/issue-621/README.md).
Hardware/native input, subjective art/listening and M1/Metal budgets remain
separate unverified qualification under #557/#561/#563.
