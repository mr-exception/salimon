# Salimon Expedition Interface — HUD and equipment UI art direction

**Decision:** Owner-approved 2026-11-07, **hybrid industrial equipment + projected instrumentation**.
**Status:** Approved presentation/style contract for the active React/browser client. The #826 shared foundation supplies scoped tokens, typed primitives and development fixtures; **adoption in individual production views remains future work**. Existing production UI continues to operate until its own implementation issues are merged and verified. Screenshot generation does not approve a visual baseline.

## 1. Identity and authority

**Equipment feels physical; information feels projected.** Salimon is a grounded, solitary science-fiction expedition. The environment, character, Scout and celestial phenomena remain the artwork; UI is a legible instrument supporting the player's understanding and actions, not a second spectacle.

Two related surface families share typography, semantic colors, spacing, icon grammar, focus, state and audio:

- **Industrial equipment surfaces:** substantive dark graphite/slate opaque panels with small ochre/mechanical details for Resource Container, Core, Fabricator, suit/equipment and Archive. Each machine's data layout reflects its actual mechanics.
- **Projected instrumentation surfaces:** lightweight restrained dark/translucent overlays with cyan functional accents for world HUD, scanner, navigation and contextual cues. Projection does **not** mean strong glow, hologram scan-lines or a permanent grid.

The present **strict overhead illustrated 2D** world remains default. Production cockpit mode is the **ship-centered 360° panoramic sensor**; the orthographic radar remains an isolated engineering fixture. Do not turn planets/interiors into 3D or return to the old production lattice/vector rays. See [2D art](top-down-art-direction.md), [panoramic navigation](panoramic-navigation-design.md), [experience direction](experience-direction.md), and [actual browser status](current-client-status.md).

**Precedence:** This document governs shared *presentation*. Owning gameplay, state, input, scanner, navigation, storage, survival, cockpit and accessibility contracts still govern behavior. If a visual recommendation conflicts with a current approved mechanic, preserve the mechanic and document a presentation adaptation rather than silently changing it. Never infer implementation from approval.

## 2. Design tokens (initial implementation targets)

Colors below are **sRGB UI design tokens**, not linear-space shader material constants or a rewrite of the [world art palette](visual-style.md). Tune for actual scene/background contrast during screenshot review.

| Role | Starting value | Meaning |
| --- | --- | --- |
| `--ui-void` | `#09141C` | Opaque dark background and dim canvas only where needed |
| `--ui-panel` | `#192B30` | Substantial equipment dialog/panel surface |
| `--ui-surface` | `#314E52` | Buttons, secondary fields and inset instruments |
| `--ui-text` | `#E8E1D1` | Principal readable UI text; avoid pure white everywhere |
| `--ui-muted` | `#AFC2C8` | Secondary text; verify contrast before shipping |
| `--ui-active` | `#69D3DD` | Active systems, focus, selected instrument information |
| `--ui-mechanical` | `#EBC076` | Mechanical affordance, physical emphasis, caution context |
| `--ui-danger` | `#E77F73` | Danger/destructive actions **with text/icon** |
| `--ui-heading` | `#FFE36A` | Panoramic compass ship-heading axis (reserved role) |
| `--ui-velocity` | `#B6F5A2` | Panoramic compass physical-velocity axis (reserved role) |

Do not communicate condition/severity by hue alone. Do not use heading/velocity colors as generic action success colors. No global bloom, perpetual pulsing, blinking decoration or blanket blur filters; a small highlight/emissive accent can mark an actual active device. Semi-transparent projected backgrounds must be dark enough over both bright ground and near-black space; support stronger opaque backgrounds in accessibility settings if needed.

**Type:** Inter/system sans-serif fallback for prose and labels, tabular numerals or restrained mono for telemetry/quantities. Body 14–16 px, secondary 12–13 px, large readouts 18–24 px, specialized captions 11–12 px only where truly supplementary. Uppercase, tracked micro-labels are for short machine/section identity, not full paragraphs. Use locale-safe numeric formatting and explicit units (kg/g, HP, m/s, reference frame). Display rounding never changes authoritative integer grams or a transaction.

**Shape:** 4 px spacing unit, preferential steps of 8/12/16/24 px; 1 px subtle boundaries; 4–6 px functional corner radii, optional clipped-corner treatment only where implemented consistently. No excessive pills, glossy gradients, skeuomorphic screws, bevels on every surface, or heavy shadows. Primary controls target ~40–44 px tall and clearly visible focus.

**Icons:** One coherent outline icon style, same stroke and size hierarchy. Pair obscure icons and high-stakes controls with text. Keep silhouettes and hit targets legible at common desktop zoom. No icon-only representation of critical hazards.

