# #422 — Predictive spatial streaming

Base: main `71dd3632` (merged U12–U19 ancestry). Implements a bounded
background catalog/system/body streaming service and native frame polling via
`SALIMON_UNIVERSE_PACK`. No IO or mutex waits occur in frame polling. Request
generations reject stale results after traversal, quality changes do not mutate
canonical bytes, and required data errors fail explicitly in the client.

`client/universe/src/streaming.rs` owns planning/worker/residency;
`streaming/tests.rs` exercises the real pack builder, filesystem reader and
worker. Runtime opens the service before the event loop and polls after camera
composition. The [contract](../../docs/universe-streaming.md), owning guides and
root routing describe budgets and failure semantics. Source checksums/schema
limits are unchanged. No renderer/GPU allocation occurs here; terrain streaming
is the next requested issue #424, gameplay consumption #425/#426.

## Validation context

2026-10-27, Linux x86_64, 2 CPU / 8 GiB cgroup, Rust 1.99.1.
The host has no package-install privilege. Rust installed successfully on the
host; Linux client dependencies were installed in the local `rust:1-bookworm`
Docker container `salimon-dev`. Initial overlapping container rustup setup failed;
the completed host toolchain was copied to `/opt/rust` and used directly, with
TLS verification and the session proxy/CA retained. These setup failures are not
passing checks. Compilation uses two jobs.

Commands inside the container use
`PATH=/opt/rust/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin`.
Exact final checks/results are recorded below before publication.

## Limits and ownership

Encoded cache bytes are capacity-accounted independently from manifest/index,
bounded decoded active cells and one in-flight catalog/decode scratch. Default
active limit 16, maximum 64; at most one worker request/result is outstanding.
Startup/exit can wait for current bounded filesystem IO; no disk-latency deadline
is implied. Catalog truth never grants player knowledge. Full-data scientific
coverage/resource qualification remains #524/#526; public release review #659.
M1/Metal/device/hardware FPS is unrun and belongs to #557/#563; no graphical
appearance changed in this PR. No pending implementation criterion is transferred
into those human reviews.

## Final checks

- `cargo build --workspace --locked -j 2`: PASS; final rebuilt native binary.
- `cargo test --workspace --locked -j 2`: PASS, 1,090 passed, zero failed,
  38 pre-existing opt-in ignored (including doc tests). See workspace-tests.log.
- `cargo test -p salimon-universe --locked streaming -- --nocapture`: PASS,
  four filesystem/worker regressions on final source. 10,000 polls / 32 sparse
  cells: median 13.139 us, p95 48.904 us, max 41,933.045 us while other checks
  were active. This includes scheduling delays and is not a quiet frame-time
  benchmark or a claim that every poll meets a wall-clock deadline.
- `cargo clippy --workspace --all-targets --locked -j 2 -- -D warnings`: PASS.
- `cargo fmt --all -- --check`, `python3 scripts/check_docs.py`,
  `git diff --check`: PASS.
- `python3 -m unittest discover -s scripts -p 'test_*.py'`: 134 run, one runner
  startup timeout under compilation. Isolated `test_salimon_test.py` rerun:
  all 22 PASS; original failed run and rerun retained, no script change.
- Production native wgpu/Vulkan streaming: 4/4 PASS; 1 verified blob read,
  1 active/wanted cell, 4,096 resident encoded bytes, no worker error.
  [Actual observation](native-streaming.json). Native command:

```sh
target/debug/examples/pack_bench artifacts/streaming-422-pack
SALIMON_UNIVERSE_PACK=/repo/artifacts/streaming-422-pack/1 \
WGPU_BACKEND=vulkan VK_DRIVER_FILES=/usr/share/vulkan/icd.d/lvp_icd.x86_64.json \
LP_NUM_THREADS=2 XDG_RUNTIME_DIR=/tmp/salimon-xdg \
xvfb-run -a -s '-screen 0 1280x800x24' python3 scripts/salimon-test \
run scenarios/evidence/universe-streaming.json --binary target/debug/salimon-client
```

Initial native attempts failed for missing Xlib/XKB libraries, then for the
ordinary five-second debug-first-frame step deadline during concurrent checks.
Installed the missing dependencies and used an explicit streaming evidence route
with startup allowance; no gameplay/physics or correctness tolerance was relaxed.
The final evidence route observes the production worker, not a substituted reader.
