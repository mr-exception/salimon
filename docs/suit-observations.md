# Suit position estimates and field marks (#625)

F10 records the current estimated spot; F11 records a body-fixed sightline with
unknown range; F12 disconnects/reconnects suit navigation. Readout text lasts 12
simulation seconds and states WELL CONSTRAINED / ESTIMATED / POORLY CONSTRAINED /
UNAVAILABLE, numerical radius when known, and bearing-only range uncertainty.
These controls do not take an equipment slot, drop cargo, consume food or emit a
scanner pulse. Pause, terminal, engineering view, construction and modal UI deny
them. Optional mark notes are source data; detailed personal editing is #626.

## Measurement contract

This is a simulated beacon/inertial instrument, never a universal GPS fix. A
powered Scout emits from an exterior roof reference [0,4.5,0] ship-local metres.
A measurement requires a finite ray within the configured 500 m, with actual
terrain, authored hull/door, installed construction and organism clearance. An
occluded/unpowered beacon gives no new fix. The body-fixed beacon solution is
quantized on a 2 m grid with a conservative 5 m position envelope; that envelope
also represents the ship's navigation reference accuracy. Ground truth drives
simulated sensors, but saved/displayed knowledge contains their bounded estimate.

Between fixes, per-tick measured body-fixed displacement has a declared 0.2%
scale bias (bounded by the configured drift rate), and the radius grows by 0.02 m
per metre of travelled path plus 0.01 m per second. The full path is accumulated,
not net displacement; a loop still accumulates error. Fix acquisition is sampled
once per shared second or on a field command; inertial integration continues at
each completed tick. Fixed runtime body axes remain explicitly fixed; no orbital
phase or rotation is guessed. Inertial-frame consumers use the universe owner's
`fix_in_frame` with an evaluated `FrameTree` and exact TT epoch; its rotating-body
regression checks that body coordinates and stale-epoch rejection are preserved.

The captured configuration is `client/runtime/config/suit_navigation.json`.
Invalid/nonfinite values, negative drift, too-narrow fix envelopes and unsupported
frame versions reject. Navigation-off clears the current estimate to unavailable
and retains prior marks. Reconnection requires actual measurement availability;
no silent position recovery follows a reload or loss of signal. F11 declines
when the navigation reference is unavailable or disconnected, rather than saving
a true camera direction as a measured body-frame bearing. There is no
new suit-battery, GPS satellite, celestial fix or scanner mode in this task.

## Knowledge, identity and persistence

Universe `surface_observations.rs` owns validated configuration, error propagation,
body-fixed estimates and an ordered book of up to 1,024 observations. Runtime
`session/field_marks.rs` owns measurement inputs, gates, bindings and controls;
native keys and automation share the same dispatch. Notes are bounded to 256
Unicode characters without control characters. IDs monotonically allocate and
never derive from a landmark, render tile or scanner contact. A sightline stores
only origin estimate, normalized direction and angular error (0.02 rad); it never
invents target distance/coordinates or a verified scientific identification.

Runtime schema 27 requires a source book for canonical journeys. Schema 24–26
canonical saves without one migrate to an empty unavailable book; older fixture
reconstruction remains explicit and contains no book. Present legacy books are
validated normally. Binding requires exact body ID, definition SHA-256, version-1
measurement/frame model and consistent TT observation/measurement timestamps;
no mark may be ahead of the session clock. Reload validates in temporary owners
before publishing, rebuilds transient sensor sampling from the physical pose and
preserves the exact knowledge bytes. Different definitions require an explicit
migration, not implicit reprojection. No catalog POI is queried or chart history
mutated. Scanner five-second cadence and three-hour freshness remain independent.

## Verification and limits

Run universe `surface_observations`, runtime `field_`, workspace build/tests,
all-target Clippy/fmt and documentation/base-impact gates. Production evidence:
`scenarios/evidence/suit-observations.json`. Source provenance, bounded drift,
unavailable/bearing cases, rotating-frame and save rejection tests establish the
instrument model. They do not establish real hardware sensor accuracy, subjective
readability approval (#561), physical function-key behavior (#557), M1/Metal
performance (#563) or a full planetary chart (#626).
