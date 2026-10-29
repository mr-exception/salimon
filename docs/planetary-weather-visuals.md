# Coherent weather presentation (#616)

`world::environment::weather_visuals::WeatherFrame` samples the authoritative
PE04 climate at one explicit shared TT epoch. A fixed 64×32 body-fixed spherical
field contains condensed cloud fraction, storm, actual precipitation flux and
explicitly admitted dust extinction, plus the same samples' tangent wind and
temperature. The exact local `WeatherSample` remains available for lighting and
future audio (#627). Climate/terrain definitions, resource availability and
canonical weather do not change with render quality, camera, LOD or reload.
Vacuum produces no airborne frame. Dust requires an explicit caller admission
that particulate material exists; storms alone never authorize dust everywhere.

`runtime/src/app/weather.rs` maps this field and an explicit world-to-body rigid
basis into `WeatherScene`; cloud-base/thickness are scalable presentation heights
(0.5/0.25 scale height), not new atmospheric physics. The existing local light
color receives overhead cloud/storm attenuation from the same local sample.
This is approximate overhead attenuation, not ray-certified terrain shadow maps.
Water/CO2/methane have separate approximate visual freezing thresholds; no liquid
accumulation, surface water, fluid simulation or new damage/hazard is introduced.
Current playable Vesper is explicitly vacuum and uploads no weather field.
Other climates are demonstrated by declared fixtures; full pack/body and rotating
frame adoption remains #427. The caller supplies the actual epoch and basis.

`renderer/src/weather.rs` / `weather.wgsl` use two fixed RGBA32Float 64×32 textures
(64 KiB total), a 224-byte uniform and one optical-volume draw. Manual bilinear
loads avoid optional float-filtering features; longitude wraps and latitude
clamps consistently. Body-fixed sampling preserves regional patterns from orbit
through descent into the cloud layer. Cloud condensation and dust are distinct
optical components; precipitation uses original flux, local temperature, wind and
shared-clock world-space streaks. Only cosmetic sub-grid wisps sway periodically
with wind; canonical macro-weather still comes entirely from PE04. Periodic phase
uses sine/integer-cycle harmonics to avoid a visual discontinuity at day wrap.

8/16/24 integration samples are presentation quality options over the same fixed
field. They do not change physical weather, resources, wind, IDs or clock. Scene
reverse-Z depth and spherical ground stop the volume at foreground surfaces.
The pass composites after atmosphere and opaque geometry, before overlays. Star
color, daylight and storm fraction drive cloud lighting; precipitation never
appears without supplied flux. No cloud or dust draw is admitted without a
physical atmosphere. The field is explicitly finite/bounded and axes are checked.

## Source and verification

World sampling/reconstruction/vacuum tests live beside `weather_visuals.rs`;
the runtime adapter preserves the full sampled frame and per-chemistry threshold.
The production opt-in `SALIMON_WEATHER_DIR=... cargo test -p salimon-renderer
--locked weather_production_reference -- --ignored --nocapture` exports cloud,
exact reload, low-quality, dust, rain, orbit and clear controls with actual Vulkan
shaders. It asserts exact reconstruction pixels, nonzero cloud/rain, dust color
and zero clear effect, plus rejection of missing atmosphere/bad grid/bad basis.
Run affected weather tests, workspace build/Clippy/fmt, tooling and docs checks.
The native `scenarios/evidence/planetary-vacuum.json` retains the actual airless
start, with its published basin walls preserved. Captures remain candidates for
human review #561, physical platform #557 and actual hardware/M1 budgets #563.

The model is bounded optical single scattering with stylized cloud microstructure
and precipitation streaks, not CFD, a physical droplet simulation, multiple
scattering, volumetric shadows, accumulated water or acoustic approval. Uniform
and grid dimensions are presentation contracts; canonical source/scene identity
is independent of them. Retain original timings as software-renderer diagnostics,
never as hardware FPS or M1/Metal qualification.

For cross-boundary evidence, set `SALIMON_WEATHER_FRAME_FILE` and run the opt-in
runtime `weather_integrated_frame_export` test first. It exports the actual world
sampler/runtime mapper field with its definition/TT epoch. Pass that same file to
the renderer evidence test; it selects the supplied maximum-cloud region for its
camera rather than forcing cloud into a dry region. The exported fixture uses
an explicitly fictional atmospheric climate on a flat engineering terrain, not
a claim of life/atmosphere on the real Moon. Source samples remain unchanged.
