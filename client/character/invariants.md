# Character Invariants

1. Ship-floor and solid-body walking use one fixed 9.81 m/s² gravity strength.
2. Ship gravity completely owns the character while the controller is inside or
   seated, irrespective of the nearby planetary surface.
3. A closed door, or any flying ship, prevents transition out of the interior.
4. A landed open-door crossing blends gravity direction for exactly 250 ms.
5. Surface positions remain at body radius plus eye height and support traversal
   around the complete sphere.
6. Cockpit entry/exit is instant and does not mutate ship velocity or orientation.
7. The crate receives no native events and exposes no GPU types.
8. Positive horizontal mouse delta turns toward the camera's screen-right direction;
   vertical mouse delta retains the conventional down-is-positive device mapping.
9. The default seated cockpit view remains primarily forward and clears the
   console/nose into the modeled cockpit glazing.
10. Ship-local floor, walk bounds, doorway, player start, and cockpit
    camera remain aligned to the wider 4.00 m-tall asset; player body height is
    1.80 m and standing eye height is 1.75 m.
11. The central Core pedestal, sofa, and worktop are solid to walking movement,
    with body-radius clearance and edge sliding; both side aisles remain traversable.
12. Rear window bulkheads remain solid with the door open. Doorway crossings
    require enough lateral clearance for the player's complete body.
13. Spawn and cockpit exit place the player in the starboard aisle without
    intersecting the Core, chair, or walking boundary.
14. Jumping keeps the complete player body below the ceiling fixtures and the
    lower door lintel; reaching the ceiling cancels upward speed.
15. Surface W/S movement follows the camera's horizontal forward/back direction,
    A/D follows its tangent-plane right/left direction, and all four directions
    remain tangent to the active solid body at every sphere orientation.