## 3. Common primitives and states

Reusable presentation primitives should include `Button`/`IconButton`, `EquipmentPanel`/`Dialog`, `UtilityPanel`, `Meter`/`Progress`, `KeyHint`/`ContextPrompt`, `StatusMessage`/`Warning`, form `Input`/`Select`/`Slider`, `Tooltip` and `Notification`. These are **component concepts**, not mandated existing API names.

- Primary buttons: solid restrained surface and thin illuminated action/focus treatment; secondary buttons: light neutral outline; destructive buttons: unmistakable text/icon plus confirmation if irreversible.
- Hover: subtle color/border change; press: immediate short downshift; focused: at least 2 px visible ring with sufficient contrast. Disabled state includes explanation when reason is non-obvious, never merely opacity/hue.
- Toggles show explicit on/off wording; sliders expose current numbers and meaningful endpoints; progress conveys exact value and process label. Never animate a success before authoritative confirmation.
- Use **micro surfaces** for short prompts/tooltips, **utility panels** for inventory/contacts/telemetry, and **equipment dialogs** for meaningful multi-step machine operations. Avoid turning every action into a blocking full-screen modal.
- Equipment dialog structure: identity/room eyebrow, device name, current state, specific machine content, actions, inline errors/confirmation, dismiss. Share the shell but not generic placeholder contents.

**Equipment-specific layouts:**
- Resource Container: available and carried kg, 10 kg/12 slot player capacity and 50 kg raw-only storage, exact positive-gram input, review → confirm deposit/withdraw; preserve previously committed transfers on Escape, interruption or focus loss.
- Fabricator: future recipes, ingredients, queue, power/blocked status, output tray; no invented ready/consumed state.
- Core: future fuel reserves/conversion/power only from actual state; do not animate a fictional conversion cycle.
- Suit/equipment: future installed-versus-stored modules, weight/slots, protection and durability as approved; do not imply equipment UI or catalogs already ship.
- Archive: recovered records and signal/context; quiet editorial text and no undiscovered information or story spoilers.
- Help/settings/pause: the same typography and control language, **not** fake in-world equipment.

## 4. 2D world: adaptive compact HUD

**Decision: adaptive compact, not always-expanded or aggressively auto-hidden.** The overhead game continues to fill the browser viewport; no sidebars, decorative header/footer strips, body scroll, phone-specific controls or camera repositioning purely to fit HUD. The player and important nearby objects remain readable.

Preferred anchors (adjust responsive positions to prevent overlap; never reserve scene layout width):
- **Upper left:** persistent compact **Health / Hunger / Fatigue** indicators with identity/labels and accessible values; critical state increases prominence. No forced always-expanded numeric detail or status inventory.
- **Upper right:** persistent condensed **carried weight / slots** and current tool status when relevant. Full raw/count item list opens on demand in a utility panel; no always-on full list.
- **Lower left or near target:** one selected/nearby **contextual action** with actual key name and current state; hide when no valid action. Do not repeatedly render a large disabled action.
- **Contextual region:** gather progress and source identity while gathering; transaction/error/save feedback near the relevant activity; serious survival/hazard warning has priority.
- **Temporary overlays:** equipment dialog, inventory detail, help and pause, with an explicit focus and occlusion policy.

