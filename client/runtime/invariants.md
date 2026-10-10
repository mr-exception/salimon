# Runtime Invariants

1. Window-bound renderer state is created only when the application lifecycle
   permits a native window and is never rendered before initialization.
2. Zero-width or zero-height drawable sizes are not used to configure or render
   a GPU surface.
3. Every accepted nonzero resize reaches the renderer before the next presented
   frame.
4. Frame and update intervals come from monotonic clocks and clamp unreported
   long OS stalls; the renderer never owns or advances simulation time.
5. Recoverable surface loss rebuilds or reconfigures rendering state. A transient
   surface acquisition failure is delayed and does not crash or hot-loop the
   application.
6. A close request or destruction of the sole window exits the event loop
   cleanly. Unrecoverable renderer failures are reported before termination.
7. The runtime may orchestrate renderer/platform capabilities but must not own
   rendering implementation, authoritative domain state, or backend behavior.
8. World, character, and ship code must not receive raw `winit` events or
   `wgpu` resources from the runtime.
9. The engineering overlay is hidden by default and toggles only on an initial
   physical F3 press, never on release or key repeat.
10. Diagnostics observes typed runtime, renderer, and future domain snapshots;
    it never becomes authoritative state or changes simulation behavior.
11. Suspension, occlusion, zero-sized drawables, and renderer reconstruction
    reset the rolling timing window before presentation resumes.
12. Native input is translated into typed portable commands; `winit` event types
    never enter the world crate.
13. Renderer reconstruction preserves portable camera state. Lifecycle gaps
    reset the update clock so the prototype cannot jump forward on resume.
14. Each world snapshot maps exactly six spheres plus three separate noncanonical
    marker cuboids. Sphere centers/radii remain `f64`. The Sun maps to emissive
    material and the point-light position; every solid maps to a lit material.
15. Diagnostics body observations preserve catalog order/names and report finite,
    nonnegative camera-to-nominal-surface distances for all six bodies.
16. Gameplay starts inside the ship landed on Earth; F2 preserves access to the
    earlier precision-tour fixture without changing domain state.
17. Runtime contextual E routing changes cockpit authority or requests a door
    action, but never owns the resulting character/ship behavior. Cockpit entry
    requires the character to be inside the ship as well as in range and aimed;
    the same gate presents `Press E to use`. Exterior and doorway characters
    cannot gain cockpit authority through a spatial interaction hit.
18. Escape releases cursor capture so native window controls remain reachable;
    clicking the game view restores mouse-look capture.
19. Gameplay interaction zones match the authored cockpit and exit-door markers;
    runtime composition does not apply a second scale to the baked ship mesh.
20. Cockpit W/S, A/D, and Left/Right Arrow state maps to typed pitch, yaw, and
    roll without changing the independent character mouse-look state; A turns
    left and D turns right in the rendered cockpit view.
21. Up/Down Arrow changes thruster only on an initial press, never key repeat or
    release; leaving cockpit or losing focus clears held steering input.
22. L is translated to the ship domain only on an initial press; the runtime
    cannot cancel or steer an active assisted landing/takeoff sequence.
23. Character surface traversal receives the body frame stored by landed or
    assisted ship state, so every solid body uses the same character gravity.
24. Every visible ship receives current snapshot speed, thruster, Core energy,
    and optional nearby-body distance/radial data through renderer-owned
    `CockpitInstruments`; leaving the cockpit does not freeze or hide these
    values. Runtime field-maps rather than derives flight telemetry or simulates
    monitor values. Contextual messages remain in the window title.
25. Applicable cockpit, door, fragment and deposit prompts carry current absolute
    world anchors. Existing eligibility/obstruction gates select them; renderer
    projection/scene depth determines presentation visibility. Carried-object
    drop guidance follows the carried pose. Global flight/tool guidance and
    three-second blocked feedback remain screen-space. Precision tour hides
    gameplay prompts.
26. E2E scenario setup requires an explicit launch flag and occurs before the
    native event loop. It changes only initial controller state and update-clock
    policy. A ready line follows successful renderer initialization; ordinary
    launches retain the default state and monotonic clock.
27. The versioned JSON automation channel starts only after E2E renderer readiness.
    Its reader never touches game state; commands run on the native event thread.
    Only explicit fixed steps advance E2E simulation, and gameplay actions retain
    production interaction and controller gates. Ordinary launches do not read stdin.

