# Task 14 — closed ship door

[Notion task](https://app.notion.com/p/3e0b456853b98166923ff463434bbd69).

## Behavior and regression evidence

The original implementation let a surface character remain inside the closed
gate after a short exit, and let an active doorway gravity blend continue after
the door closed. Both cases were reproduced by focused tests before the fix.
Closing now cancels a doorway blend before movement, resolves a body overlapping
the gate to the nearer side with body clearance, and keeps a surface walker
outside the closed gate. Cockpit interaction also requires the authoritative
character location to be `InsideShip` before granting ship control.

Character regressions cover center and jamb approaches, walking, jumping,
oblique pressure, closure during a blend, three reopen/crossing cycles, and
rotated ship frames at large absolute coordinates. Existing ship tests continue
to cover the flight door lock and the closed-door takeoff requirement. An
independent code review found and verified fixes for two large-coordinate
rounding cases at the gate and jambs.

## Native macOS playtest

On macOS 26.2 arm64, a native validation executable drove the production
`salimon-character`, `salimon-ship`, and `salimon-world` APIs at 16 ms steps
from the normal landed-Earth starting state. It used real movement input and
door/assist actions, without directly setting expected end states. The
[native playtest log](native-playtest.log) records the full result:

- Outside close: 60 walking ticks and 30 jump ticks at the center and both
  jambs stayed `Surface` with no doorway blend; the closed gate held the eye
  near ship-local X = −8.16 m. Oblique jamb pressure also stayed outside.
- Closing on either side during the 250 ms doorway blend resolved to that
  physical side and blocked subsequent crossing while closed.
- Three close, reopen, re-enter, and exit cycles passed through normal
  character movement and ship door state.
- A non-polar rotated Mars landing at a world position near 10¹² m blocked
  90 inward walking/jump ticks while closed and allowed re-entry after opening.
- Open-door takeoff produced `Close door before takeoff`; assisted and normal
  flight refused door opening with `Door locked while in flight`.

The native game window launched, reached `wgpu` renderer initialization, and
closed through its macOS window control. The UI capture remained uniformly
dark, including with the earlier Task 17 build, so this session did not provide
a visual signoff of the rendered door or the wider scene smoke checklist. The
native movement/state playtest above verifies this task's collision behavior;
the visual capture limitation is recorded rather than presented as a pass.

## Automated gates

On the final diff:

- `cargo build --workspace --locked` — passed.
- `cargo fmt --all -- --check` — passed.
- `cargo clippy --workspace --all-targets --locked -- -D warnings` — passed.
- `cargo test --workspace --locked` — 164 tests passed.
- `git diff --check` — passed.
