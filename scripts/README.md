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
`timeout_ms`, and ordered `steps`. Setup uses the four native fixtures listed
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

## Phase 0 baseline suite

The default `suite` checks four native gameplay paths:

| Scenario | Coverage |
| --- | --- |
| `landed-earth.json` | Known initial player pose and ship state; walking/aiming into cockpit control; non-pilot control rejection; closed-door exit collision; opening and exiting to the surface; closing the door outside and walking against it twice; reopening and returning inside; the open-door takeoff interlock; closing the door and completing assisted takeoff. |
| `resource-deposits.json` | Real airlock exit and surface walk to a stable generated silicate deposit; checks material, positive mass, proximity, and presentation data. |
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
Its baseline is automatically included in CI's default suite. Extraction totals
are diagnostics only; physical-fragment output is implemented separately in #45.
