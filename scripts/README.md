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
protocol: `inspect`, `step`, `key`, `look`, `interact`, `landing`, or `thruster`.
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

Before resource transfer/cargo work, run the four prerequisite routes together:

```sh
python3 scripts/salimon-test suite --group ship-eva
python3 scripts/salimon-test suite --group ship-eva --evidence \
  --screenshot-command '["python3", "scripts/capture_settled.py", "{path}"]'
```

Both commands support `--binary` and `--artifacts`. The first runs cargo-room,
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
acceleration. Cargo traversal and airlock collision checks cover the real layout.
The required Linux CI job runs this evidence group against the release binary.
See [checked-in validation](../docs/issue-39/README.md) for results and screenshots.

## Phase 0 baseline suite

The default `suite` checks seven native gameplay paths:

| Scenario | Coverage |
| --- | --- |
| `cargo-room.json` | Exit/re-enter the actual aft airlock, traverse the new port cargo passage, test outer walls and the solid partition, return to door/cockpit, take off and walk the room again in flight. |
| `landed-earth.json` | Known initial player pose and ship state; walking/aiming into cockpit control; non-pilot control rejection; closed-door exit collision; opening and exiting to the surface; closing the door outside and walking against it twice; reopening and returning inside; the open-door takeoff interlock; closing the door and completing assisted takeoff. |
| `resource-deposits.json` | Real airlock exit and surface walk to a stable generated silicate deposit; checks material, positive mass, proximity, and presentation data. |
| `resource-streaming.json` | Partial mining, walking beyond the 120 m active radius, explicit source-ID absence, return with identical mass, full depletion, and a second round trip without regeneration or duplicate fragments. |
| `carrying.json` | Physical pickup, blocked second pickup, equipped-tool independence, occupied/clear placement, subsequent pickup, and entity proximity after walking. |
| `mining.json` | Surface tool equip/aim/hold, exact timed extraction, aim/range/release rejection, return to the deposit, bounded depletion, and stow; no inventory credit. |
| `orbit-earth.json` | Known seeded orbit pose; thruster changes; starting assisted landing; repeated landing action cannot cancel it; leaving cockpit control during landing; autonomous completion with zero ship velocity. |

These files use only the existing `key`, `look`, `interact`, `thruster`,
`landing`, mining equip/hold keys, and fixed `step` operations after fixture setup. They never set
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

Every push to `main` and every pull request runs the deterministic baseline
`suite` against the staged **release** executable on Ubuntu 24.04 in
`.github/workflows/native-build.yml`. This required Linux check covers launch,
seeded readiness, the cockpit/door/surface/assisted-takeoff path and the
orbit/assisted-landing path. It uses a dedicated 1280×800 Xvfb display and
discovers the installed Mesa lavapipe ICD filename and selects it via
`VK_DRIVER_FILES`; software Vulkan is the
CI fallback, not a hardware performance or native graphics fidelity check.
Display, Vulkan, and screenshot-helper failures fail the job. A capture
preflight saves `display-ready.png`, and automatic failure screenshots remain
enabled throughout the suite.

The runner's nonzero exit status propagates through `tee` using Bash
`pipefail`. `baseline.log` contains the aggregate report, including the failed
scenario/step reason. The always-run artifact upload preserves all
`artifacts/e2e/` results, per-step state, protocol/process logs, failure state
and available failure screenshots in `salimon-ubuntu-24.04`, alongside runnable
builds and packaged smoke evidence. Each native scenario has a bounded timeout;
the CI step also has a ten-minute limit. Inspect `result.json` for the first
failed step before checking its logs and screenshots.

The fast required baseline has no successful-run screenshot checkpoints. For
the slower visual suite, manually dispatch **Native builds** with
`visual_evidence` enabled. It runs both `scenarios/evidence/*.json` paths and
uploads named screenshots plus `visual.log`; capture failures fail that step.
The separate required packaged smoke still checks normal startup and real OS
keyboard input. Both Linux checks use software Vulkan.

macOS and Windows CI run build, Rust, and applicable Python contract checks;
graphical E2E is explicitly not run there and this is recorded in the job
summary. Validate those platforms with local baseline and evidence runs on
their native GPU desktops. No successful Linux check claims macOS/Windows
graphics coverage.

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

