# Regional climate and weather (#615)

World `environment::climate` owns version-1 immutable climate sidecars and pure
body-fixed queries. `ClimateDefinition` binds the complete geological definition
(and therefore the exact terrain) to an independent seed and validated pressure,
chemistry, condensable availability, scale height, rotation/year periods, tilt,
stellar flux, albedo and greenhouse authoring inputs. No palette or geological
history implies an atmosphere. Callers must publish explicit inputs; no observed
body receives fictional weather automatically and Vesper's publication is frozen.

`SLMCLM01` encodes the version, independent seed, inputs and upstream geology.
SHA-256 identifies the whole sidecar. Restore decodes against the known physical
body and rejects unsupported versions, chemistry, nonfinite/out-of-range values,
wrong terrain/body, truncation and trailing bytes. `compatible_with` requires the
exact definition and geology; changing either requires explicit migration.
Existing U11 pack bytes and session schema remain unchanged. Sidecars are
publication data, not player knowledge or a mutable session cache.

Queries take a body-fixed unit direction and the existing uniform **TT Epoch**.
They evaluate directly relative to J2000 TT noon at 1:1 time, including past epochs;
no wall-clock sampling, random generator, integration timestep, loaded region,
LOD or device quality enters the result. Online, offline and future authoritative
hosts must supply the same verified epoch and sidecar. UTC/TDB are rejected rather
than silently converted. Offline trajectory/clock verification remains U31–U33;
this evaluator does not pretend that those unimplemented hosts already exist.

The bounded stylized model combines latitude/seasonal illumination,
Stefan–Boltzmann equilibrium plus declared greenhouse, a 6.5 K/km elevation lapse,
scale-height pressure, a continuous global spherical storm front and tangential
circulation. Available condensables gate cloud/precipitation; geological
weathering modulates precipitation. Chemistry and condensable identity are retained.
Condensation temperature cutoffs are coarse authored approximations, not phase
diagrams, weather forecasts or scientific reconstruction. This model grants no
liquids, standing water, hazards, damage or survival effects. Pressure/temperature
and appearance approximations are not substitutes for U18 density/drag physics.

`WeatherSample` exposes pressure/temperature, clouds, storm strength, physical
condensable flux metadata, atmospheric extinction and body-fixed tangent wind.
Vacuum has exactly zero pressure, wind, clouds, storm, precipitation and extinction.
Zero extinction means unlimited atmospheric visibility. The body-local wind must
be rotated into inertial gas axes by the composition owner before supplying the
existing `salimon_ship::atmosphere::Atmosphere::sample` wind input; this change
does not modify drag, engine force or energy rules. Renderers/audio may approximate
presentation from these values, but must not generate different authoritative
weather. Clouds and precipitation rendering remain #616; sound is #627.

## Validation

`client/world/src/environment/climate/tests.rs` exercises actual evaluator/codec
roundtrips, independent concurrent hosts, partition-independent offline queries,
region/cube seams and poles, evolving fronts, vacuum exclusion, elevation,
malformed publication, version migration and a frozen numerical energy vector.
`client/ship/tests/regional_weather.rs` additionally exercises the real U18 gas
and drag operators using climate wind with an explicit matching frame.
Run `cargo test -p salimon-world --locked environment::climate`, then the workspace
build/tests/Clippy/format and documentation checks in [validation](validation.md).
This portable metadata implementation changes no rendered route. Human appearance
and hardware qualification remain separately unverified under #561/#563.
