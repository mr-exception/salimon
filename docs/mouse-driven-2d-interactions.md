# Mouse-driven 2D exploration and interaction contract

**Status:** approved design, **not yet implemented**. Applies to the React/TypeScript browser's 2D planetary surface and Scout interior. Does **not** change the independently controlled panoramic cockpit or isolated radar fixture. Implementation: [#890](https://github.com/mr-exception/salimon/issues/890)–[#894](https://github.com/mr-exception/salimon/issues/894).

## Intent and integration

Salimon's existing world has WASD walking, swept collisions, doorway frame transfer, fixed-tick updates, a smoothing follow camera, resource gathering, a Resource Container and cockpit interaction. Add a primarily mouse-driven control layer **on those systems**, not a parallel player simulation, separate interior scene, new persistence model or new camera in the cockpit. Interactions use stable target IDs and authoritative frame-relative spatial data.

The owner approved **Option A**: right-click on any interactive visible object **always opens a menu**, never immediately triggers its default action. Right-click on ordinary navigable ground commands movement.

## Controls

| Input | 2D effect |
| --- | --- |
| Right-click valid ground | Move to destination using a navigable route; replace previous queued route/action |
| Right-click interactive object | Open or replace context menu; no automatic movement or action |
| Left-click menu action | Dismiss menu and run selected action (including approach when required) |
| Left-click outside menu | Dismiss menu without issuing movement |
| WASD | Direct movement; interrupts queued movement or approach and work when required by existing domain rules |
| Mouse wheel over world | Bounded camera zoom, not ship camera/throttle |
| Middle-button drag over world | Temporarily pan the 2D view |
| Space while world is focused | Recenter/follow player; never activate while typing or using a dialog |
| Escape | First dismiss topmost menu/dialog under existing focus priority; otherwise cancel pending route/approach/interruptible action |
| E / existing accessible action UI | Preserve current keyboard interaction as fallback during migration |

Do not intercept clicks/wheel over HUD, inventory, dialog, editable field or browser chrome. Suppress browser-native context menu only inside the interactive 2D world. Desktop mouse/keyboard is the supported input; no mobile/touch scope.

## Ground movement / navigation

1. Convert the pointer from viewport/SVG coordinates through the **current 2D camera transform** into the proper ship-local or planet-fixed point. Zoom, pan, resize and device pixel ratio must not corrupt targeting.
2. Reject blocked/out-of-world destinations with comprehensible feedback. If a nearest valid approach point is substituted, visibly distinguish it from the requested point and ensure it is genuinely reachable.
3. Plan a collision-aware route using the existing avatar clearance and true world collision geometry (hull, walls, fixtures, door, terrain). Route around obstacles; never teleport, tunnel, pass through closed doors, or create a second collision authority.
4. Follow waypoints on deterministic fixed steps, with stable arrival tolerance and no jitter at corners. Replan or fail safely when a dynamic obstacle/door changes. Prevent unbounded replan loops and stuck movement.
5. Clicking another ground point replaces the current command. WASD, Escape, focus/scene loss, an applicable gameplay interruption and new mutually exclusive action cancel it. A world click while a context menu is open dismisses it and commands ground movement only if it is a right-click on navigable ground.
6. Ship-interior and planet-surface transitions use the existing doorway crossing/anchor conversion. No exterior navigation while airborne, and no navigation through a locked aft door.
7. Persist ordinary location/checkpoints via the existing ResourceSession rules, **not** queued routes, waypoints, selected UI actions or camera offsets. Reload never resumes a pending command.

## Context-menu behavior