The Linux CI baseline step also requires the matching resource-deposit visual
scenario, saving a `generated-silicate-deposit` screenshot and authoritative
state within 4 m of the deposit. A screenshot failure fails CI. This uses the
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
Its baseline is automatically included in CI's default suite. Both variants
assert physical fragment identity, material, mass, volume, pose, and bounded
output after depletion. Extraction totals and session fragment totals are
diagnostics only, never inventory. The partial/depleted screenshots show the
physical cubes left beside the deposit.
Linux's required baseline job runs this mining evidence variant; capture errors
fail the job. Checked-in [issue #45 evidence](../docs/issue-45/README.md) records
the initial Linux validation.

## Local streamed resource state (#50)

`scenarios/resource-streaming.json` uses the same real mining route, releases
mining input, walks 153.6 m away and back, and checks the source is absent from
`world.deposits_by_id` while away. It verifies partial mass restoration, then
repeats after depletion and verifies zero mass, no visual/target, and unchanged
physical output totals. The baseline is included in the default suite; Linux CI
also runs `scenarios/evidence/resource-streaming.json` with named screenshots.
No teleportation or test-only state mutation is used. Local session state is
retained only until the world/session ends, without disk or backend persistence.

## Physical pickup/drop (#46)

`scenarios/carrying.json` follows the real mining route, approaches and aims at
physical pieces, and uses `pickup`/`drop` keys (Q/G). It verifies single-object
rejection, material/mass/identity preservation, tool independence, placement
rejection on occupied ground, release, subsequent pickup, and carried entity
proximity after movement. The evidence variant adds first pickup, blocked second
pickup, placement, and moving carry screenshots. Both run in required Linux CI.
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

## Cargo-room walkthrough (#34)

The default suite includes `cargo-room.json`. Its evidence variant is required
in Linux CI and records exterior/cockpit plus landed/flying cargo views. Run:

```sh
python scripts/salimon-test run scenarios/evidence/cargo-room.json \
  --screenshot-command '["python", "scripts/capture_settled.py", "{path}"]'
```

The helper waits 0.5 seconds before the normal platform screenshot command to
reduce stale frames from asynchronous GPU presentation. It is not a GPU fence;
authoritative movement/state assertions remain the verification source. Capture
errors still fail the scenario and the runner's normal deadlines apply. Use a
dedicated unobscured test display. Checked-in validation images and results are
in [issue #34 evidence](../docs/issue-34/README.md).

## Space airlock access (#35)

`open-space` initializes a stationary flying ship 10 km beyond the shared
3,000,000 m nearby-body surface-distance threshold. `space-airlock.json` uses
normal controls to leave the cockpit, walk to the gate, open it, exit without
planetary snapping, close it from outside, verify blocked re-entry, reopen and
re-enter, close it again, and verify blocked exit. It runs in the default suite.
`scenarios/evidence/space-airlock.json` adds four screenshot/state checkpoints and
is required in Linux CI. Use the settled capture helper as for the cargo room.

The portable runtime test executes the same access scenario, while ship tests
cover inclusive threshold locking and assisted-sequence locking. Moving-ship
velocity inheritance and free 3D EVA are #37, and nearby-body transitions #36.

## Moving-ship EVA (#37)

`moving-eva.json` sets one thruster increment through normal cockpit controls,
then follows the airlock route at 25,000 m/s. It verifies inherited world velocity,
no-input ship-relative stability over 600 fixed frames, Space/Shift vertical
translation, pitched/yawed view-relative translation, assisted stop, closed-gate
collision, and clean interior re-entry. The baseline runs in the default suite;
the synchronized evidence variant adds drift, vertical-flight and re-entry
screenshots and runs in required Linux CI. Portable runtime tests execute both
stationary and moving routes, and character contracts compare 10/20/100 ms
updates and verify detached motion/view independence from ship changes.

## Nearby-body EVA (#36)

`nearby-eva.json` uses the `eva-approach` initial fixture and normal gameplay
controls to leave a moving ship, stay in open-space mode outside influence, then
cross Earth's inclusive shared surface-distance threshold. Assertions check the
player-selected body, nearby movement mode, continuous altitude, inherited
25,000 m/s motion, and bounded radial acceleration. The baseline runs in the
default suite; its evidence variant adds three screenshot/state checkpoints and
runs in required Linux CI. Character contracts additionally check boundary mode
changes, gravity integration across update sizes, influence exit, and contact.

## Planet/ship fragment transfer (#47)

`fragment-transfer.json` uses the real mining/pickup route, carries one fragment
through the open gate, drops/retrieves it inside, carries it back outside, then
repeats. It checks identity/mass, ownership, world versus ship support, re-entry,
and stable ship-local placement while walking away/back and during assisted
takeoff and 25,000 m/s flight. The portable runtime executes the same route,
checking carried proximity after each input/step. Narrow contracts cover rotated
and translated ship frames, pickup detachment, invalid placement, and sight
obstruction. The default suite includes the baseline; Linux CI also requires
the evidence variant with settled capture:

```sh
python scripts/salimon-test run scenarios/evidence/fragment-transfer.json \
  --screenshot-command '["python", "scripts/capture_settled.py", "{path}"]'
```

Inspection exposes each visible fragment's `reference_frame` (`ship` for loose
interior anchors, `world` otherwise) alongside its existing carried flag and
ship-local pose. These fields describe actual physical entities, not inventory.

## Physical cargo containment (#49)

`physical-cargo.json` mines a generated deposit and stores two real fragments
across separate trips through the cargo passage. It verifies the one-object
limit, room leave/re-entry, removal to the surface, and stable cargo through
assisted takeoff and flight. `cargo.fragments` lists only actual loose fragments
whose complete conservative bound fits within the generated cargo-room bounds;
it is independent of player distance and excludes carried and cabin/surface
objects. `cargo.fragment_count` derives from that list, never an inventory counter.

The baseline runs in the default suite and its complete action/assertion route
also runs in a portable runtime regression. The evidence variant adds five named
captures and runs in required Linux CI with settled capture. Capture failures fail
that run. See [validation status](../docs/issue-49/README.md) for the passing native CI results, checkpoint states, screenshots and reproduction
commands.

## Lower cockpit windows (#38)

`lower-cockpit-windows.json` starts in the deterministic Earth approach and uses
normal free-look to inspect the lower pane at 30 degrees down / 34 degrees to
each side of the console. It starts assisted landing, checks the surface distance
falls below 25 m, waits for touchdown, inspects both sides, restores the forward
view, takes off and leaves cockpit control. Its evidence variant records eight
native screenshots, including the seated low-altitude surface view. The default
suite runs the baseline; Linux CI requires the settled-capture evidence variant.

```sh
python scripts/salimon-test run scenarios/evidence/lower-cockpit-windows.json \
  --screenshot-command '["python", "scripts/capture_settled.py", "{path}"]'
```

The authored pilot view begins at -0.10 radians; mouse deltas in this route
include that offset. Asset validation independently checks actual exported
triangle sightlines. Passing gameplay assertions alone does not establish visual
correctness; inspect the native checkpoints as well.
