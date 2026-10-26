# Physical room and fixture lights (#580)

Ship `lighting.rs` owns durable room identity, Work/Relax/Night/Custom preferences,
master enable, general brightness/warmth, five optional fixture overrides and
installed fixture operability. Runtime maps the actual Core bus and shutter
mechanism into renderer-only effective output. Preferences do not generate power,
change stored joules, turn on broken lamps or reset on blackout. Fixture wattage
remains unapproved; no fictional energy consumer or backup light is introduced.
Damage-system updates to lamp operability belong to #549; this PR admits and
persists existing inoperable fixture source without implementing combat damage.

Aim at the bunk/seat/table, galley, hygiene or Core/Fabricator physical control;
use F4 to cycle room presets, F5 room master, F6 the local fixture, F7 its
brightness, F8 room warmth and F9 general room brightness. Cargo-door controls
operate the independent cargo room. Custom controls retain other overrides;
selecting a preset resets overrides. Input uses ordinary reach/sight/menu gates.
Future rooms retain independent settings by ID, even if their divider is removed;
#545 owns actually constructing and registering those rooms.

Five ceiling-mounted housings/diffusers and bounded local falloff supply task
light. Original hull geometry and independent fixture instances receive the same
room/effective-light DTO. Machine emissions/screens retain actual machine/bus
semantics and never follow room master. Existing ordinary metal/composite/fabric
colors and faceted geometry remain; no new PBR or shadow/GI simulation is claimed.

Window light uses actual Sun direction and conservative planet-sphere occlusion,
then actual manual shutter travel. It adds a localized cool contribution near the
real observation window even in blackout. This is bounded stylized illumination,
not physically accurate radiometry, atmospheric scattering or ray-traced transport.
A small dark reflectance/readability floor is not a powered emergency lamp.
The real glazing and moving shutter continue to show/hide the actual exterior.

Renderer ownership: `CabinLighting` has two room fills and six bounded local
contributions (five electrical, one exterior), adding 224 uniform bytes to the
ship path. Fixture instances add one vec4 (48 → 64 bytes) for effective RGB;
geometry resources and material/instance batching remain shared. Ten small
housing/diffuser cuboids use the existing instance pass. Native state captures
retain actual resource/CPU/GPU observations; software timings do not qualify FPS.

Schema 16 adds validated lighting source; historical saves migrate default Work
preferences with operable fixtures. Historical envelopes carrying custom light
state reject, missing installed rooms reject, and save/restore does not replay
commands, emit sounds, advance time or change ship energy/mass.