- Context hit-testing uses stable object identity and collision/render presentation surfaces, deterministic overlapping target priority and actual visibility. Unseen or unknown objects cannot be remotely commanded merely because they have stored world coordinates.
- Pointer menu is viewport-clamped and keyboard accessible with clear labels, disabled states, reasons and focus restoration. Opening it does not pause the game.
- A right-click on another object replaces the menu; right-click ground closes it and starts movement; outside left-click/Escape closes it without execution. Loss of object/scene/focus invalidates the menu.
- **Inspect** can be remote when the object's approved information is already known; no unearned discovery or remote physical effects. Proximity-required actions explicitly say they will approach first.
- Action definitions are typed and reusable: stable action ID/label, availability/reason, required range/target frame, tool/authority/capacity prerequisites, interrupt policy, execution handler and feedback.
- Show disabled but meaningful actions with explanations; never show unimplemented Core/Fabricator/other future mechanics as usable. Examples: resource Mine/Gather, container Operate/Store/Retrieve, cockpit Use, door Open/Close, Inspect and future pickup. Exact actions are derived from actual object capabilities.

## Approach and execution lifecycle

`selected → route → approach → revalidate → execute → complete`, with terminal `cancelled` and `failed` paths.

- Selecting a menu action that requires proximity resolves an accessible interaction point **outside** the target collider, within actual range, and routes there; it never attempts to walk into the target sprite.
- Revalidate on selection and immediately before any side effect: stable ID still exists; target remains known/reachable, current frame/ship phase is valid, tool/ownership/power/capacity still suffice, player remains in range and action stays legal.
- Long-lived work rechecks relevant invariants on each domain tick; a single click must not duplicate a transfer or re-trigger a completed command.
- New movement/action replaces queued approach. Existing mining interruptions, explicit confirmation for container transfers, cockpit seat ownership, door locking, survival/pause and noncancelable launch/landing remain authoritative. Do not bypass a panel's confirmation step.
- Inaccessible/removed targets, collisions and state changes must fail with concise feedback; no silent teleporting or hidden queued action after reload. Pending command is session-local; durable transactions commit only through existing atomic domain APIs.

## Camera

Extend, do not replace, the 60 Hz smoothing follow camera already owned by the top-down runtime. Character stays near center while moving. Wheel zoom stays in bounded, legible range with reliable pointer/world transforms; middle drag pans independently of player position. **Manual pan suspends recentering at the offset until Space or the next movement/approach command**, which smoothly restores follow. Resize and ship/planet handoff must not cause jumps or change gameplay coordinates. Camera offsets/zoom remain presentation-local, never persistent world or cockpit state. Focus/pointer release prevents stuck panning.

## Feedback, accessibility and tests

Use the shared Expedition Interface HUD tokens for a restrained destination/arrival marker, active command label (e.g. Moving to deposit / Mining), cancel affordance and short unreachable/invalidated notice. Avoid blocking world input with HUD overlays.

Cloud-first unit tests: projected hit target precedence, click/zoom/pan transforms, path blocking/nearest-point behavior, replanning/cancel precedence, frame conversion, execution state machine, stale target, equipment/capacity/range revocation, and exactly-once domain commits.

Built-production browser/Playwright: actual right-click outdoor path around hull/crate and door, context menu always opens without action, keyboard focus/Escape/outside dismissal, remote deposit action followed by approach and gathering, container confirmation, cockpit entry, camera follow/zoom/pan/recenter, resizes, blur, save/reload, blocked/closed door, airborne exterior and uninterrupted WASD fallback. Capture screenshots/trace evidence per [acceptance policy](acceptance-policy.md); do not silently approve or weaken existing image baselines.

## Delivery sequence

1. [#890](https://github.com/mr-exception/salimon/issues/890): pointer-to-world mapping, path planning and click movement.
2. [#891](https://github.com/mr-exception/salimon/issues/891): follow/zoom/pan/recenter; may proceed in parallel with #890, integrate transforms before acceptance.
3. [#892](https://github.com/mr-exception/salimon/issues/892): action registry and accessible right-click context menus; can proceed alongside movement.
4. [#893](https://github.com/mr-exception/salimon/issues/893): unified approach/execution, depends on #890 and #892.
5. [#894](https://github.com/mr-exception/salimon/issues/894): HUD, help, quality and end-to-end integration, depends on previous steps.

This document is a **future design** until issues have code-backed evidence and are merged.
