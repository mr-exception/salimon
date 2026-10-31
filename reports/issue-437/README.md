# Cloud flight, collision and cockpit acceptance (#437)

Parent: CI repair PR #714, branch `fix/main-ci-evidence-dependencies`,
`02bdd51dcff85d1405a20e72ea9d4d720d1724fb`; source main
`0a7715f93ad7c2da5b3e7621cb3b7eca9871bf66`.
Explicit prerequisites #417–#421 are closed and their trajectory, surface-assist,
atmosphere, contact and navigation implementations are present on this ancestry.
All 23 open issues and active PRs were enumerated; no active PR existed before #714.
#547 already has its runtime implementation; it is not counted as a new delivery.

## Delivery

A maintained [acceptance matrix](../../docs/universe-flight-verification.md)
combines production Rust contracts, nine native GPU routes and a normal-launch
OS-input smoke. `scripts/verify_universe_flight.py` runs the checks, retains exact
commands/status/times/source hashes/backend context and refuses incomplete route,
assertion, capture or raw-telemetry evidence. Tests protect failure handling and
complete selection. The existing planetary reviewer accepts an explicit route
inventory without changing its default planetary matrix.

A new funded tangential insertion regression checks independent osculating energy/eccentricity, a doubled-mass negative control and an unpowered full-orbit continuation. No physics, resource, model, save or control parameter is changed. Orbital tests
exercise the real-scale production APIs with declared moving ephemerides; they
are **not** evidence of a playable interplanetary orbit. Canonical Vesper and the
seven legacy initial-condition routes are identified separately. Protocol steps
are not physical keyboard events; the separate X11 F2 smoke exercises the actual
normal window/event path. Closed-game trajectory integration remains #432–#434.

## Reproduction

Use Python 3.12.14 with Pillow, Rust/Cargo 1.99.1, the locked dependencies and
Linux native build/graphical dependencies in the validation guide. Set up Xvfb
1280×800, Mesa llvmpipe Vulkan and xdotool. This executor used Mesa 25.0.7,
LLVM 19.1.7 and `LP_NUM_THREADS=2`. Save and settings files are isolated under the
new output directory; no existing player checkpoint is used.

```sh
cargo build --workspace --locked
python3 scripts/verify_universe_flight.py --binary target/debug/salimon-client \
  --output artifacts/issue-437/fresh-run
python3 -m unittest discover -s scripts -p 'test_*.py'
python3 scripts/check_docs.py --base fix/main-ci-evidence-dependencies
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
```

Raw original captures, state/metrics, command logs, hashes and actual results are
recorded with the final evidence index below. GPU counters stay unavailable/null
when not supplied; software timings are not a hardware FPS claim.

## Retained failures and corrections

- First pilot exited before ready: stale/unavailable X server. A live Xvfb in the
  same process session corrected setup. The pilot then passed 13/13 steps; its
  default OS screenshots are supporting evidence only.
- The full driver was restarted to explicitly request `["native"]` GPU captures;
  the final reviewer rejects desktop/CPU captures for route evidence.
- The first full Rust contract attempt exhausted this executor's 7.9 GB disk due
  to generated debug/incremental artifacts. Removing only the generated
  incremental cache recovered 2.4 GB. No source, report or save was removed.
- The initial unmodified build lacked ALSA development files; those were supplied
  for the CI repair and reused here. No device audio verification is claimed.

## Documentation impact and remaining qualification

Updated runtime AI routing, maintenance/validation guides and the new command's
coverage/boundary guide. Ship/runtime architecture and invariants are linked from
that guide; their physics/save/control contracts are unchanged. Same-PR local
link and documentation-impact checks apply. Historical reports retain immutable
references from the parent CI repair.

Physical devices/OS suspend/wake remain unverified under #557; handling/readability
and subjective appearance remain #561; M1/Metal/hardware budgets remain #563.
These existing release-qualification owners retain those criteria. No baseline
promotion, source-rights approval, release, deployment or merge is performed.
The source issue remains open until the delivering PR merges.

## Final results

- `cargo test --workspace --locked`: PASS, 1,244 tests and 51 opt-in ignored at the unchanged production source before the added insertion regression.
- `cargo test -p salimon-ship --locked funded_tangential_burn -- --nocapture`: PASS, exactly one new test; see [log](insertion-test.log.gz).
- Driver ship contracts: PASS, 150 tests / 1 opt-in ignored; session contracts: PASS, 537 tests / 25 opt-in ignored.
- Native matrix: PASS, 9/9 routes, 974/974 steps, 18 direct GPU originals. Normal launch / actual X11 F2 round trip: PASS.
- `python3 -m unittest discover -s scripts -p 'test_*.py'`: PASS, 143 tests. Formatting, all-target workspace Clippy with warnings denied, documentation and base-impact checks: PASS.
- Parent #714 at `02bdd51d`: all three remote source-build checks pass. This child PR's published checks are tracked separately.

[Evidence index](evidence-index.json) records commands, times, environment,
per-capture raw metrics/hashes, all original capture paths and retained paths.
[Original images](captures.md), [unaltered driver result](driver-result.json.gz),
[ship log](ship-contracts.log.gz), [session log](session-contracts.log.gz),
[OS-input log](os-input.log.gz), [workspace log](workspace-tests.log.gz),
[script log](script-tests.log.gz) and [Clippy log](clippy.log.gz) are retained.
[Raw native state/contract/process evidence](raw-native.tar.gz) preserves original
result/scenario JSON and stderr log bytes with member paths relative to the driver's output root. Repeated stdout/protocol copies of the same telemetry are omitted; PNGs are
retained separately without duplication. Original absolute paths in those records
identify the collection workspace; the index maps them to retained artifacts.

The driver started with dirty source on the parent revision. Its recorded source
hashes bind that collection. During collection, only `trajectory/tests.rs` changed:
the new insertion regression was added and separately executed after rustfmt;
its before/after hashes are explicit in the index. No production Rust code or
executable changed. Final published-head CI executes the complete final test set.
The interrupted display/setup and disk failures remain in [attempts](attempts/).
These failures were corrected, not relabeled as passing runs.
