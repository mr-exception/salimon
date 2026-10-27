# Native E2E scenario runner

Build standalone executables with the [cross-platform build interface](BUILDING.md).
Pass its staged executable to `--binary` to test the distribution artifact.
For normal launch and real OS keyboard input, use the separate
[packaged black-box smoke command](PACKAGED_SMOKE.md).

Run from the repository root on a machine with a native window and working GPU:

```sh
scripts/salimon-test run scenarios/landed-earth.json
scripts/salimon-test suite
```

The command builds `salimon-client` with the locked dependencies once, then
launches a fresh game for each scenario. Use `--binary /path/to/salimon-client`
to reuse a prebuilt binary. It prints a single JSON report to stdout and exits
nonzero if building, launch, a protocol command, a condition, or any scenario
fails. It terminates the game after every result and kills it if termination
does not complete within two seconds. The `suite` command discovers all
`scenarios/*.json` files in filename order.

Each JSON file contains `setup` (`scenario`, `seed`, `step_ms`), an overall
`timeout_ms`, and ordered `steps`. Setup uses the native fixtures listed
in [runtime documentation](../client/runtime/README.md). The default overall
timeout is 60000 ms; each step defaults to 5000 ms. Startup shares the overall
timeout. A step can specify its own `timeout_ms`. Example:

```json
{
  "setup": {"scenario": "orbit-earth", "seed": 42, "step_ms": 16},
  "timeout_ms": 60000,
  "steps": [
    {"assert": {"path": "ship.flight_state", "equals": "Flying"}},
    {"action": {"op": "thruster", "direction": 1}},
    {"wait": {"path": "ship.thruster_percentage", "gte": 1}, "frames": 1, "timeout_ms": 3000}
  ]
}
```

`action` takes any operation and its parameters from the native command
protocol: `inspect`, `step`, `key`, `mouse`, `look`, `aim_fragment`, `restore_look`, `interact`, `landing`, `thruster`, `crafting`, or `consume`.
`assert` inspects state once; `wait` inspects until matched, advancing the
simulation by `frames` (1–600, default 1) between checks. Condition `path`
uses dot-separated object keys and zero-based array indices, such as
`world.bodies.3.name`. Choose one comparison: `equals`, `not_equals`, `gt`,
`gte`, `lt`, `lte`, or `approx`. Numeric `approx` accepts a nonnegative
`tolerance` (default 0.000001). Waits fail when their deadline expires.

Runner contract tests need only Python 3 and can run without a display:

```sh
python3 -m unittest discover -s scripts -p 'test_*.py'
```

## Ship/EVA prerequisite suite (#39)

Before resource transfer work, run the four prerequisite routes together:

```sh
python3 scripts/salimon-test suite --group ship-eva
python3 scripts/salimon-test suite --group ship-eva --evidence \
  --screenshot-command '["python3", "scripts/capture_settled.py", "{path}"]'
```

Both commands support `--binary` and `--artifacts`. The first runs cockpit-nose,
space-airlock, moving-eva and nearby-eva in that order. The second runs their
synchronized screenshot variants; capture failure fails the suite. Each scenario
starts a fresh native client and emits the shared result, step snapshots,
protocol/process logs and failure artifacts. A missing prerequisite file fails
rather than silently reducing coverage. `--group` and `--evidence` require `suite`;
`suite --evidence` without a group runs every evidence variant.

