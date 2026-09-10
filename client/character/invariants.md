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