Preserve the actual `TopDownView` gameplay and [full-viewport 2D contract](playable-2d-foundation.md#viewport-and-hud-803), including accurate SVG-world-to-screen positioning, ship-up orientation and keyboard reach/targeting. Current 200 HP maximum, 10 kg/12 shared slots, integer-gram quantity handling, hunger/fatigue and terminal run semantics remain authoritative. A visual meter does not create new hazard effects, automatic healing or pause behavior. No invented "HUD off" mechanic that hides the last warning.

## 5. 3D panoramic cockpit: precision sensor HUD

The *production* 360° optical sky must fill the viewport; this style layer **does not add** a hull/cockpit model, body lattice, off-screen target rays, heading/velocity rays or a permanent side drawer. [Panoramic navigation](panoramic-navigation-design.md#compact-full-viewport-hud) and the delivered #814/#815 functional layout govern.

- Always on-screen: small cyan **observation reticle**, scanner state/contact count/pulse, speed with explicit frame, flight/grounded status, throttle access, mini 3D orientation compass, and active critical warnings.
- Compass semantics immutable: cyan = camera/view axis, yellow = ship heading, light green = velocity; distinguish front/back with shape/fill, handle overlap, and suppress velocity direction when stationary.
- Searchable/paginated contacts default collapsed; a selected **scanned** contact opens small identifying/actions UI for separate `Look at` (camera) and `Set direction` (ship), subject to authority.
- Expanded telemetry/help are on demand, not permanently layered over celestial observation. Compact survival status remains available during cockpit use without competing with navigation's primary instruments.
- Scanned, off-screen or optically occluded contacts stay discoverable in the contact browser, but do **not** gain through-body world markers. Visible unscanned bodies must never gain name, identity, tooltip or selectable commands.
- Preserve inertial camera, independent ship movement, 75° default/0.1° minimum FOV, wheel observation zoom, scanner 5-second pulse/100,000 light-second radius, grounded gates, cockpit seat/power transitions and local camera persistence. No new flight mechanics.

## 6. Motion, response and sound

These are **targets to validate**, not gameplay timers:

| Event | Initial visual timing | Rule |
| --- | --- | --- |
| Press/release | 70–110 ms | Input action never waits on animation |
| Hover/focus | 100–150 ms | Focus visible immediately |
| Tooltip | 100–140 ms | Readable without dependence on motion |
| Utility panel | 160–220 ms | Nonblocking and cancellable |
| Equipment dialog | 180–240 ms | Small fade/translation, no theatrical zoom |
| Brief confirmation | 200–350 ms | Only after committed success |
| 2D ↔ cockpit presentation | ~200 ms | Preserve all authority/lifecycle rules |

Never simulate a machine state from client-side display timing. Animations must be interruptible, lightweight and reduced/disabled with `prefers-reduced-motion`. Do not continuously pulse status bars or shake/blur the camera. Critical warnings are persistent/clear until no longer relevant, not repeatedly flashing.

Sounds: restrained neutral press, short mechanical relay for actual machine operations, quiet accepted-transaction acknowledgment and distinct critical alarms; avoid UI chatter or perpetual sci-fi beeps. Every signal has a visual/text alternative and mute/reduced-feedback support. Do not synthesize audio claims without a real playback implementation. Retain the game's silence/atmosphere.

## 7. Input, focus, feedback, and failure edges

1. **Input authority:** Any focused dialog/form/input blocks flight, camera, walking, mining and shortcut side effects. Keyboard-specific controls are shown as actual bindings, not decorative hints. Clickable HUD bounds intercept pointers; the rest of scene remains interactive.
2. **Escape order:** Dismiss topmost modal → expanded utility panel → contextual selection as appropriate, before invoking view-specific Escape/exit/cancel. Preserve existing cockpit normal exit, power-loss seated fallback and 2D interaction cancellation. Never make a closed modal reopen from a held key.
3. **Time:** Ordinary menu openness does not itself freeze survival/ship simulation. The deliberate Pause action retains its owning clock rules. Blur/background follows existing checkpoint/input release behavior; no accidental completion of mining or transfers.
4. **Transactions:** Review/confirm/reject state and precise grams are visible; no optimistic success banner before commit. Interrupted uncommitted draft cancels without rollback of prior committed resources.
5. **Warnings:** Priority order: terminal/critical danger > actionable error > active work feedback > routine status. Avoid hiding critical warning behind contact list, dialog or inventory panel; preserve actionability and screen-reader alert without repeated live-region announcements each simulation tick.
6. **Small desktop windows:** Collapse secondary detail rather than clipping controls, relocate or scroll *within* panels, and retain full scene occupancy; supported keyboard interaction works at 960×600 and 1280×720 as well as 1920×1080. No mobile requirements.
7. **Power/unavailable:** Distinguish disabled/grounded/seated/paused/unpowered states using text and icons; no fake physical failure or unauthorized ship actions. The only existing Core is a future/incomplete machine integration; UI style must not claim otherwise.
8. **Contrast/screen readers:** Verify contrast in real captures over bright and dark scenes, logical heading/landmark/label order, visible focus, keyboard-only dialog entry/return, numeric values alongside bars, non-color-only dangers, accessible tooltips, and concise aria-live notifications (deduplicated).
9. **Persistence:** Visual widget expansion state may be local presentation, but stateful gameplay remains with its domain owner. Style changes cannot change save schema, scanner entitlement, combat/collision, equipped contents or world physics.

## 8. Suggested implementation breakdown and dependencies

Documentation approval **does not** deliver any of the following:

1. **P0 Shared foundation:** CSS token map + UI primitives and behavior/state fixtures. Reuse current React/TypeScript and CSS; no heavyweight framework or parallel renderer.
2. **P0 Adaptive 2D HUD:** refactor survival/inventory/context to compact layers while preserving already delivered gameplay. Extend details only when requested or relevant.
3. **P0 Panoramic visual integration:** apply shared tokens/components to delivered compact panoramic HUD (#814) and grounded cockpit (#815) **without rebuilding their 3D camera/scanner logic**.
4. **P1 Equipment proof:** migrate the existing Resource Container modal to common industrial style while preserving its exact-gram transfer workflow (#785).
5. **P1 Unified status/interactions:** keyboard/focus/Escape safety, feedback severity, optional sound and reduced motion across views; avoid duplicated feature ownership.
6. **P1 Verification:** deterministic UI gallery/state fixture, real browser 2D/cockpit/modal screenshots, visual diff checks, focus/keyboard and failure tests, reported evidence and explicit owner baseline review.

Future Core/Fabricator/Archive/suit-specific surfaces are **style application tasks attached to those feature owners**, not fictional new machine gameplay. Coordinate with older native #466/#373/#503 as historical design context only; their closure does not mean a browser-wide UI system was shipped.

## 9. Acceptance / review matrix

- **Readability:** contrast over authored bright/dark scene captures, normal zoom, tabular units, distinguishable icon/color states and retained critical alert priority.
- **Viewport:** 960×600, 1280×720 and 1920×1080 desktop; 2D world and panoramic sky reach every edge, no body scroll, clipping or obstructed character/controls.
- **State gallery:** default, hover, focus-visible, disabled-with-reason, pending, accepted, rejected, hazard, terminal, empty inventory, capacity full, scanner zero contacts and selected/occluded/lost contact.
- **Behavior:** focus lock, Escape hierarchy, pause versus menu, grounded cockpit, power interruption, manual observation versus ship steering, transaction interruption and authoritative HUD updates.
- **Accessibility:** mouse, keyboard-only and reduced-motion passes; meaningful accessible names/live regions, no critical color-only signals.
- **Regressions:** existing 2D collisions/navigation, scan permission/occlusion and save tests remain effective. Use the repository's Node/TypeScript tests, build, Playwright/Chromium and original screenshot captures with explicit baseline approval.
- **Evidence:** each implementation issue reports precise commands, viewport/DPR, SHA, actual PASS/FAIL/NOT RUN and unresolved human/device qualification. Cloud software rendering cannot assert native GPU performance.

## Shared foundation implementation (#826)

`client/src/ui/tokens.css` is the single scoped DOM token source; `primitives.tsx`
and `primitives.css` provide typed Button/IconButton, EquipmentPanel/UtilityPanel,
Dialog, Meter/Progress, KeyHint/ContextPrompt and StatusLabel. Wrap consumers in
`.expedition-ui` (including dialog ancestors); imports intentionally have no
`:root`, body, canvas or scene selectors. The palette remains dark/opaque over
both bright planet and dark space contexts. Heading/velocity tokens are reserved
for compass consumers. These are sRGB display inks, never renderer materials.

Buttons accept `variant`, `active`, `pending` and `disabledReason`; blocked controls
remain keyboard reachable with a visible, associated explanation and cannot invoke
the supplied click action. Native button labels and attributes remain available.
Owners still validate every command. Status labels include explicit words; only
meaningful event changes belong in live regions. Meter/progress display values,
maxima and units, defensively bounding malformed readings without writing state.

Dialog is controlled with `open`/`onDismiss`, uses native modal inertness, labelled
initial focus, Tab wrapping, Escape dismissal, focus return and bubbling keyboard
isolation. Mount it beneath the token wrapper. Keep critical warnings and the
owner's input-release/focus gates when integrating it into a game screen; mounting
a shell never pauses simulation or authorizes actions. No screen migration ships
with this foundation. Input/Select/Slider, Tooltip and Notification concepts from
section 3 remain future extensions, outside #826's requested primitive set.

The development-only `/tests/ui.html` gallery contains illustrative states in both
contexts. `tests/ui.spec.ts` verifies keyboard/blocked actions, readable names and
units, reduced motion, viewport bounds and byte-identical repeated originals.
Run `cd client && npm test && npm run build && npm run test:browser`; focused
iteration uses `npx playwright test --project=ui`. Captures are **candidates for
human review**, not approved visual baselines. See the [#826 report](../reports/issue-826/README.md).

## 10. Explicit non-goals and delivery state

No new UI framework, physical 3D cockpit art, mobile interface, simulated machine operation, new game resource, gameplay balance change, authoritative backend, save migration, scanner discovery change, unscanned names, automatic pause, or photorealistic fidelity requirement.

**Approval applies to design direction.** Each view's actual rendered result needs its own implementation PR, tests and screenshot review; one document cannot certify production styling. Existing #803 fullscreen 2D, #785 storage transfers, #814 panoramic compact HUD and #815 cockpit integration remain delivered as their own distinct feature scopes.
