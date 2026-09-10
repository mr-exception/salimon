# Character Architecture

The controller stores one portable position state: cockpit, ship-local interior,
world-space doorway blend, or world-space surface. Ship-local positions move with
the ship without copying ship simulation into this crate. Doorway and surface
states use world coordinates so the gravity transition and radial projection are
explicit.

`MovementInput` is the typed platform boundary. `ShipFrame` and `SurfaceFrame`
are read-only environmental inputs. `CharacterSnapshot` is the presentation and
diagnostics output. The runtime maps these types to/from ship and renderer DTOs;
neither dependency points back into character.
