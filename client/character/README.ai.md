# Character AI Maintenance

Read `architecture.md` and `invariants.md` before edits. Keep this crate portable:
no `winit`, `wgpu`, renderer, diagnostics, or ship-crate dependency. Native input
translation and domain composition belong to `client/runtime`.

Important regression areas are diagonal-speed normalization, edge-triggered
jumping, closed/flying doorway containment, the exact 250 ms gravity blend,
instant cockpit transitions, horizontal mouse-delta direction in both walking
and cockpit views, the default forward cockpit-window sightline, and
constant-radius full-sphere movement. Add a focused unit test whenever one of
these rules changes.
