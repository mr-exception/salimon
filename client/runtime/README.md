# Runtime

The `salimon-client` binary is the native client composition entry point. It owns
the `winit` application lifecycle, native window, redraw scheduling, resize
routing, recoverable surface-loss handling, monotonic frame/update clocks, and
client composition. It drives `salimon-character`, `salimon-ship`,
`salimon-world`, `salimon-renderer`, and `salimon-diagnostics` but does not own their domain, GPU, or aggregation
implementation.

Run from the repository root with `cargo run --locked -p salimon-client`.
The process requests a physical 1920×1080 initial drawable for reference
performance measurements, opens a resizable native window, and runs until the
window is closed.

## Deterministic E2E launch

For declarative end-to-end tests, see [the scenario runner](../../scripts/README.md).

Pass `--e2e` after `--` to opt into reproducible initial conditions. For example:

```sh
cargo run --locked -p salimon-client -- --e2e --scenario orbit-moon --seed 42 --step-ms 16
```

Scenarios: `landed-earth` (default, standing in the ship), `cockpit-earth`
(seated in the landed ship), `orbit-earth`, and `orbit-moon` (seated in a flying
ship, 1 km above the nominal surface). The world catalog is immutable; the seed
selects a reproducible tangent offset of at most 100 m for orbit scenarios.
`resource-approach` starts seated 1 km directly above the Earth reference landing
site without the orbit fixture's tangent offset, allowing a full landing, mining,
and cabin-delivery route. Its seed still selects the normal generated resource world.
`--seed` defaults to 0. `--step-ms` defaults to 16 and accepts 1–100 milliseconds.
The command channel advances simulation only when asked to step; redraws do not
advance it. Ship flight, doors, character motion, and collision remain on their
normal controller paths. The process still needs a native window and renderer.

Wait for the stdout line `SALIMON_E2E_READY scenario=<name> seed=<n> step_ms=<n>`
before sending input. It is emitted after window and renderer initialization;
the next stdout line is a versioned JSON ready event. Send newline-delimited JSON
requests on stdin, one at a time, and read one JSON response per request. The
legacy ready line is not a JSON response. Example:

```json
{"protocol":1,"id":1,"op":"key","key":"forward","pressed":true}
{"protocol":1,"id":2,"op":"step","frames":10}
{"protocol":1,"id":3,"op":"key","key":"forward","pressed":false}
{"protocol":1,"id":4,"op":"inspect"}
```

`key` accepts `forward`, `backward`, `left`, `right`, `jump`/`ascend`, `descend`, `roll_left`, and
`roll_right`. Held forward/backward/left/right steer the ship when seated, and
move the character otherwise. `look` accepts finite pixel deltas `dx` and `dy`.
`aim_fragment` accepts an existing `fragment_id` and computes mouse-look deltas
toward its current center. It changes only camera aim; pickup still requires the
normal range, line of sight, and F key, and no player or object is teleported.
`interact` uses the same aimed/range interaction as E, so walking and looking at
the cockpit seat is required to enter it; interacting again leaves it. `landing`
uses the same L action and its cockpit and proximity gates. `thruster` accepts a
`direction` of `1` or `-1` and applies one Up/Down Arrow step when seated. `step`
accepts 1–600 `frames` of the launch-configured duration. `inspect` returns
player pose/location, ship pose/flight/door/control/telemetry, the aimed
interaction target, and all catalog bodies with ship surface distances.
`player.ship_local_eye_position_meters` expresses the eye in the current ship
frame for every location, including surface/doorway positions where
`player.local_ship_position_meters` is null. This allows door-collision
assertions without subtracting large world coordinates in scenario files.

Responses include `protocol`, the caller's string or numeric `id`, `ok`, and
either `result` or `error` with `code` and `message`. Requests need protocol 1,
an ID, and an operation. Lines are limited to 16 KiB. A command waiting longer
than five seconds gets a structured timeout; expired queued commands are dropped.
Only E2E launches start a stdin reader; ordinary launches do not consume stdin.
The JSON channel does not expose mutable scenario state after setup.

Setup or GPU/window failure exits nonzero with a diagnostic on stderr/log output.
Close the window to finish the run. Scenario flags without `--e2e`, malformed
values, and unknown scenarios fail before opening a window. Normal launches use
the original monotonic update clock and default starting state.

## Lifecycle contract

- Create window-bound rendering state only after the native application is
  ready to resume.
