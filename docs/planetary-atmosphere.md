# Planetary optical atmosphere (#614)

`world::environment::atmosphere::OpticalProfile` derives an optional optical
profile from the immutable, validated PE04 `ClimateInputs`. Vacuum returns none.
Pressure scales optical density relative to 101325 Pa; declared nitrogen, carbon
dioxide and hydrogen choose approximate refractivity multipliers 1, 2.4 and 0.24.
RGB reference Rayleigh coefficients are 5.8/13.5/33.1 × 10^-6 m^-1 (680/550/440 nm).
This is a scalable visual approximation, not measured composition spectroscopy,
CFD, multiple scattering or a scientific radiative-transfer certificate. It does
not change U18 gas density, drag, weather truth or canonical source definitions.

`runtime/src/app/atmosphere.rs` maps physical radius/center, optical profile and
caller-provided star direction/spectrum/eclipse visibility into the neutral
`AtmosphereScene`. The current playable canonical Vesper explicitly has zero
pressure and remains airless, with no atmosphere draw. The mapper accepts other
published climates without assuming universal Earth blue or inventing gas from
planet palettes. Full pack-to-playable-body adoption remains #427; the explicit
Earth-like optical test world is a declared fixture, not a new canonical world.

`renderer/src/atmosphere.rs` / `atmosphere.wgsl` use a finite exponential spherical
shell (12 scale heights). Sixteen view samples and six illuminator-path samples
integrate single Rayleigh scattering with wavelength-dependent transmittance,
planetary day/night shadow and the supplied partial/total eclipse fraction.
Sun color modulates scattering. CPU f64 camera-minus-planet subtraction precedes
GPU narrowing into planet-radius units. The 128-byte uniform, no new vertex data,
one optional fullscreen draw and fixed integration counts bound GPU work.

The pass runs after opaque/held/ship rendering and before overlays. It samples
actual scene reverse-Z depth to stop the optical path at foreground objects;
terrain, ship interiors and cockpit displays therefore do not receive a sky
paint-over. The spherical ground also clips distant paths, preserving the limb
from surface through descent and orbit without LOD-dependent atmosphere swaps.
The color pass blends integrated radiance with mean RGB extinction; colored
extinction of underlying surfaces is approximated by this mean. Fog textures,
arbitrary blue gradients, clouds and precipitation are not used by this pass.
Atmospheric scattering is not a change to direct point-light energetics. Weather
visual coupling remains #616 and sound #627; no hazards/hydrology are introduced.

## Verification

Run `cargo test --workspace --locked atmospher`, build/all-target Clippy/fmt and
docs checks. `SALIMON_ATMOSPHERE_DIR=... cargo test -p salimon-renderer --locked
atmosphere_production_reference -- --ignored --nocapture` exports production
Vulkan day, night, total eclipse, descent, orbit, vacuum and near-depth controls.
Tests assert wavelength ordering, exact vacuum/night/eclipse absence in a black
control, upper-sky attenuation with altitude, opaque occlusion and malformed DTO
rejection. Full-image averages across altitude are not monotonic because the
visible ground/horizon solid angle changes; the upper-sky test compares a fixed
clear-sky region. Captures are review candidates, not automatically approved
baselines. Actual M1/Metal costs and subjective appearance remain #563/#561.
