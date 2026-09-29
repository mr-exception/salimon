# Native E2E scenario runner

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
