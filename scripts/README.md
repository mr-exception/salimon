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

The opt-in `scenarios/evidence/landed-earth.json` example demonstrates state and
visual checkpoints without imposing screenshot dependencies on the default
suite. Existing suite scenarios still emit logs/results and automatic failure
captures. Use it on an unobscured native display or a dedicated Xvfb display with
a compatible Vulkan implementation:

```sh
xvfb-run -a -s '-screen 0 1280x800x24' scripts/salimon-test \
  run scenarios/evidence/landed-earth.json --binary target/debug/salimon-client
```