The moving route checks all three ship-relative position axes within 0.02 m
after 600 frames (9.6 seconds) without input, zero relative speed within 0.02 m/s,
and ship/player inherited speed of 25,000 m/s. It also checks velocity after
re-entry. The nearby route checks open-space mode before threshold crossing,
Earth selection inside 3,000,000 m, continuous altitude and bounded radial
acceleration. Cockpit-to-nose traversal and airlock collision checks cover the real layout.
Run this evidence group manually against a locally staged release binary.
See [historical issue discussion](https://github.com/mr-exception/salimon/issues/39) for results and screenshots.

## Integrated Scout and construction acceptance (#551)

Run the complete maintained capture group against a newly built native binary:

```sh
python3 scripts/salimon-test suite --group scout-construction --evidence \
  --binary target/debug/salimon-client --artifacts artifacts/scout-construction \
  --screenshot-command '["native"]' --summary
```

This group always selects the evidence routes, including when `--evidence` is
omitted. Its explicit inventory fails on missing files, launch, action, state or
capture failures. Keep capture enabled for acceptance. It covers cockpit/flight,
Core, cargo door/ramp/carrying, habitation, galley inspection and lighting,
Core upgrades, blueprint placement/intake/completion, two-deck traversal,
module installation/upgrade/restoration, damage/detachment, stop/takeoff and
offline resume. The three cancellation routes use a declared 20 kg hull-cell input
and ordinary work/control commands at exactly 0/50/99%; they assert no early
installed colliders and one physical iron refund of 20/10/0.2 kg. Fixtures do
not establish actual mining/acquisition balance or physical device input.

For full machinery/Archive coverage also rerun the retained production drivers:

```sh
python3 reports/issue-489/walkthrough.py --binary target/debug/salimon-client \
  --output artifacts/scout-machinery
python3 reports/issue-505/verify.py --binary target/debug/salimon-client \
  --output artifacts/scout-archive --scale 1
python3 reports/issue-505/verify.py --binary target/debug/salimon-client \
  --output artifacts/scout-archive-power --power
```

Use fresh output directories and the Linux Xvfb/software-Vulkan setup in
[validation](../docs/validation.md#native-graphical-gates). Preserve per-step state,
protocol/process logs, original images and source/binary hashes. The current
[#551 report](../reports/issue-551/README.md) records coverage, actual failures and
human/platform/performance handoffs. These fixed-step protocol routes do not
establish human appearance approval, Metal compatibility or physical 60 FPS.

## Resource collection suite (#51)

Run the complete resource regression suite, then its synchronized visual routes:

```sh
python3 scripts/salimon-test suite --group resource-collection
python3 scripts/salimon-test suite --group resource-collection --evidence \
  --screenshot-command '["python3", "scripts/capture_settled.py", "{path}"]'
```

Both commands accept `--binary` and `--artifacts`. The six required routes run in
this order: resource-deposits, mining, carrying, fragment-transfer,
resource-streaming, resource-loop. Each starts a fresh seeded native client. Missing scenario
files, failed launches, failed assertions and failed evidence captures fail the
aggregate report; no route is silently skipped. Run the evidence
suite manually against a staged release executable and retain its runner artifacts.

| Boundary | Regression coverage |
| --- | --- |
| Distribution/generation | `client/world/tests/resource_distribution.rs` and `resource_generation.rs`: deterministic seeds, material weights, geometry, ordering, reload identity and invalid inputs. |
| Extraction/physical output | `client/world/tests/mining.rs` and `resource_fragments.rs`: range, aim, line of sight, depletion, nonnegative mass and conservation across all materials and time steps. |
| Carrying/transfer | Runtime carrying contracts and production-action walkthroughs in `client/runtime/src/automation.rs`; native carrying and fragment-transfer routes protect one-object rejection, round trips, cabin placement, removal and ship-relative support in flight. |
| Stream restoration | `client/world/tests/resource_streaming.rs` and native resource-streaming: partial and depleted source IDs leave the active radius and return with unchanged mass and no duplicate output. |

Evidence variants add only named screenshot checkpoints to the baseline actions
and assertions. Every run preserves structured results, per-step state and logs;
failures also preserve available state and screenshots. Runner contracts cover
failed capture, process cleanup and assertion/startup failure; suite contracts
verify that failed launches retain six isolated result directories. Native
macOS/Windows graphics need local GPU validation; manual Linux runs may use software Vulkan.

## Deterministic first resource loop (#52)

`resource-loop.json` is one uninterrupted reference scenario, using seed 0 and
16 ms steps. `resource-approach` starts seated 1 km directly above the Earth
reference landing site. Normal assisted landing reaches the surface; all later
walking, mining, pickup and placement use production controls. No scenario
command teleports the player, inserts deposits or fragments, or sets fragment state.

```sh
python3 scripts/salimon-test run scenarios/resource-loop.json
python3 scripts/salimon-test run scenarios/evidence/resource-loop.json \
  --screenshot-command '["python3", "scripts/capture_settled.py", "{path}"]'
```

Run both variants manually through baseline discovery and the
resource-collection evidence group. The route checks two actual locally
materialized collectible types (iron ore and silicate rock), walks to the stable
silicate source, verifies physical mass/identity/volume after mining, leaves and
returns across the 120 m streaming boundary after partial mining and again after
depletion, then carries fragment 1 through the aft door and places it on the main
cabin deck. The fragment keeps its identity, material and ship-relative support.
The evidence variant adds named screenshots without changing actions or assertions.

Every run writes structured results, per-step state snapshots and process/protocol
logs; evidence captures also save their authoritative states. See the shared
artifact format below and [historical issue discussion](https://github.com/mr-exception/salimon/issues/52).
This scenario covers the first local resource loop; refining, crafting, trading,
survival consumption, asteroid mining and backend persistence are outside it.

## Phase 0 baseline suite

The default `suite` discovers every baseline scenario, including these routes:

| Scenario | Coverage |
| --- | --- |
| `cockpit-nose.json` | Walk around the pilot chair and unified console into the short cockpit nose, then return to the cabin. |
| `landed-earth.json` | Known initial player pose and ship state; walking/aiming into cockpit control; non-pilot control rejection; closed-door exit collision; opening and exiting to the surface; closing the door outside and walking against it twice; reopening and returning inside; the open-door takeoff interlock; closing the door and completing assisted takeoff. |
| `resource-deposits.json` | Real airlock exit and surface walk to a stable generated silicate deposit; checks material, positive mass, proximity, and presentation data. |
| `resource-streaming.json` | Partial mining, walking beyond the 120 m active radius, explicit source-ID absence, return with identical mass, full depletion, and a second round trip without regeneration or duplicate fragments. |
| `carrying.json` | Physical pickup, held-F repeat suppression, equipped-tool independence, occupied/clear placement, subsequent pickup, and entity proximity after walking. |
| `mining.json` | Surface tool equip/aim/hold, exact timed extraction, aim/range/release rejection, return to the deposit, bounded depletion, and stow; no inventory credit. |
| `orbit-earth.json` | Known seeded orbit pose; thruster changes; starting assisted landing; repeated landing action cannot cancel it; leaving cockpit control during landing; autonomous completion with zero ship velocity. |

These files use only the existing `key`, `look`, `interact`, `thruster`,
`landing`, equipment keys, left-mouse hold/release, and fixed `step` operations after fixture setup. They never set
expected end states. Held movement keys are released before interacting so
cockpit authority changes cannot redirect an outstanding movement key.
Door collision checks assert both location and ship-relative eye position;
remaining outside alone would not prove the door stopped movement.

Walking routes use the fixtures' clear side aisle around the central Core and
the rear doorway. Their frame counts are deliberately tied to `step_ms: 16`.
Assisted-sequence completion polls authoritative state with bounded fixed-frame
advances instead of wall-clock sleeps. Changing the ship layout, movement
speed, or fixture coordinates requires updating the routes and checking them
again against the native client. Run `suite` at least three times when changing
these fixtures/routes; identical setup and actions should produce identical
per-step state snapshots regardless of render timing.

The required baseline suite emits structured state/results/logs and automatic
failure capture without requiring screenshots on successful runs. The matching
`scenarios/evidence/landed-earth.json` and `scenarios/evidence/orbit-earth.json`
add named screenshot checkpoints for the same actions and assertions; contract
tests keep these variants synchronized. Run both individually for visual QA:

```sh
scripts/salimon-test run scenarios/evidence/landed-earth.json
scripts/salimon-test run scenarios/evidence/orbit-earth.json
```

Use an unobscured native desktop or dedicated X11 display with Vulkan support.
Linux Xvfb plus Mesa lavapipe can verify the native protocol/gameplay paths with
software rendering; it does not establish hardware performance or native macOS/
Windows graphics fidelity. Missing display/GPU support is a failed launch,
not a passed or skipped scenario. Visual runs require a working capture helper.

## Continuous integration coverage

Every push to `main`, pull request and manual dispatch runs **Source build
(Ubuntu)**: one `cargo build --workspace --locked` in the normal debug profile.
The source-build workflow retains Cargo caching, cancels superseded runs per PR/branch and
has a 15-minute timeout. It does not stage release/debug packages or upload
artifacts, and does not run tests, model processing, benchmarks, graphical
scenarios or packaged smoke. macOS/Windows builds also run in the weekly workflow.

Run the baseline/evidence suites above deliberately on a native desktop or a
prepared Linux Xvfb/Mesa environment. Capture, display and assertion failures
remain failures when these commands are run. Keep their `artifacts/e2e/` results,
logs and screenshots for review; periodic CI uploads them with 30-day retention. Frozen model capture
and temporal comparison commands remain in the [visual fixture guide](../scenarios/visual/README.md).
The separate [periodic workflow](../docs/periodic-ci.md) runs these Linux suites
nightly and benchmarks/platform builds weekly. It has no PR/push trigger. Its
manual dispatch selects `nightly` or `weekly`, with an optional forced rerun.
Each run has summaries, raw artifacts and a history dashboard; GitHub Pages
publishing is opt-in. The source-build dispatch has no `visual_evidence` option.

See [automatic CI and check-name migration](../docs/validation.md#automatic-ci)
when updating required checks. Repository protection settings are not changed
by the workflow. No source-build result establishes gameplay, visual quality,
native OS input, hardware performance or macOS/Windows compatibility.

## State and visual evidence

Every scenario gets a unique `artifacts/e2e/run-*/` directory (including parse,
startup, assertion, and timeout failures). Override the root with `--artifacts
/path/to/evidence`. `result.json` is the durable schema-versioned report also
returned in the CLI's stdout summary. It records normalized setup, executed
steps, start/duration timings, responses, authoritative state snapshots,
assertions with actual values/presence and polling attempts, errors, and exit
status. Each executed step also has `step-NNN.json`; `scenario.json` preserves
the normalized input. Repeated/concurrent runs never overwrite one another.

The existing comparisons work for player positions, interaction availability,
flight/door state, and any future inspection field. `exists: true/false` tests
path presence (a present `null` value still exists). `contains` checks array
membership, an object key, or a substring. Examples:

```json
{"assert": {"path": "world.bodies.0.name", "exists": true}}
{"assert": {"path": "ship.flight_state", "contains": "Landed"}}
{"assert": {"path": "interaction", "not_equals": null}}
{"screenshot": "cockpit-before-takeoff"}
```

A `screenshot` step saves `step-NNN-NAME.png` and the accompanying authoritative
state. Names accept 1–80 ASCII letters, digits, underscores, or hyphens. A named
checkpoint fails explicitly if capture is unavailable or fails. On gameplay
failure the runner attempts `failure.png` **before terminating the game**. It
records capture failure/unavailability without replacing the original error.
`failure-state.json` contains the latest successfully inspected state; following
protocol timeout/disconnection it may precede the failure. No further protocol
request is attempted after timeout, avoiding response desynchronization.

Capture defaults to macOS `screencapture -x` (Screen Recording permission may be
needed), X11 ImageMagick `import -window root` on Linux (`sudo apt install
imagemagick`; requires `DISPLAY`), or Windows PowerShell's System.Drawing desktop
capture helper. Wayland, headless sessions, and restricted desktops may require
a custom helper. Capture is an **OS desktop screenshot**: keep Salimon visible
and unobscured on a dedicated display; desktop contents outside the game may be
included. It captures the last presented frame, which may lag the latest state
on a slow renderer. Visual evidence is diagnostic; simulation assertions use
state, never pixel matching. Rendering/GPU fidelity must be checked separately
when running a software renderer.

Use a trusted capture helper to target only the game window or another display:

```sh
scripts/salimon-test run scenarios/evidence/landed-earth.json \
  --screenshot-command '["my-window-capture", "--output", "{path}"]'
```

The option is a JSON **argv array**, executed without a shell, and must include
`{path}` for the output PNG. It is a local CLI setting, never an executable
command embedded in a scenario. `--screenshot-command '[]'` disables capture
explicitly; named screenshot steps then fail. Each helper has a bounded timeout
and must exit zero and produce a PNG. Helper output is preserved as
`NAME.capture.log` when available. Automatic failure capture adds up to five
seconds to cleanup, independently of the expired scenario deadline.

To diagnose a run, start with `result.json`'s first failed step and compare its
assertion `actual`/`actual_present` with the expected condition. Read the step
state and `failure-state.json` to distinguish gameplay failure from stale or
missing inspection data. `protocol.jsonl` preserves requests/responses including
all wait polling states; `stdout.log` and `stderr.log` preserve full process
output (the in-memory exception only includes a short stderr tail). Then inspect
the named checkpoint/failure PNG for context and its capture log for display
errors. A missing screenshot does not mean a gameplay assertion passed.

The opt-in evidence scenarios add visual checkpoints without imposing screenshot
dependencies on the default suite. Use them on an unobscured native display or a
dedicated Xvfb display with a compatible Vulkan implementation:

```sh
xvfb-run -a -s '-screen 0 1280x800x24' scripts/salimon-test \
  run scenarios/evidence/landed-earth.json --binary target/debug/salimon-client
```

The manual resource-deposit visual scenario saves a `generated-silicate-deposit` screenshot and authoritative
state within 4 m of the deposit. A screenshot failure fails the run. This uses the
same real gameplay route as `scenarios/resource-deposits.json`; no state
teleportation or fixture deposits are used. Run it locally with:

```sh
scripts/salimon-test run scenarios/evidence/resource-deposits.json
```

Deposit colors/proportions are validation greyboxes: orange iron ore, muted
green-grey silicate rock, and cyan water ice. Inspect `world.deposits` and
`world.nearest_deposit` for the generated identity, material, physical bounds,
remaining mass, positions, player distance, and renderer-facing geometry.

## Mining evidence

`scenarios/evidence/mining.json` adds equipped, actively mined, and depleted
screenshots to the same deterministic mining route. Run it with a working
native capture helper to inspect the greybox handheld tool and removed deposit.
Its baseline is included when the default suite is run manually. Both variants
assert physical fragment identity, material, mass, volume, pose, and bounded
output after depletion. Extraction totals and session fragment totals are
diagnostics only, never inventory. Current authored fragments emit from exposed deposit facets and settle through
physics; see the surface-emission sequence below.
Run this mining evidence variant manually; capture errors fail the run. Historical [issue #45 discussion](https://github.com/mr-exception/salimon/issues/45) records
the initial Linux validation.

## Local streamed resource state (#50)

`scenarios/resource-streaming.json` uses the same real mining route, releases
mining input, walks 153.6 m away and back, and checks the source is absent from
`world.deposits_by_id` while away. It verifies partial mass restoration, then
repeats after depletion and verifies zero mass, no visual/target, and unchanged
physical output totals. The baseline is included in the default suite; manually run `scenarios/evidence/resource-streaming.json` with named screenshots.
No teleportation or test-only state mutation is used. Local session state is
retained only until the world/session ends, without disk or backend persistence.

## Physical pickup/drop (#46)

`scenarios/carrying.json` follows the real mining route, approaches and aims at
physical pieces, and uses `grab_drop` (F). Legacy `pickup`/`drop` aliases also route contextual F. Mining uses
`{"op":"mouse","button":"left","pressed":true}` (false to release);
the old `mine` keyboard alias is rejected. It verifies E cannot grab/drop, held-F repeat
suppression, material/mass/identity preservation, tool independence, release,
subsequent pickup, and carried entity proximity after movement. The evidence
variant adds first pickup, repeated-F suppression, placement and moving carry
screenshots. Both remain available for manual verification.
Run the evidence variant with a working display/capture helper:

```sh
scripts/salimon-test run scenarios/evidence/carrying.json
```

The runtime contract test also executes the same scenario against the actual
portable gameplay update/input path without a GPU, checking every assertion.
This is logic coverage; a native E2E run establishes launch/render/capture coverage.

## Contextual resource UI (#48)

The existing mining/carrying routes also assert `resource_ui.context`, including
material identity, rounded mass, partial depletion, tool-stowed inspection,
active mining, empty context after looking away and stowing, loose-object
pickup, and the one-object limit. Their evidence variants add
`deposit-context-tool-stowed`, `active-mining-context`, and
`fragment-target-context` checkpoints alongside `blocked-second-pickup`.
Both variants already run in the required Linux native-build job.

## Cockpit nose walkthrough

The default suite includes `cockpit-nose.json`. Its evidence variant records the
walkable nose from the character's actual position. Run:

```sh
python scripts/salimon-test run scenarios/evidence/cockpit-nose.json \
  --screenshot-command '["python", "scripts/capture_settled.py", "{path}"]'
```

The helper waits 0.5 seconds before the normal platform screenshot command to
reduce stale frames from asynchronous GPU presentation. It is not a GPU fence;
authoritative movement/state assertions remain the verification source. Capture
errors still fail the scenario and the runner's normal deadlines apply. Use a
dedicated unobscured test display. The prior cargo-room route and its evidence
are tracked under historical [issue #34](https://github.com/mr-exception/salimon/issues/34).

## Space airlock access (#35)

`open-space` initializes a stationary flying ship 10 km beyond the shared
3,000,000 m nearby-body surface-distance threshold. `space-airlock.json` uses
normal controls to leave the cockpit, walk to the gate, open it, exit without
planetary snapping, close it from outside, verify blocked re-entry, reopen and
re-enter, close it again, and verify blocked exit. It runs in the default suite.
`scenarios/evidence/space-airlock.json` adds four screenshot/state checkpoints and
is available for manual verification. Use the settled capture helper as for the cockpit nose.

The portable runtime test executes the same access scenario, while ship tests
cover inclusive threshold locking and assisted-sequence locking. Moving-ship
velocity inheritance and free 3D EVA are #37, and nearby-body transitions #36.

## Moving-ship EVA (#37)

`moving-eva.json` starts from the `coasting-space` fixture with explicit
25,000 m/s world +X momentum and zero throttle, then follows the airlock route. It verifies inherited world velocity,
no-input ship-relative stability over 600 fixed frames, Space/Shift vertical
translation, pitched/yawed view-relative translation, assisted stop, closed-gate
collision, and clean interior re-entry. The baseline runs in the default suite;
the synchronized evidence variant adds drift, vertical-flight and re-entry
screenshots and is available for manual verification. Portable runtime tests execute both
stationary and moving routes, and character contracts compare 10/20/100 ms
updates and verify detached motion/view independence from ship changes.

## Nearby-body EVA (#36)

`nearby-eva.json` uses the `eva-approach` initial fixture and normal gameplay
controls to leave a moving ship, stay in open-space mode outside influence, then
cross Earth's inclusive shared surface-distance threshold. Assertions check the
player-selected body, nearby movement mode, continuous altitude, inherited
25,000 m/s motion, and bounded radial acceleration. The baseline runs in the
default suite; its evidence variant adds three screenshot/state checkpoints and
is available for manual verification. Character contracts additionally check boundary mode
changes, gravity integration across update sizes, influence exit, and contact.

## Planet/ship fragment transfer (#47)

`fragment-transfer.json` uses the real mining/pickup route, carries one fragment
through the open gate, drops/retrieves it inside, carries it back outside, then
repeats. It checks identity/mass, ownership, world versus ship support, re-entry,
and stable ship-local placement while walking away/back and during assisted
takeoff and 25,000 m/s flight. The portable runtime executes the same route,
checking carried proximity after each input/step. Narrow contracts cover rotated
and translated ship frames, pickup detachment, invalid placement, and sight
obstruction. The default suite includes the baseline; manually run
the evidence variant with settled capture:

```sh
python scripts/salimon-test run scenarios/evidence/fragment-transfer.json \
  --screenshot-command '["python", "scripts/capture_settled.py", "{path}"]'
```

Inspection exposes each visible fragment's `reference_frame` (`ship` for loose
interior anchors, `world` otherwise) alongside its existing carried flag and
ship-local pose. These fields describe actual physical entities, not inventory.

## Ship-supported fragments

`fragment-transfer.json` verifies that a loose fragment dropped on the cabin deck
keeps its ship-local pose while the ship moves, and that pickup and surface
placement transfer the same physical entity between reference frames. The
resource loop additionally delivers a mined fragment into the cabin. The former
dedicated cargo-room behavior is described in [issue #49](https://github.com/mr-exception/salimon/issues/49).

## Lower cockpit windows (#38)

`lower-cockpit-windows.json` starts in the deterministic Earth approach and uses
normal free-look to inspect the lower pane at 30 degrees down / 34 degrees to
each side of the console. It starts assisted landing, checks the surface distance
falls below 25 m, waits for touchdown, inspects both sides, restores the forward
view, takes off and leaves cockpit control. Its evidence variant records eight
native screenshots, including the seated low-altitude surface view. The default
suite runs the baseline; run the settled-capture evidence variant manually.

```sh
python scripts/salimon-test run scenarios/evidence/lower-cockpit-windows.json \
  --screenshot-command '["python", "scripts/capture_settled.py", "{path}"]'
```

The authored pilot view begins at -0.10 radians; mouse deltas in this route
include that offset. Asset validation independently checks actual exported
triangle sightlines. Passing gameplay assertions alone does not establish visual
correctness; inspect the native checkpoints as well.

### Equipment toolbar input/state

Automation `key` accepts `slot_1`–`slot_5` (aliases `1`–`5`) through the
production numeric-key route. Inspection returns `equipment.slots`, a five-item
array initially `["mining_tool", null, null, null, null]`, and
`equipment.selected_slot`, a one-based slot number or null. Selection is absent
at launch, cleared on pickup and blocked while carrying; dropping leaves it
absent until another explicit selection. The carrying baseline verifies all
five slots and the pickup/drop lockout. The bottom-center toolbar, held tool,
reticle and mining gates derive from this selection. Select `slot_1` to equip;
select `slot_2`–`slot_5` to stow and cancel held mining. The old toggle command
`equip_mining_tool` and M binding are removed; use explicit slot selection.

Inspection also returns `mining.held_item_visible`, using the same held-item
DTO helper as redraw (gameplay view, surface location and selected mining tool).
It is false throughout carrying and after drop until explicit slot 1 selection
on the surface. Carrying/resource-loop baseline and evidence variants check
failed pickup preserves selection, all five inputs are blocked while carrying,
and re-selection does not resume a previous mining hold.

## Toolbar gameplay coverage (#132)

`carrying.json` checks the initial five-slot loadout with no selection and all
1–5 selection/equip decisions. At a reachable surface deposit, each empty slot
2–5 rejects a held left-mouse mining attempt without creating fragments or
extracting mass. Slot 1 then equips the visible tool and mines through that same
mouse route. Equipped pickup, all-five-slot carrying lockout, deselected drop
and explicit re-equip are checked in the remainder of the route.

The synchronized evidence variant adds `toolbar-mining-tool-selected` and
`toolbar-empty-slot-selected`; `carrying-first-fragment` and
`dropped-toolbar-deselected` establish the other two required states. The
existing `equipment.slots`, `equipment.selected_slot`, `mining.equipped`,
`mining.held_item_visible`, `mining.active` and output fields suffice.
Both routes run through baseline discovery/resource-collection evidence in
manual verification; the portable runtime also executes both JSON contracts.

```sh
python scripts/salimon-test run scenarios/carrying.json --binary artifacts/build/release/salimon-client
python scripts/salimon-test run scenarios/evidence/carrying.json --binary artifacts/build/release/salimon-client --screenshot-command '["python", "scripts/capture_settled.py", "{path}"]'
```

## Fragment pile geometry regression (#148)

`fragment-pile-floor.json` and `fragment-pile-surface.json` share the opt-in
`fragment-pile` initial fixture with seeds 0 and 1. Six physical 2 kg fragments
(two variants of each material) start above one area; production fixed steps
settle them for 5.76 seconds, then continue for 2.88 seconds. Evidence variants
capture both checkpoints. These fixtures exercise contacts independently of
mining surface emission; normal carrying/transfer scenarios still test F actions.

```sh
python scripts/salimon-test run scenarios/evidence/fragment-pile-floor.json --binary artifacts/build/release/salimon-client
python scripts/salimon-test run scenarios/evidence/fragment-pile-surface.json --binary artifacts/build/release/salimon-client
```

Use the standard platform screenshot/display setup. Compare checkpoints for
motion and contact gaps; screenshots alone do not prove conserved mass or support
removal, which the focused world/physics/runtime regressions cover.

`aim_fragment` turns the E2E camera toward the current pickup bounds of an existing
`fragment_id` using mouse-look deltas. It changes no object/player position and
does not bypass pickup range, occlusion, or the normal F key contract. Scenario
pickup checkpoints use it because rotated convex piles settle differently.

`restore_look` reverses the preceding `aim_fragment` camera deltas so a scripted
walking route keeps its intended heading after the pickup checkpoint.

## Surface emission sequence (#150)

`mining-emission.json` shares the real mining walkthrough and splits the first
second into 1/10/49 fixed-frame steps. Its evidence variant captures surface
origin, outward ejection, three repeated pieces and settling after release;
state assertions protect 2 kg/s, stable IDs and conserved 5.76 kg output. Both
are included in the resource-collection group, and the baseline in default
discovery/portable runtime contracts.

```sh
python scripts/salimon-test run scenarios/evidence/mining-emission.json --binary target/debug/salimon-client --screenshot-command '["python", "scripts/capture_settled.py", "{path}"]'
```

Long resource-collection fixed-step batches (300–600 frames) explicitly use
`timeout_ms: 30000` for the combined action and following inspection. The
default 5-second budget still applies to ordinary inputs/assertions; whole
scenario deadlines remain bounded. This accommodates complete convex piles
on shared software-rendered hosts without changing fixed steps or assertions.

The game automation server grants valid protocol-1 `step` requests of 1–600
frames a 120-second deadline for both queue expiry and reply waiting. A single
loaded support-removal update can rebuild hundreds of contacts; client/scenario
deadlines still impose their own shorter limits. The ready
event's `timeout_ms: 5000` continues to describe ordinary commands. A longer
scenario budget alone does not extend the server deadline for other commands.

## Fragment load benchmarks (#155)

See [FRAGMENT_BENCHMARKS.md](FRAGMENT_BENCHMARKS.md) for native release and CPU-only
matrices, fixture setup, measurement scopes and baseline-relative gates. The
500-fragment correctness scenario runs in the standard baseline; benchmark
wall-clock sampling remains separate from fixed-step automation.
Loaded acceptance scenarios may add `fragment_settle_steps` to setup, using the
same production preparation as settled benchmarks before renderer readiness.
`world.sleeping_fragment_count` and per-fragment `sleeping` observe the last
completed update; carrying is excluded. `contact_fragment_ids` and
`environment_supported` expose that completed support snapshot; they are observers,
not controls. Ship-local positions use stored anchors to avoid round-trip world
precision loss in inspection. See the benchmark guide for limits and
startup-deadline requirements.

Signed throttle automation uses `key` values `thruster_up` and `thruster_down`
with press/release booleans. Holding either repeats each 100 ms of simulation
time, with one point immediately on the press edge. The `thruster` action is a
single powered-cockpit percentage-point command; negative settings use reverse
thrust and do not directly assign speed.

## Realistic flight regression (#195)

`python scripts/salimon-test suite --group realistic-flight --summary --binary
/path/to/salimon-client` exercises signed throttle/coasting/reverse stop, dead-Core
mechanical refueling and automatic thrust restoration, crash recovery/reservation,
inertial takeoff, a physically onboard 500-fragment population and moving EVA.
Use `--evidence` for the same actions plus screen checkpoints. New initial fixtures
are opt-in `--e2e` scenarios; no mutation/teleport/refuel command bypasses production
interaction after initialization. The manually invoked default suite includes these routes.

`cargo test -p salimon-client flight_regression --locked` replays the new routes
without a window and adds runtime mass/consumption and cabin zero-G transition
checks. Existing ship, character and physics tests retain detailed partition,
collision, power and contact edge coverage. These checks establish portable
correctness; Linux software rendering does not certify Apple M1 frame rate.

## Apple M1 flight/Core follow-up (#198–#200)

The realistic-flight group includes `cargo-blackout.json` (500 physical pieces,
held-object zero-G drift, mechanical intake, gradual conversion, restored power
and later depletion) and `landing-speed-cutoff.json` (19.84 m/s accepted, 20.48 m/s
tangential motion rejected, uncancellable touchdown). Both use existing production
actions after opt-in fixture initialization; evidence variants add captures only.

On the native desktop, use a release build and direct Metal framebuffer captures:

```sh
SALIMON_REFERENCE_SIZE=1 RUST_LOG=info python3 scripts/salimon-test suite --group realistic-flight --evidence --binary artifacts/build/release/salimon-client --screenshot-command '["native"]'
```

The environment switch requests a 1920×1080 physical drawable (also on Retina).
These fixed-step routes prove state transitions and rendered checkpoints; use
the fragment benchmark for independently sampled native frame pacing.

Production GPU model captures, offline visual comparisons and frozen-frame
flicker checks: [visual fixture workflow](../scenarios/visual/README.md).

`compare_terrain_style.py` compares offscreen `terrain_style_reference` captures:
matched scene coverage/costs, color changes, contact sheets and timing records.
Requires the existing optional Pillow dependency. See
[terrain validation](../docs/terrain-validation.md#option-a-shader-evidence-278).

Consumable automation: M/N consumption keys are retired (#346). The Q wheel
(#377) owns selection; precision-tour `near` retains its camera meaning.

Fabricator automation (#347): `{"op":"crafting","action":"add"}` invokes the
open machine modal. Actions: `close`, `previous_recipe`, `next_recipe`, `less`,
`more`, `next_job`, `cancel`, `add`. Closed modal returns `applied:false`.
Inspection adds `fabricator` (bins, FIFO jobs, tray, power demand and UI state)
and `consumables.food` / `consumables.medical` arrays with stable item identities.

Q-wheel automation (#377): send `key:consumable_wheel` with `pressed:true/false`;
`{"op":"wheel_hover","slot":0}` selects a zero-based food/medical segment (0–4;
null clears hover). Left mouse press consumes the occupied selection once.
`wheel_next`/`wheel_confirm` key presses exercise Tab/Enter fallback. Inspection
exposes `consumable_wheel.open/held/hovered`. Actions use the same portable modal
and item transaction as native events, without desktop focus admission. Survival
effects use the same exact-item transaction; native pointer/appearance signoff
remains separately under #376.

Consumable recovery protocol (#370):
`{"op":"consume","slot":1,"job":12,"unit":1}` identifies an exact current
item (slot 1–3 food, 4–5 medical). Obtain job/unit from inspection; both must match
the selected slot. Success returns `applied:true`; empty/stale/denied/unknown
effects return `applied:false` with a reason, without removing anything. Invalid
argument shapes fail validation. This is the portable shared transaction for
#377's native Q wheel, not a new gameplay hotkey; M/N remain absent. Inspection
adds each item's validated recovery definition and `survival` current meters/
active effects. Whole-game survival clock/save activation remains #371.

## Deterministic input boundary (#313)

`salimon-test` and model visual fixtures isolate their E2E child from desktop
keyboard/mouse events and focus/resize resets of scripted held controls/time.
They set `SALIMON_E2E_ISOLATE_INPUT=1`; `inspect.native_input_isolated` records it.
This prevents a shared desktop or initial cursor warp from changing fixture poses.
Direct `salimon_test.Game` defaults to `native_input=True` for opt-in OS probes
and removes the variable; use those probes for native input/lifecycle signoff.
Isolation never certifies real input or normal wall-clock interpolation.

## Complete reference-frame logging (#483)

`RUST_LOG=info,salimon_client::frame=trace` opts into every existing frame timing
record (interval/present wall, CPU update/render, asynchronous latest GPU, tick
and backlog). Ordinary debug output remains sampled every300 frames. Use the
same log level/output for both reference variants and preserve warmup/spikes;
logging IO is measurement overhead, never free production performance. Frozen
E2E reference scenes measure presentation without normal-play simulation work.
See the [lighting report](../reports/issue-483/README.md) for exact context and
limits. The logger never changes scheduling, input or GPU polling.

## Scout machinery fixtures (#489)

The opt-in `scout-fabricator` E2E setup begins landed near the physical machine
with starter ingredients and the existing exact 2 kg carried fixture fragment.
`scout-work-blackout` begins with a queued unfunded ration and the Core-restart
held fragment/aim. Both validate their initial save source. Subsequent production
intake/menu/step/collection commands have no test bypass. They never alter a
normal player's save or imply physical mining/travel.
`reports/issue-489/walkthrough.py` captures real GPU surfaces through the existing
window adapter. Named `pedals-close` and `fabricator-close` cameras only change
presentation; the original four Scout lighting/Blender cameras are unchanged.
See [validation and limits](../reports/issue-489/README.md).

## Archive cloud verification (#505)

`--e2e --scenario scout-archive` starts with normal new-run source placed at the
physical Archive; it does not open/read records. `inspect.archive` contains
`source`, `open` and optional rendered `text`. `key: interact` maps to physical E
for opening/closing through production dispatch. Existing `thruster_up/down`,
`roll_left/right` and `wheel_next` map to arrows/Tab and navigate the reader.
Use `reports/issue-505/verify.py --binary target/debug/salimon-client --output
artifacts/new-505 --scale 1` under Xvfb/Vulkan for all seven records at both
resolutions. Scales 1/1.5/2 are simulated via X11, not physical DPI evidence.

## Recovery cloud verification (#515)

`scout-recovery` explicitly starts with a held 2 kg iron identity and a 0.5 kg
silicate bin, no module/job/progress. `key: recovery` maps to physical R through
normal Archive dispatch; `inspect.recovery` is immutable source evidence. Use
`reports/issue-515/verify.py --binary target/debug/salimon-client --output
artifacts/new-515` under Xvfb/Vulkan. It drives intake, menu, funded craft, tray
collection/installation, real cabin walking, R start/stop and both paid records.
This fixture does not measure mining/acquisition, balance or physical OS input.

The #566 collector adds the explicit `--profile cloud-llvmpipe-1280` admission
profile for Xvfb 1280×800, scale1, Vulkan llvmpipe. The default M1/Metal profile
remains strict. Both require100 consecutive focused, non-occluded, zero-tick
frames and unchanged source/context. The X11 helper verifies exact target PID,
focused window, viewable map state and geometry; it does not certify physical
input or scanout. Every repeated frozen capture must have the same PNG hash.
See [matched cloud lighting evidence](../reports/issue-566/README.md) for commands,
all failures, expression-only control lineage and hardware-counter limitations.


## Bounded native capture polling (#568)

`--screenshot-command '["native"]'` requests the actual completed GPU color
surface. The runner first requests real window restoration, then schedules the
capture and polls `{"op":"inspect","scope":"capture"}`. Current executables
return only `capture_status`, avoiding repeated large world/fragment snapshots;
older executables ignore the optional scope and retain compatible full inspection.
All envelopes, timeout limits, E2E gates and actual display/occlusion admission
remain in force. Restoration is an actual logged window operation, never a
synthetic focus flag. GPU images do not certify physical OS-input delivery.

The native marker is never an external executable. After a scenario/protocol
failure the runner retains checkpoint images, original error and last state;
it reports a new native failure screenshot unavailable because a timed-out
response may still be outstanding. It never issues a second request into a
possibly desynchronized channel or hides the original failure with a PATH error.
Contract tests cover bounded polling payload, original PNG validation, the native
marker and failure-state preservation. Actual native corpus captures remain a
separate required test. See [the #568 report](../reports/issue-568/README.md).

Cabin evidence: `python3 scripts/salimon-test run scenarios/evidence/habitation-use.json --binary target/debug/salimon-client --screenshot-command '["native"]'` uses the explicit `scout-cabin` camera setup and production E/X/U commands.

Finish evidence: `room-finishes.json`, `room-finishes-blackout.json`, and `furniture-finish.json` under `scenarios/evidence/` use production V/O/Z input after explicit camera setups. Named diagnostic views in `scenarios/visual/scout-finish-cameras.json` change presentation only. Native inspection reports actual ship resident vertex/uniform bytes and draw/triangle counts.

Core upgrade evidence: `core-upgrade-blackout.json` and `core-upgrade-powered.json` under `scenarios/evidence/` use explicit two-receipt/one-carried-fragment seeds, then production I/E/Y inputs. They prove transaction/presentation behavior, not a mining or walked acquisition route.


Construction flight readiness (#546): protocol key `construction_stop_resume` maps to native T while the C builder is active, using the same reachable project admission. Run `scenarios/evidence/construction-interlocks.json` for paid Stop/Resume/refund and `scenarios/evidence/construction-interlocks-cockpit.json` for cockpit takeoff refusal. See [the canonical contract](../docs/construction-flight-safety.md).


Optional modules (#547): protocol `construction_module` maps to native G in the C builder. Run `scenarios/evidence/construction-module.json`, `construction-module-upgrade.json` and `construction-module-restore.json` for seeded installation, upgrade downtime/Stop/Resume and physical restoration. Each uses real E intake, paid work and commissioning. See [controls and setup limits](../docs/non-core-modules.md).

Persistent surface-age evidence #581: run `scenarios/evidence/surface-age-used.json`
with the normal native capture command (and the matching new/idle/recolor/blackout/
reload/damage/repair routes). The used setup executes 600 actual locker open/close
cycles before the first frame; the other declared seeds and independent renderer
stress workload are documented in [the surface-age contract](../docs/surface-age.md).

## Source review preparation (#537)

```sh
python3 scripts/prepare_source_review.py --output artifacts/source-review
python3 -m unittest discover -s scripts -p 'test_*source*.py'
```

This offline command freezes verified bytes and a deliberately unapproved human
review form without changing production eligibility. Existing output is preserved;
corrupt inputs leave no published partial packet. Use the
[source contract](../docs/universe-sources.md#preparing-the-human-source-review-537)
and [decision checklist](../reports/issue-537/README.md) for current limitations,
provider access, exact source/snapshot approval and physical balance evidence.