- Route nonzero drawable-size changes to the renderer and avoid configuring or
  drawing a zero-sized surface while minimized.
- Schedule continuous redraws while the window is active.
- Rebuild or reconfigure rendering state after recoverable surface loss and exit
  cleanly for a close request or unrecoverable GPU failure.
- Measure frames from a monotonic clock in the runtime. The renderer must not
  become the owner of simulation time.
- Advance portable character, ship, and camera-fixture state with a bounded
  monotonic delta. Reset only the update clock across lifecycle gaps; preserve
  portable state across renderer reconstruction.

Press **F3** to show or hide the engineering diagnostics overlay. It is hidden by
default and remains a developer view rather than a normal gameplay HUD. Runtime
frame observations feed the diagnostics crate after each successful present;
the prior published image is supplied to the renderer on the next frame. Timing
history resets across suspension, occlusion, zero-sized drawables, and renderer
reconstruction so pauses do not contaminate FPS data.

Gameplay view is the default. WASD/Space/mouse events are translated to typed
character input. Looking at the cockpit while close to its seat from inside the
ship presents `Press E to use`; E performs contextual cockpit/door interaction.
The doorway and surface positions can still operate the landed door but cannot
enter cockpit control. F2 toggles the camera fixture, which approaches
automatically from a six-body compressed Solar System overview to Earth's
meter-scale surface markers and retreats in a loop.
Escape releases the mouse-look cursor lock and a click in the game view captures
it again.
Press **P** on its initial key press to
pause/resume the transition, **R** to restart it at the far endpoint, and **N**
to select the exact 12 m near-surface dwell and pause it for inspection. The
keys **1–6** select Sun, Mercury, Venus, Earth, Moon, and Mars, respectively,
and restart that body's tour. Earth is the initial target; R retains selection.
The runtime translates those `winit` events into typed world commands; raw platform
events never cross the world boundary.

While seated in the cockpit, W/S pitch, A yaws left, D yaws right, and
Left/Right Arrow roll.
Up/Down Arrow change direct-speed thruster power by one percentage point on each
initial press. The current practical metric speed and percentage are published
on the physical cockpit screens through `ShipMeshInstance::instruments` on every
gameplay frame, including while the player walks away from the seat. The port
screen shows live stored/capacity Core energy; the starboard screen shows the
nearest body within 3 Mm, its surface distance, and approaching/receding/zero
radial speed, or a clear out-of-range state. Mouse look
remains independent and never steers or recenters the ship. Leaving with E clears
held steering but preserves heading, thruster, and autonomous forward motion.
The speed screen preserves the ship snapshot's direct-speed reading. Separate
snapshot velocity keeps nearby radial telemetry accurate during assisted landing
and takeoff without redefining the direct-speed field.

While controlling the cockpit, L starts assisted landing whenever the ship is
inside a solid body's `1.15R` volume. When landed, the same key starts takeoff;
the door must be closed. Contextual prompts and interlock/progress messages remain
in the native window title. Automatic sequences continue after leaving
the cockpit, and runtime composition switches character radial gravity to the
active Mercury, Venus, Earth, Moon, or Mars surface frame.

Runtime maps contextual interaction zones to the enlarged cockpit and exit-door
markers. It retains the 0.05 m gameplay near plane and passes the baked-scale GLB
to the renderer without an extra transform.

## Boundaries

Keep behavior in its owning portable crate. The runtime retains character, ship,
and camera objects only as the composition root. It maps the snapshot's six bodies to
`SphereInstance` values with generic material styles, maps the Sun to `PointLight`,
keeps the three precision-marker cuboids separate, maps the ship snapshot to a
generic mesh instance with live renderer-owned `CockpitInstruments` data, and
supplies diagnostics with nonnegative camera-to-body surface observations for
every catalog body; diagnostics displays the closest.
`winit` integration remains here until platform-specific behavior justifies an
adapter under `client/platform/`. The runtime must not acquire GPU, backend,
networking, or persistence responsibilities.

