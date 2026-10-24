# Physical room finish controls (#579)

`salimon_ship::finishes` owns bounded cosmetic source: five independent groups
(walls, floor, trim, cabinetry, upholstery) per durable room ID, plus overrides
keyed by exact installed furniture ID. The Scout cabin is room 1 and the cargo
compartment room 2. Finish identity is independent of connected-space topology;
opening a divider never merges the histories. New physical rooms register a new
identity with original finishes. Placeholders grant no installed room or finish.

Aim at the galley controls or the cargo-door controls from the relevant side:
V selects the group, O cycles Original / Slate / Ochre, Z restores the original.
Aim at the bunk, padded seat, table or locker to use O for that individual item
and Z to inherit its room group. The contextual bottom prompt names the selected
group/style. E retains the normal physical interaction. No modal or held input
captures flight, pause, Q-wheel or carrying controls. Controls require nearby
interior aim through the existing production interaction admission; blueprint
and modal UI gates remain authoritative. Controls work in Core blackout.

Recoloring is immediate/free cosmetic tint only. It never changes physical
material, roughness, geometry, mass, resource quantities, energy, component age
or damage. Physical replacements remain resource-fed construction with a new
part identity. Removing an item removes only that exact override through the
explicit registry API; no new ID inherits it. The current registry does not
simulate wear or damage, nor claim that future room construction (#545) or damage
(#549) has shipped. Portable structural tests create two real room records and
remove their shared wall while retaining both independent finish records.

## Rendering and source validation

The approved authored materials remain the baseline. Three curated sets of linear
factors, bounded to [0,4], preserve each surface's material and clamp cosmetic RGB
to [0,1] before lighting. Original is an exact identity factor. The ship importer
uses an explicit node allowlist for the five groups. Screens, glass, Core status,
functional/warning colors, light emission, restraints, rails and clamps are excluded.
The two room palettes occupy ten vec4 values in one shared ship uniform (352 bytes
total). One group attribute increases the ship vertex stride from 76 to 80 bytes;
fragment-local room selection keeps a sharp cargo boundary. No materials, draw
batches or textures are created by recoloring. Fixture overrides use the existing
48-byte instance records and immutable shared meshes. A seat's pad/back share
one installed item ID; its structural frame retains the authored finish.

Whole-session schema 13 stores the bounded registry; schema 1–12 migrates to
original finishes and rejects incompatible custom source. Restore validates item
association and palette bounds before publishing. Selection of a group is transient
UI; actual color changes are discrete checkpoint events. Exact save/restore tests
preserve physical source and custom colors without ticking or spending resources.

Future modular rooms supply the same stable room/part identities to the registry
and renderer during actual installation/removal; no spatial connected-component
recoloring is permitted. This boundary does not generate physical geometry.
See [the report](../reports/issue-579/README.md) for actual native captures/resource
counters and separate human appearance/hardware limitations.
