# Active browser launch and landing contract

The default built `/` starts outdoors at the authored Vesper patch. Browser
issues #838–#844 deliver the lifecycle, door, takeoff, flight, deterministic sites,
landing planner and cockpit controls. This React/TypeScript implementation is
separate from historical Rust/native #72/#190/#418; those are design/history,
not executable browser code or current acceptance evidence.

## Playable loop and limits

Board through the aft doorway, approach the doorway from inside and use **E** to
close it, then walk around the cabin crate to **Use cockpit**. **Take off** requires
the live owner seat, power, all occupants aboard, a supported grounded site and no
conflicting work/maneuver. An open door is never closed automatically. The
uncancellable 360-tick/six-second ascent retains continuous physical pose and
hands off to flying with throttle zero. Arrow keys steer; signed throttle applies
thrust; Space zeros it. WASD/left-drag rotate only the independent observation
camera. Set direction captures a one-time ship heading, not a navigation autopilot.

**Land** selects a currently scanned solid body (selected contact, otherwise the
nearest eligible solid body); Look at does not select a landing target. Admission
requires altitude <=0.15 body radii, full surface-relative speed <=20 m/s including
tangential motion, a valid clear footprint and cockpit authority. Heading is not
required. The bounded dynamic descent is uncancellable; manual input, leaving the
cockpit or a power-fixture interruption cannot override it. >=100 m/s relative
impact destroys the whole ship and cannot create a traversable landing.

The world owns one 60 Hz clock in both views. Blur releases held input; hidden
page time and explicit pause stop active simulation. Leaving the cockpit during
flight exposes only the ship-local 2D cabin. The sealed door blocks EVA. Verified
touchdown alone restores the matching planet frame, returns the pilot standing
inside, and permits opening the door and walking outside.

Supported sample bodies are **Vesper, Vesper Moon and Caligo**. Other physical
regions materialize deterministic sparse empty patches; stars and asteroids are
unsupported. Returning within the authored Vesper region restores its original
source IDs and depletion, never a fresh resource copy. The one global cargo/source
ledger and schema-v11 visited-site state survive launch, revisit and reload;
valid legacy saves migrate, inconsistent frames/plans fail safely. The observation
camera is session-local, not durable authoritative state.

Energy is explicitly unlimited; cockpit power is a test fixture, not a Core.
Canonical terrain/rendering, gravity, ephemerides, atmosphere, EVA, Core funding,
multiplayer and server reconnect remain future work. See [planet generation and
its implementation boundary](canonical-planet-generation.md), [gravity](universe-gravity.md),
[Core](core-upgrades.md), [backend](backend-architecture.md) and their linked
implementation plans. These samples are not canonical planet qualification. Browser planetary follow-ups
are [#847 canonical surface adapter](https://github.com/mr-exception/salimon/issues/847),
[#850 illustrated terrain](https://github.com/mr-exception/salimon/issues/850),
[#853 regional persistence](https://github.com/mr-exception/salimon/issues/853) and
[#854 exploration QA](https://github.com/mr-exception/salimon/issues/854). Those
issues do not implement gravity, Core or multiplayer; their future contracts
above still require separately scoped browser work.

## Regression ownership and evidence

`tests/flight-loop.spec.ts` starts with empty browser storage and keyboard/mouse
input on built `/`: gather, board, seal, ascend, steer, interrupt, reload airborne,
land at the original coordinates, disembark and observe the depleted source.
No save/pose injection joins those phases. `tests/landing.spec.ts` separately
seeds a synthetic moon approach (interplanetary travel is not proved), then uses
real Land, walk, reload, relaunch and revisit controls on Vesper and the moon.
`tests/surface-sites.spec.ts` covers Caligo and site-frame fixtures.

| Scenario | Asserted by |
| --- | --- |
| Open door, outside/invalid pilot, power, duplicate commands, conflicting operation | `scoutDoor.test.ts`, `scoutFlight.test.ts`, `boarding-door.spec.ts`, `flight.spec.ts` |
| Radial/tangential >20 m/s, range, unsurveyed target, obstruction/invalid footprint | `landing.test.ts`, `surfaceSite.test.ts`, `landing.spec.ts` |
| Uncancellable assists, continuity, one clock, camera/input independence | `takeoff.test.ts`, `landing.test.ts`, `flight.spec.ts`, `flight-loop.spec.ts` |
| Destructive collision, stale exterior, cockpit exit, pause/hidden/blur | `landing.spec.ts`, `flight.spec.ts`, `flight-loop.spec.ts` |
| Site restore/migration, invalid save, cargo/source conservation and unique entities | `scoutFlight.test.ts`, `surfaceSite.test.ts`, `surface-sites.spec.ts`, `flight-loop.spec.ts` |

Run `npm test`, `npm run build`, then `npm run test:browser -- --project=cockpit`
from `client/`. Existing navigation/scanner/HUD reference suites remain in the
full browser workflow. `npm run test:browser:negative` builds disposable broken
launch/touchdown transitions and requires the named loop assertion to fail;
a build failure or timeout does not count. `SALIMON_MUTATION` selects one
mutation for focused diagnosis. Existing visual/UI negative commands remain.

The fixed sample seed is `sampleUniverse / surface:v1`, epoch
`2026-11-04T00:00:00Z`, viewport 1280×800 and DPR 1 for cockpit journeys.
Original/repeat screenshots, changed-control negative captures, JSON state and
loop action traces live in ignored `client/test-results/`; exact results and
revision bindings belong in the PR. Existing approved references are not silently
updated. Software Chromium/SwiftShader captures establish that context only,
not native hardware/GPU certification. CI artifacts retain 14 days; the owner
reports CI results in the active chat. Local passes alone do not imply final QA
acceptance or approval of new screenshot baselines.
