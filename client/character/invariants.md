# Character Invariants

1. Ship-floor and solid-body walking use one fixed 9.81 m/s² gravity strength.
2. Ship gravity completely owns the character while the controller is inside or
   seated, irrespective of the nearby planetary surface.
3. A closed door, or any flying ship, blocks passage in either direction. Closing
   during a doorway blend ends it before movement and resolves overlap to the
   nearer physical side; an existing surface walker always remains outside.
4. A landed open-door crossing blends gravity direction for exactly 250 ms.
   WASD stays camera-relative throughout the blend; re-entry requires movement
   toward the cabin rather than a particular key.
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
11. The central Core pedestal, sofa, worktop, pilot chair, and cockpit
    console/monitor assemblies are solid to walking movement, with body-radius
    clearance and edge sliding; both side routes into the cockpit remain
    traversable.
12. Rear window bulkheads remain solid with the door open. Doorway crossings
    require enough lateral clearance for the player's complete body, including
    oblique movement throughout the gravity blend.
13. Spawn and cockpit exit place the player in the starboard aisle without
    intersecting the Core, chair, or walking boundary.
14. Jumping keeps the complete player body below the ceiling fixtures and the
    lower door lintel; reaching the ceiling cancels upward speed.
15. Surface W/S movement follows the camera's horizontal forward/back direction,
    A/D follows its tangent-plane right/left direction, and all four directions
    remain tangent to the active solid body at every sphere orientation.
16. The cockpit uses its actual forward deck boundary plus solid side-hull
    proxies; a walking player cannot bypass the side consoles into the exterior
    shell.
17. Surface walkers collide with the front, sides, and aft bulkheads regardless
    of door state. The landed open aft gate is the only entry route, with the
    same body clearance as interior traversal. Hull contact preserves surface
    eye radius and permits sliding; distant or vertically separated surface
    walkers cannot be captured by the doorway.
18. Closing around a surface walker in the gate clears the overlap before
    walking or jump input is processed. A closed gate cannot grant ship gravity,
    interior state, or cockpit entry. Repeated reopening restores the ordinary
    250 ms blend and body-clear passage in either direction.
