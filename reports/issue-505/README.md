# #505 — Archive production interactions and persistence

Third delivery in the requested stack, after [PR #584](https://github.com/mr-exception/salimon/pull/584).
Adds an opt-in initial position at the separate physical Archive and immutable
protocol observations of its source/open/text state. E opens/closes the actual
reader; existing arrow/Tab aliases navigate through production input dispatch.
No record content, knowledge, save schema or gameplay authority changes.

## Results and evidence

The [driver](verify.py) captures all seven intact/damaged records at 1280x800 and
1920x1080, at each simulated X11 scale 1/1.5/2: 42 original record frames.
Physical closed terminals and Escape/pause add nine frames. A separate Core
blackout/intake/refuel route adds four terminal frames at the two resolutions.
Every capture asserts unchanged authoritative source/ship state before/after
GPU readback. Complete originals, state, protocol and per-run metadata are in
[evidence](evidence/); large JSON is losslessly gzip compressed.

Production tests prove E/range/aim/interior/occlusion admission, arrow/Tab
navigation, E close, Escape pause, Q/H/world exclusion, focus/power/range resets,
continued simulation while reading and no invented coordinates/Hall spoilers.
The capture driver checks all seven stable record identities/read counts and
unknown/navigation disclaimer text, modal exclusion and continuing hunger drain.

A new regression runs the real asynchronous **Save Now** request to verified
completion, requires a UTC receipt without ticking simulation, drops the worker,
relaunches through normal `Checkpoints::open_at`, and compares exact read/selection
source while reader stays closed. Verified death/restart resets source and a
second launch confirms the reset is durable. Existing tests preserve old-schema
initialization and malformed/duplicate/unknown source rejection.

[Focused Archive tests](archive-tests-final.log): **14 pass**. Workspace
build/tests, Clippy and formatting results are retained alongside the report.
Python runner tests, syntax, docs/source-impact and whitespace checks pass.

## Reproduction and context

Use the native dependencies and Xvfb/lavapipe setup in the
[preceding engine report](../issue-498/README.md). On this executor source
`/workspace/.salimon-tools/env.sh`; set `DISPLAY=:99`, `WGPU_BACKEND=vulkan`,
`VK_DRIVER_FILES=/workspace/.salimon-tools/sysroot/usr/share/vulkan/icd.d/lvp_icd.json`.

```sh
cargo build --workspace --locked
cargo test -p salimon-client --locked archive
cargo test --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo fmt --all -- --check
python3 reports/issue-505/verify.py --binary target/debug/salimon-client --output artifacts/new-505-scale1 --scale 1
python3 reports/issue-505/verify.py --binary target/debug/salimon-client --output artifacts/new-505-scale1.5 --scale 1.5
python3 reports/issue-505/verify.py --binary target/debug/salimon-client --output artifacts/new-505-scale2 --scale 2
python3 reports/issue-505/verify.py --binary target/debug/salimon-client --output artifacts/new-505-power --power
python3 -m unittest discover -s scripts -p 'test_*.py'
python3 scripts/check_docs.py
```

Capture manifests identify source revision `ee55680` and binary SHA-256. Backend
is Linux/Mesa 25.0.7 llvmpipe / LLVM 19.1.7 software Vulkan, debug profile, Xvfb;
actual window/scale/adapter context is retained per frame. `WINIT_X11_SCALE_FACTOR`
is a simulation, never physical DPI. Requests use 96 ms and production 16 ms
ticks. Desktop controls are isolated; no physical keyboard/OS experiment is claimed.
Placement is an explicit initial fixture, not a walked acquisition route. The
blackout terminal camera is an explicit reference viewpoint; intake still uses
its real aimed character/held-fragment source. No image baseline is promoted.

## Remaining review and documentation impact

Human reading/accessibility, wall-screen orientation, muted/amber text, housings,
all intact/damaged record images and blackout/restoration review remain
[#561](https://github.com/mr-exception/salimon/issues/561). Return per-image
accept/reject with reviewer/date and original hashes. Physical keyboard/DPI and
OS focus/lifecycle qualification remain [#557](https://github.com/mr-exception/salimon/issues/557).
Hardware GPU counters, matched release frame-tail budgets, Metal and hardware FPS
are unavailable/unrun, owned by [#563](https://github.com/mr-exception/salimon/issues/563).
Recovery and authored destination leads remain separate work; no narrative or
navigation knowledge is granted by verification.

Updated runtime README/architecture/invariants, runner usage and validation
specify the new fixture/key alias/immutable observation and test boundary.
Owning [architecture](../../client/runtime/architecture.md) and
[invariants](../../client/runtime/invariants.md) map fixture/observation to
`e2e.rs`, `scout_walkthrough.rs`, `automation.rs`; durable evidence belongs to
`session/archive/tests.rs` and the unchanged [save contract](../../docs/save-contract.md).
The renderer has no new API or resource: the added named Archive camera feeds
its existing scene boundary. Paths/links and source-impact checks are recorded.
