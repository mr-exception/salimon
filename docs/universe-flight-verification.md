# Flight acceptance matrix (#437)

Run from the repository root with a built executable, X11 display, Mesa Vulkan,
xdotool and the [graphical prerequisites](validation.md#native-graphical-gates):

```sh
python3 scripts/verify_universe_flight.py --binary target/debug/salimon-client \
  --output artifacts/flight-acceptance/fresh-run
```

Use a new output directory on every run. The command runs production ship and
session Rust contracts, all nine `universe-flight` GPU evidence routes, then a
normal launch with OS-injected F2 press/release. Each subprocess has a deadline;
failed commands preserve their logs and a failed aggregate. Complete route and
step inventories, original PNG dimensions/GPU source, raw telemetry and hashes
are checked before success. This validates evidence integrity, not subjective
appearance. No existing baseline is promoted and no thresholds are loosened.

| Criterion | Production owner and automated coverage |
| --- | --- |
| Takeoff, thrust, Core funding and mass | Ship surface/trajectory/propulsion; runtime canonical funded takeoff/landing and `flight_regression` session contracts; `universe-flight`, `inertial-flight`, `cargo-flight` routes |
| Orbit and moving-body gravity assist | Ship `trajectory/tests.rs`: funded tangential circularization with doubled-mass negative control and unpowered continuation, analytic orbit convergence, moving-body relative-energy conservation and independent three-body RK4 reference; exact source/provider failure tests |
| Drag and energy | Ship `atmosphere/tests.rs`: terminal-speed refinement, orbital energy loss, online/offline operator equivalence and atomic provider failures |
| Landing, first contact and crash ordering | Ship `surface/tests.rs`, `contact/tests.rs`; swept moving spheres/triangles, bounded exhaustion, elapsed-work funding and response energy; `landing-speed-cutoff`, `crash-recovery` routes |
| Cockpit prediction and resets | Ship `prediction.rs`/`preview.rs`; runtime navigation cache/save and control admission tests; `universe-navigation`, `moving-eva` routes |
| Power failure, refuel and restart | Runtime `propulsion_tests`, `flight_regression` and canonical restore tests; `core-restart`, `cargo-blackout` routes |
| Window and input integration | `packaged_smoke.py` normal launch from an unrelated directory, actual presented frame, OS F2 round trip and desktop captures |

## Scope of the evidence

The two canonical routes use published Vesper terrain. The seven realistic-flight
regression routes retain declared historical initial conditions and exercise
production controllers; they do not establish a canonical interplanetary journey.
Protocol route actions bypass physical keyboard delivery. The separate normal-launch
smoke establishes X11/window key integration for F2, not hardware flight controls.

Orbital insertion, gravity assist and ephemeris contact tests execute the real-scale
production APIs with explicit test ephemerides. The playable local controller
remains distinct from those inertial APIs. An orbit test is not proof of playable
interplanetary flight, catalog-wide streaming, or arbitrary closed-game progression
(#432–#434). The [navigation contract](universe-navigation.md) documents that boundary.

Software-Vulkan contexts/raw timings are retained per capture; missing GPU counters
remain null, never zero. Cloud timings do not establish physical 60 FPS or M1/Metal.
Physical device/platform behavior remains #557, handling/appearance #561, and
hardware budgets #563. No personal signoff is needed for routine cloud acceptance.

Owning contracts: [ship architecture](../client/ship/architecture.md),
[ship invariants](../client/ship/invariants.md),
[runtime architecture](../client/runtime/architecture.md) and
[runtime invariants](../client/runtime/invariants.md).