- Handheld extraction requires gameplay view, Surface location, an equipped tool,
  held mining input, and a valid unobscured world target. It never uses E.
- Inspection and presentation apply the same world-owned session mass deltas.
- Focus loss, cursor release, view switch, and stowing clear held mining input.
- Fragment presentation/inspection read world-owned physical entities; neither
  can mutate mass or allocate output. Stowing does not remove fragments.

- Authored handheld presentation is visible only in gameplay on the surface
  while equipped; active feedback requires held input and a valid target. The
  separate screen-space reticle remains available when stowed.

- Gameplay aiming uses a screen-space centered dot when the tool is stowed and
  `+` when equipped, selected from the actual equip state every redraw. Camera
  pitch/yaw, location and drawable resize do not shift it. Precision tour hides
  it. No world-space aim cuboid is emitted; interaction/mining rays are unchanged.

- Loose-fragment simulation preserves session iteration order, excludes the carried
  ID and objects at least 125 m from the player, and preserves IDs/mass/material during pose writeback; physics updates the normalized orientation. Ship anchors/velocities use ship-local metres;
  surface objects use absolute metres; orientation/angular motion follows the same frame. Physical response is owned by
  `salimon-physics`; runtime supplies geometry and frame conversions only.

- F initial presses grab an aimed reachable world object, or release the carried
  object. A miss never starts mining; repeated or synthetic native presses cannot
  repeat the action. E never changes carrying.
- Only left mouse controls held mining. F press/repeat/release never activates
  the tool or sustains extraction after mouse release. Successful pickup cancels
  mining. Focus loss, cursor release, view switch and stow clear input and F latch.
- Automation `grab_drop` and legacy `pickup`/`drop` aliases use the production F
  route. Mining uses the shared left-mouse route; the old `mine` key is rejected.

- Equipment toolbar state has exactly five slots, initially mining tool then four
  empty slots, and zero or one selected slot (initially none). Empty selection
  targets are valid; repeated selection of the same slot never toggles it off.
- Gameplay 1–5 selects slots only on real initial native presses with cursor
  captured; releases, repeats and focus replay do not select. Precision-tour
  1–6 inspects bodies without changing equipment state.
- Successful physical pickup clears toolbar selection immediately; a failed pickup
  preserves selection and mining input. World-session carrying blocks all slot
  selection; dropping never restores selection. Explicit selection after drop
  never resumes a previous mining hold.
- Toolbar loadout/selection maps directly to renderer DTOs in gameplay and is
  hidden in precision tour. Toolbar presentation cannot change equip behavior;
  slot 1 is the sole mining equip authority; empty/absent selection stows it.
- Mining owns no independent equipped boolean. Targets, extraction, held tool,
  reticle and prompts all consume the toolbar decision. Switching slots cancels
  held input immediately; pickup cancels mouse mining without releasing the
  consumed F latch. M and the legacy equip automation key are unavailable.

- The carrying baseline/evidence route protects selection and mining together:
  all four empty slots must prevent left-mouse extraction at a reachable deposit, while
  explicit slot 1 selection restores the held tool and extraction eligibility.
  Evidence checkpoints observe equipped, empty-selected, carrying-deselected
  and post-drop-deselected states without changing gameplay actions.

- Resource gameplay dimensions use the shared world `resource_size` policy;
  enlarged bounds never substitute for density-derived solid material volume.
  See [resource size contract](../world/resource-contracts.md#gameplay-size-policy-144).

- Mining emission starts on an exposed authored deposit facet, reserving full
  growth support plus 5 mm and checking ground/player/existing pieces. There
  are no rows/grids. Only successful new-identity allocation installs an impulse;
  existing/growing/streamed objects are never implicitly re-ejected.

- Benchmark populations are opt-in initial conditions from production extraction;
  subsequent updates, carrying and support queries retain production behavior.
  Benchmark observations never change IDs, mass, poses or collision ordering.
- Native wall-clock benchmark results and fixed-step correctness/CPU costs remain
  distinct. Raw benchmark frame intervals preserve spikes; missing GPU/sleep data
  is null/explicit rather than synthesized. Requested settled state must be checked
  against observed moving-object counts.

- Fragment activation follows stable IDs at the portable physics boundary.
  Sleeping objects stay in session presentation/picking/collision; omission wakes
  dependent contacts and streaming return rebuilds support. Ship-local rigid
  movement preserves sleep and f64 world writeback. See the [activation adapter](architecture.md#fragment-activation-adapter-156).