See [README.ai.md](README.ai.md), [architecture.md](architecture.md), and
[invariants.md](invariants.md) before changing lifecycle or timing behavior.
Use the root [native smoke check](../../README.md#native-smoke-check) to validate
launch, resize, minimize/restore, close, and relaunch behavior.

## Planetary deposit presentation

Gameplay materializes deterministic deposits within 120 m of the player's eye
using the world generator and the session seed (zero for normal launch). The
precision tour remains unchanged. `resource_presentation.rs` maps the world
snapshot into four authored Blender mesh variants per material. Stable deposit
identity selects the variant; uniform scale inscribes it in the spherical
physical bound at its absolute f64 anchor. These meshes use the renderer
resource pipeline; see [authored presentation](architecture.md#authored-iron-deposits-86).
The renderer
receives only generic presentation DTOs, with no material IDs or mining state.
Depleted deposits are omitted by the mapping. The stateless generator is reconciled with the world-owned mining session
journal before every presentation, targeting, and inspection query. Leaving
120 m unloads a deposit from the active list without discarding partial or
depleted state; returning restores the same ID/mass and omits depleted visuals.

Automation exposes `world.deposits` and `world.nearest_deposit` with stable body-scoped IDs, material keys,
world/body-local/ship-local positions, mass/state, physical radius, and visual
color/extents. `world.deposit_query_error` is null on successful queries.
`world.deposits_by_id` indexes the same active snapshots by stable ID so E2E
checks can prove a specific deposit was unloaded without depending on list order.
`scenarios/resource-streaming.json` walks beyond the active radius and returns
after partial extraction and depletion; its evidence variant captures both
unloaded areas and restored deposits.

## Handheld mining (#44)

Press **1** to equip the mining tool; **2–5** selects empty slots and stows it. On a planetary surface, aim the center
marker at a deposit within **4 m**, then hold **F** or the **left mouse button**.
The tool removes **2 kg/s** while aim, range, line of sight, and remaining mass
are valid. Release to stop; changing view, releasing the cursor, or losing focus
clears held input. Hull/gate/engine proxies and the solid planet obstruct mining.
An authored tool and illuminated indicator show equipped/active state; contextual
prompts explain equip/use. Tool gear does not occupy world-object carry capacity.

Automation keys `slot_1` (equip), `slot_2`–`slot_5` (stow) and `mine` (held press/release)
use the same input path. `inspect` exposes `mining.equipped`, `held`, `active`,
`target`, `range_meters`, `rate_kg_per_second`, and diagnostic
`extracted_mass_kg`. Deposit inspection and visuals read world-owned session
mass deltas. Extraction is advanced only by simulation steps in E2E mode.
Physical collection uses F as described below.

## Physical resource fragments (#45)

Mining ejects visible material-specific fragments from the deposit. A piece grows
up to 2 kg before the next piece starts; its size follows material density and
mass. Fragments remain in the local session after stowing the tool or leaving
the active area. Loose pieces move under gravity and contact, and follow the player while carried.
Iron ore has an angular rust-and-graphite cluster, silicate rock a low layered
shape, and water ice a tall cyan crystal cluster. Each uses two authored Blender
mesh variants selected by stable fragment identity within its physical bound.
`world.fragments` exposes nearby IDs, source-deposit IDs, material keys, mass,
volume, side length, absolute pose, and visual extents. `world.fragment_count`
and `world.fragment_mass_kg` inspect all session output (diagnostics, not inventory).
Presentation and nearby inspection use the same 120 m query. Mining evidence
screenshots show both fractional output and the pieces left after depletion.

## Physical carrying (#46)

On a planetary surface, aim the center marker at a fragment within **3 m** and
press **F**. Fragment bounds, solid terrain, and hull/gate/engine sight proxies
validate the target. One shared domain `CarrySlot` holding `WorldObjectId` represents the permanent limit
for all future world-object kinds; there is no upgradeable capacity or resource
inventory. Equipped mining gear stays separate. Pickup clears toolbar selection;
1–5 is ignored while carrying, and drop leaves the toolbar deselected. Aiming at
another fragment displays the one-object limit; F releases the held object.

The same physical entity follows the eye/look pose, stays visible at the player's
left hand, and retains its material, mass, volume, provenance, and orientation.
Collecting a partial piece seals its mass: further mining starts/grows a different
loose piece. Press **F** again to release it from hand height. The piece falls,
bounces lightly, and settles against the ground or other fragments. Inside the
ship, simulation uses ship-local coordinates so a settled pile travels with the hull.

The `grab_drop` automation key uses contextual F pickup/drop; `interact`
operates cockpit/door E interactions only. Inspection exposes
`carrying.object_id`, `target_id`, `context`, and `last_action_feedback`; each
nearby physical fragment includes `carried`, distance to player, and ship-local
position. The final feedback is diagnostic history, while gameplay transient
feedback expires after three seconds. Carry poses synchronize on movement and
look input. `scenarios/carrying.json` verifies pickup, occupied-slot rejection,
release and subsequent pickup, tool independence, conservation,
and visible entity proximity after walking. The evidence variant adds named
screenshots, and required Linux CI runs both variants.

## Contextual resource UI (#48)

The existing bottom action bar shows the aimed deposit's catalog material name,
approximate remaining kilograms, and untouched/partly-mined state within the
production 4 m mining range and line of sight. Inspection works with the tool
stowed; extraction still requires equipping it. The second line offers equip,
mine, active-mining release, or stow controls according to current state.
Aimed loose fragments show material and approximate kilograms with pickup;
carrying shows drop or the permanent one-object limit when aiming
at another fragment. Ship interaction messages and short action feedback keep
their existing priority. With no target, no carried object and a stowed tool,
resource context disappears. There is no inventory panel or resource balance.

`resource_ui.context` in test-mode inspection uses the same context builder as
the rendered action bar. The mining and carrying baseline scenarios assert
material/mass/state, mining controls, empty context, pickup and carrying limits;
their evidence variants capture deposit targeting with the tool stowed, active
mining, loose fragment targeting and blocked second pickup.

Test-mode windows request a 1280 x 800 physical drawable so the full viewport
and bottom action bar fit the dedicated CI display. Normal launch retains the
1920 x 1080 benchmark drawable. Desktop capture must include the entire window
when validating HUD legibility.

## Ship-relative EVA (#37)

The runtime supplies both the current ship frame and world velocity to the
character update after advancing the ship. Exiting a flying airlock captures
that velocity and orientation once; detached world motion is then independent
of later ship steering/thrust. Mouse look controls EVA yaw/pitch; WASD translates
in that view, Space ascends, and Left Shift descends. Normalized 3D input uses
3.8 m/s flight assist: releasing input removes only controlled translation,
preserving inherited world drift. Re-entry adopts ship gravity and velocity,
while preserving the world look direction.

Automation exposes `player.velocity_meters_per_second`,
`speed_meters_per_second`, `relative_velocity_meters_per_second`,
`relative_speed_meters_per_second`, and the existing ship-relative eye position.
The velocity fields describe EVA motion; interior/cockpit use the ship-frame
transport velocity. `moving-eva.json` tests a 25,000 m/s ship and ten seconds of
no-input drift, vertical/pitched controls, door collision, and re-entry.
Nearby-body gravity transition is handled separately by #36.

## Nearby-body EVA (#36)

`eva-approach` is an initial-conditions-only fixture: Earth surface distance
3,200,000 m, ship-forward radially inward, zero initial speed. The scenario uses
normal thruster/airlock/walking actions to exit at 25,000 m/s and cross influence.
`player.location` reports `NearbyBody` while airborne under radial gravity;
`player.nearby_body` reports the selected solid body and player surface distance.
Velocity inspection includes inherited motion plus gravity and assisted input.
Body selection uses the player's world position independently of ship telemetry.

### Loose fragments aboard the ship

Fragments dropped on the cabin deck retain ship-local support anchors. Inspection
reports `reference_frame: "ship"` for nearby loose ship-supported fragments and
`"world"` for loose surface fragments; carried fragments keep the same identity
through pickup and release. `fragment-transfer.json` covers cabin release,
retrieval, surface removal, and ship-local stability in flight. `resource-loop.json`
continues from landing and mining through delivery onto the main cabin deck.

## Toolbar validation (#132)

The [carrying baseline](../../scenarios/carrying.json) and its
[evidence variant](../../scenarios/evidence/carrying.json) protect the five-slot
loadout, initially absent selection, and selection of all slots. On the surface,
every empty slot hides the held tool and prevents extraction during an F hold;
slot 1 restores the tool and permits mining. Equipped F pickup clears selection,
all five keys stay blocked while carrying, and F drop leaves selection absent.
A fresh explicit 1 press restores the tool without resuming mining.

The evidence route captures equipped slot 1, selected empty slot 2, carrying
without selection, and post-drop deselection. See the
[runner coverage](../../scripts/README.md#toolbar-gameplay-coverage-132).

`restore_look` reverses the preceding `aim_fragment` camera deltas so a scripted
walking route keeps its intended heading after the pickup checkpoint.
