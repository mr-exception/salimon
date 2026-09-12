# Character Architecture

The controller stores one portable position state: cockpit, ship-local interior,
world-space doorway blend, or world-space surface. Ship-local positions move with
the ship without copying ship simulation into this crate. Doorway and surface
states use world coordinates so the gravity transition and radial projection are
explicit.

The ship interior uses a rectangular walking envelope with a body-radius margin
and three simple planar obstacles for the central Core pedestal, port sofa, and
starboard worktop. Movement resolves
forward and sideways axes separately against that obstacle, preserving sliding
and the clear routes on both sides without a physics engine or mesh collision.
The side bounds account for the projecting window sills. The aft bulkhead stays
solid outside the body-clear doorway aperture, even when the door is open.
Jump height is bounded by the lowest ceiling fixtures and a lower local bound
at the door lintel. Spawn and cockpit exit share a clear starboard aisle position.

`MovementInput` is the typed platform boundary. `ShipFrame` and `SurfaceFrame`
are read-only environmental inputs. `CharacterSnapshot` is the presentation and
diagnostics output. The runtime maps these types to/from ship and renderer DTOs;
neither dependency points back into character.

The seated cockpit snapshot starts with only a slight downward pitch so its Task
10 anchor and forward ray clear the console and solid nose while crossing the
glazing. The 1.80 m body / 1.75 m eye-height contract and ship-local anchors
follow the wider 4.00 m-tall asset. Mouse look remains unrestricted and
independent of ship orientation.
