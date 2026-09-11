# Ship

`salimon-ship` owns portable Phase 0 ship state. Task 11 establishes the landed-on-
Earth starting pose, cockpit-control authority, persistent direct-speed motion,
and the landed-only door rule. Opening or closing the exit uses a typed action;
while flying it remains closed and emits the cockpit message `Door locked while
in flight`.

Leaving the cockpit changes only control authority. A flying ship keeps its pose,
orientation, selected thruster percentage, and direct-speed motion. Later tasks
own keyboard steering, thruster adjustment, and assisted landing/takeoff. The
Task 10 scale pass places the runtime mesh's `-0.20 m` lowest local-Y point on
Earth's nominal surface and preserves identity orientation so local `+Y` follows
the starting surface normal.
The custom source and runtime model remain under
[`client/assets/ship`](../assets/ship/README.md).

See [README.ai.md](README.ai.md), [architecture.md](architecture.md), and
[invariants.md](invariants.md) before extending flight behavior.
