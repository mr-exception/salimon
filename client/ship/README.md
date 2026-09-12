# Ship

`salimon-ship` owns portable Phase 0 ship state. It provides the landed-on-Earth
starting pose, cockpit-control authority, persistent direct-speed motion,
Task 12 steering/thruster controls, solid-body boundary correction, Task 13
assisted landing/takeoff, and the
landed-only door rule. Opening or closing the exit uses a typed action;
while flying it remains closed and emits the cockpit message `Door locked while
in flight`.

Leaving the cockpit changes only control authority. A flying ship keeps its pose,
orientation, selected thruster percentage, and direct-speed motion. Steering
ramps on/off over 0.12 seconds and settles at 5 degrees/second; this smoothing is
presentation feel, not inertia. Thruster changes are clamped to 0–100% and map
directly to speed. The runtime supplies typed held axes and discrete percentage
steps. `L` starts an uncancellable automatic landing at the current approach
normal inside a body's `1.15R` volume, or an automatic takeoff while landed.
Both sequences continue without cockpit authority; an open door blocks takeoff
with `Close door before takeoff`.

The wider ship uses a conservative 15 m collision radius for its
20.30 × 4.00 × 20.00 m exterior. The Task 10 playtest revision places the runtime
mesh's `-0.108 m` lowest local-Y point on
Earth's nominal surface and preserves identity orientation so local `+Y` follows
the starting surface normal.
The custom source and runtime model remain under
[`client/assets/ship`](../assets/ship/README.md).

See [README.ai.md](README.ai.md), [architecture.md](architecture.md), and
[invariants.md](invariants.md) before extending flight behavior.
