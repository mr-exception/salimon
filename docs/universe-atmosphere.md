# Planet-specific atmosphere and drag (#419)

`salimon_ship::atmosphere` supplies the real-scale continuum drag model used by
`Trajectory::step_with_drag` and `Maneuver::activate_with_drag`. It consumes
explicit planet radius, density, scale height, model ceiling and rigid spin;
it never guesses an Earth atmosphere for a planet with unknown properties.
`None` from the trajectory air provider means declared vacuum, while unavailable
or invalid data must return an error. Source acquisition/physical-field selection
remains with universe source adapters; this change does not publish scientific
planetary atmosphere data or infer missing measurements.

## Density and gas velocity

Density below 90% of the model ceiling is rho0 * exp(-altitude/H). A C1 smoothstep
taper brings it to exact zero at the ceiling. Heights must be finite and above
the physical radius; solid interior queries fail. Radius, height and ceiling
must be positive finite values; density is finite and nonnegative. This is an
explicit isothermal exponential continuum approximation. Weather, rarefied flow,
Mach-dependent coefficients, heating and damage are outside this model's validity.

Gas inertial velocity is body ephemeris velocity + omega cross ship offset +
provided inertial-axis wind. Body motion epochs must match the query. Wind and
spin are explicit inputs, not noise or player-dependent state. No far galactic
absolute f64 position is formed; body subtraction uses split coordinates first.

## Integration and energy

`Aerodynamics` validates nonnegative finite Cd and actual projected area. The
relative acceleration is -(rho * Cd * A / (2m)) * |u|u, using current ledger-derived
Propulsion mass. The instantaneous force calculation funds drag compensation in
surface-assist admission, including wind contact/friction checks. Insufficient
thrust or charge still rejects before reserving any Core energy.

Free-flight integration uses symmetric half drag operators around the existing
#417 gravity/funded-impulse step. At fixed density/gas velocity the exact operator
is u_new = u_old/(1 + k |u_old| dt). Unlike Euler drag, it cannot reverse relative
velocity or add gas-relative kinetic energy under a stiff step. Inertial kinetic
energy can increase when gas transfers momentum to a slower ship; that is not
free propulsion. Gravity and gas samples at both endpoints retain the caller's
explicit epoch and provider validity. Vacuum preserves the original trajectory
exactly. Provider/numerical failure publishes neither state nor gravity-selector
changes. Overflow fails with an actionable reduce-step/model error.

Online and offline consume this same operation with the same fixed steps.
Closed-game composition supplies coasting thrust and must stop engines under the
existing policy; this API does not authorize offline engines, jump, time skipping
or a second Core debit. Drag changes no mass or stored Core joules. Mechanical
energy dissipation is not a newly implemented heating/damage system. Shared
clock/event/save orchestration remains #432–#434.

Strang splitting is second order for smooth fields; it is not permission for
arbitrarily large atmospheric steps. A caller must refine dt for its physical
error budget (the integrator still enforces its configured <=60 s maximum).
The included near-constant-g descent test observes ~4x velocity error reduction
when halving 0.1 s to 0.05 s against a 0.001 s reference, and converges to the
independent terminal-speed formula. No numerical or model accuracy claim is made
outside the declared conditions.

## Source, validation and adoption

- `client/ship/src/atmosphere/mod.rs` owns validated atmosphere/aerodynamic values.
- `client/ship/src/trajectory.rs` owns atomic Strang composition.
- `client/ship/src/surface/mod.rs` owns Core-funded assist admission with passive
  gravity plus drag; its original vacuum entry point remains compatible.
- `client/ship/src/atmosphere/tests.rs` covers profiles, rotation/wind/epochs,
  closed-form drag, no reversal, vacuum equivalence, terminal convergence,
  orbit decay, identical offline/online replay, rollback and paid drag assists.
- Run `cargo test -p salimon-ship --locked atmosphere::`, workspace build/tests,
  all-target Clippy, fmt and documentation/base-impact checks.

This extends the production portable #417/#418 real-scale API. Existing
compressed gameplay, save schema, shaders and UI remain unchanged until the
already scoped canonical-world adoption (#424–#427). Real-scale contact is #420;
scientific full-pack acceptance is #524/#526. Physical platform, human presentation
and hardware qualification remain separately unverified under #557/#561/#563;
no new renderer output requires graphical captures for this domain-only PR.
