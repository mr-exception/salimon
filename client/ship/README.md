# Ship

`salimon-ship` owns portable ship state. It provides the landed-on-Earth
starting pose, cockpit-control authority, persistent inertial motion,
steering/thruster controls, continuous solid-body contact,
timed assisted landing/takeoff, and the
landed/open-space door rule. Opening or closing the exit uses a typed action.
Landed ships allow access; flying ships allow it only when the existing nearby
sensor reports no celestial body within the inclusive 3,000,000 m surface
threshold. Nearby flight and assisted sequences retain the `Door locked while
in flight` message. No pressure or survival simulation is implemented.

Leaving the cockpit changes only control authority. A flying ship keeps its pose,
orientation, selected thruster percentage, and inertial motion. Steering
ramps on/off over 0.12 seconds and settles at 5 degrees/second; this smoothing is
presentation feel, not inertia. Throttle selection is signed −100..+100%; funded
Core propulsion energy gives impulse at 1 N·s/J, divided by cached total mass
to change persistent world-space velocity. Zero throttle coasts. Up/Down
presses change one point and repeat every 100 ms held, independently of OS repeats.
The runtime supplies typed held edges; only powered cockpit free flight accepts
changes. Exit clears holds while preserving selection; landing/collision resets it. `L` starts an uncancellable automatic landing at the current approach
normal inside a body's `1.15R` volume, or an automatic takeoff while landed.
Both sequences continue without cockpit authority; an open door blocks takeoff
with `Close door before takeoff`.

Landing takes 8 seconds: 2 seconds to smoothly align the hull while holding its
position, 4 seconds to approach a low hover, and 2 seconds to touch down. The
final descent covers at most 15 m (a quarter of the available height for a close
approach). Takeoff lifts 10 m in 2 seconds, then rises another 40 m in 4 seconds using
smoothstep segments. It ends 50 m above the captured landed origin at rest,
with throttle zero and immediate powered manual control. The stages peak at
7.5 and 15 m/s respectively, each with acceleration magnitude at most 15 m/s²; position and velocity are
continuous at the join and handoff. This local clearance is independent of the
planetary landing-eligibility shell. There is no free-flight planetary gravity,
so zero-throttle flight remains stationary until thrust is selected. Alignment follows the shortest quaternion
arc. Timing uses supplied elapsed time, including frames longer than 100 ms;
no orientation or position change occurs on activation.

Each snapshot carries authoritative Core power/whole-joule telemetry and
the nearest catalog body within an inclusive 3,000,000 m surface distance.
The configurable starter Core stores 45 GJ in a 60 GJ capacity. Forward/reverse
thrust consumes up to 100 MW/50 MW plus independent gravity/cockpit idle loads.
Power loss stops thrust, cockpit controls/display and artificial gravity while
preserving throttle and velocity; charging resumes selected throttle. Fractional
joules remain internal. Physical conversion/intake is implemented; persistence
remains deferred. See [Core power](architecture.md#authoritative-core-power-188). Nearby telemetry includes
the body's name, surface distance, and signed radial speed: negative approaches,
positive recedes, and zero is stationary.

The current scout uses a conservative 16 m collision radius for its
18.33 × 4.00 × 20.00 m exterior. The landed pose places the authored runtime
feet on the sampled surface with 2 m non-gear hull clearance and aligns local
`+Y` with the starting surface normal.
The custom source and runtime model remain under
[`client/assets/ship`](../assets/ship/README.md).

See [README.ai.md](README.ai.md), [architecture.md](architecture.md), and
[invariants.md](invariants.md) before extending flight behavior.

The starter assistance gate is total planet-relative speed ≤20 m/s, regardless
of heading, plus cockpit authority and sufficient reserved energy. A settled
crash remains GroundedAfterCrash until powered L recovery completes; it cannot
take off directly. See [safe recovery](architecture.md#safe-assisted-landing-and-recovery-190)
and [navigation](architecture.md#vector-navigation-and-braking-193).
Future propulsion/landing upgrades and additional module power loads are OUT OF
SCOPE; configuration APIs do not implement a progression system.

B explicitly toggles engines while powered/seated outside an active assist. Engine
off coasts with idle loads, preserves throttle and survives save/reload/blackout.
Default dry acceleration is 1000/500 m/s² forward/reverse at 100/50 MW; landing
still requires <=20 m/s. See [math and assist reconciliation](architecture.md#funded-core-impulse-and-engine-switch-493).
