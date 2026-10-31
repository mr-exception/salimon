# Bounded navigation predictions (#421)

Ship owns forecasts; runtime caches observations and maps them into renderer-only instruments. Neither renderer, prediction nor cache issues commands, changes an assist, spends authoritative energy or grants an autopilot. Existing range, radial/total relative speed, velocity direction, heading angle, acceleration and braking feasibility remain live observations.

## Real-scale API

`client/ship/src/prediction.rs::coasting` clones the production `Trajectory` and its gravity selector. Positions remain `GalacticPosition` split cells; target position/velocity and gravity candidates are explicit, pure epoch-aware providers in the integrator's TT/TDB scale. No global galactic f64 coordinate is constructed. Caller data must already be available; no disk or network work belongs in a provider.

Requests bound work to 1–128 intervals, even nanoseconds per interval, at most 60 seconds per interval and one hour total (the source trajectory may impose a smaller step). Nonfinite radius/heading/target velocity, bad heading normalization, invalid intervals and source/target failures return errors without altering source state. The result includes initial target-sphere range, inertial ship-minus-target relative velocity, signed radial velocity (negative approaching), heading-to-target angle, first-interval average acceleration, closest **sampled** range/time, fine endpoint and optional first sampled entry window.

Both the requested and half-step integrations use the authoritative gravity/Verlet operation. Maximum matched-endpoint separation is a **convergence indicator**, never a certified error bound. First sampled sphere entry brackets sample times; it does not certify continuous first contact, docking or arrival. A pass between samples can produce no entry window. It is a coasting-gravity forecast: thrust, drag, terrain/contact and subsequent control changes invalidate that model. Source/ephemeris uncertainty is not estimated by step halving. No guaranteed arrival or long-distance straight-line ETA is fabricated.

Tests compare a moving target and analytic empty-space motion across cells, exact authoritative half-step propagation, gravity refinement, sampled crossing/miss behavior, malformed input and provider failures. The existing independent RK4/orbit/continuous-contact foundation tests retain their separate budgets.

## Playable cockpit preview

The canonical translating-frame controller is distinct from the generic split-inertial Trajectory API. `ship/src/preview.rs::navigation_preview_at` clones the **actual current controller and motion clock**, including velocity, selected terrain/body, held throttle/steering, power, accepted conversion and collision state. It advances 128 × 64 ms and a half-step reference, returning at most an **8.192 second** preview. Contact stops the preview and supplies an approximate observed contact-time window. Landed, assist/crash or out-of-sensor-range states have no free-flight prediction.

The target is the initially selected nearby body; minimum altitude is relative to its nominal sphere, while contact uses the actual selected production terrain/contact rules. No hull safety is inferred from nominal altitude. The preview assumes held inputs and current mass, known on-board conversion and ordinary configured consumers. It cannot anticipate future runtime auxiliary demand, refueling, new cargo, commands, unknown ephemerides or a different physical model. Canonical previews use `advance_celestial` at each interval, sharing live gravity and contact; a missing/invalid clock yields no preview. Predicted power loss and contact have explicit warnings. Coarse/half-step positional difference is shown as `STEP DIFF`; it does not include model uncertainty.

Runtime `session/navigation.rs` refreshes at most every 500 ms of admitted simulation time while unchanged controls remain held; commands, flight/terrain/engine/throttle/power/mass changes invalidate it immediately. Displayed `AGE` exposes the snapshot age; horizon/contact/closest times subtract that age to remain relative to now. Presentation reset/restore clears the disposable cache. Cache state is absent from saves and cannot affect tick admission. Renderer `PredictionInstruments` contains observations only and displays the approximate contact or minimum altitude/time, numerical difference, age and `HOLD INPUTS` / `NO AUTOPILOT` / power warning. Atlas updates retain existing visible-value caching. The scanner/chart/discovery/Earth-route state is untouched; this adds no galaxy-wide destination reveal.

## Verification and routing

- `cargo test -p salimon-ship --locked`: integrator references, forecast and contact/power nonmutation tests.
- `cargo test -p salimon-client --locked navigation_`: cache expiry, invalidation, restore/source byte preservation.
- `cargo test -p salimon-renderer --locked`: visible atlas changes, compact text and unchanged upload caching.
- `scenarios/evidence/universe-navigation.json`: declared Vesper approach initial pose/velocity; actual free-flight preview, restore, production contact and invalidation, with physical cockpit original software-Vulkan captures.
- All owning ship/runtime/renderer README.ai, architecture and invariant guides link here. [Report](https://github.com/mr-exception/salimon/blob/f1cd0b3e878fac487ce898547a0c8c35aa40519b/reports/issue-421/README.md) records exact results, failures and limitations. Hardware/subjective qualification remains #557/#561/#563.
