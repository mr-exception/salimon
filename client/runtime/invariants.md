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
19. Gameplay interaction zones match the Task 10 cockpit and exit-door markers;
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
25. Normal gameplay view presents applicable cockpit/interaction messages in a
    readable bottom-centered action bar. State-derived prompts disappear when
    their source becomes inapplicable; immediate blocked-door feedback expires
    after three seconds. Precision-tour view never displays the gameplay bar.
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

- Gameplay aiming uses a screen-space centered dot when the tool is stowed and
  `+` when equipped, selected from the actual equip state every redraw. Camera
  pitch/yaw, location and drawable resize do not shift it. Precision tour hides
  it. No world-space aim cuboid is emitted; interaction/mining rays are unchanged.
