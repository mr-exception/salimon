# Ship

`salimon-ship` owns portable Phase 0 ship state. Task 8 establishes the landed-on-
Earth starting pose, cockpit-control authority, persistent direct-speed motion,
and the landed-only door rule. Opening or closing the exit uses a typed action;
while flying it remains closed and emits the cockpit message `Door locked while
in flight`.

Leaving the cockpit changes only control authority. A flying ship keeps its pose,
orientation, selected thruster percentage, and direct-speed motion. Task 9 owns
keyboard steering and thruster adjustment; Task 10 owns assisted landing/takeoff.
The custom source and runtime model remain under
[`client/assets/ship`](../assets/ship/README.md).

See [README.ai.md](README.ai.md), [architecture.md](architecture.md), and
[invariants.md](invariants.md) before extending flight behavior.
