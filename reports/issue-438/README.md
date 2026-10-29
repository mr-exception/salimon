# Issue #438 — Scanner cloud integration acceptance

Delivered a required `scanner` evidence suite, exact scanner service funding and
modal/view regressions, a native capture reviewer and a production placement
fix found by the 2x matrix. A tall bottom toolbar/status stack covered the
selected hologram contact at 1280x800 / simulated 2x scale. Fresh scanner status
now uses the existing action bar at top-right for 1280+ physical pixels, leaving
the projector visible. Compact windows and chart mode retain bottom placement.
Renderer owns generic `TopRight`; runtime chooses from actual viewport width.
No second overlay, simulation, input, collider or saved authority was added.

Parent is [PR #699](https://github.com/mr-exception/salimon/pull/699), branch
`issue-503-scanner-readability`, exact revision
`27a8e08fc00e1bbcdfd38c4bc089032f5a74a042`. #429/#430/#431 implementations are
merged and present on this ancestry. Merge #699 first, then this PR. The parent
head's Source build (Ubuntu) and macOS/Windows fiction publication golden checks
passed in [run 40855640292](https://github.com/mr-exception/salimon/actions/runs/40855640292).
No merge was performed.

## Behavior, source and acceptance matrix

| Requirement | Production path and evidence |
| --- | --- |
| Five-second cadence | Native route has no contacts at 4,992 ms; first observation at 5,008 ms (16 ms fixed tick). Exact service test separately checks 4,999/5,000 ms and the next pulse. No certified extrapolation/backdating is invented. |
| Charge and power | Actual scanner service pays exactly 4,999 J before the boundary, 10,001 J at the boundary and 15,000 J for the next five seconds under configured 1 kW idle/10 kJ pulse. Insufficient funding exhausts the 500 J fixture without refreshing history. Native route exhausts the real Core through launch reconciliation and suppresses projection. |
| TTL/history/restore | Existing production save tests verify three-hour expiry, repeat launch/no replay, rediscovery and atomic malformed/schema/catalog rejection. Native zero-elapsed save/launch preserves contact timestamp/clock/history and resets only transient selection; one-day/five-day launch checks expire contacts without inventing detections. |
| Input/modal isolation | Production dispatch tests cover pause, Archive, crafting, Q, exterior, precision tour and terminal. Native Q/pause routes cannot cycle selection; normal cockpit M can. Input is the protocol-to-native/session adapter, not a physical keyboard claim. |
| Cockpit collision | Existing 31-step `cockpit-nose` evidence route walks the actual controller/cabin collision boundary. Its compressed `landed-earth` setup is explicit; canonical scanner routes are separate. Projector geometry adds no authoritative collider. |
| Readability/layout | Production wgpu engineering-camera captures at 640x360, 1280x800 and 1920x1080, plus observed simulated 1.5x/2x scales at 1280x800. Pixel review requires an unobscured selected marker in the fixed camera-relative ROI. The wide cockpit camera has native source/dimension checks only; it uses another projection. |
| Temporal stability | Fresh and repeated original GPU captures must have identical RGB dimensions/bytes at a frozen simulation time. This is a deterministic render check, not normal-play frame-pacing qualification. |

`scanner/integration_tests.rs`, the existing scanner source/save/presentation
regressions, `scenarios/evidence/scanner-integration.json`, the named suite in
`scenario_suites.py` and `review_scanner_captures.py` map to the
[maintained scanner contract](../../docs/universe-scanner.md#cloud-scanner-acceptance-438).
Owned architecture/invariants, Scout presentation, maintenance map, technical
architecture, scripts README and validation commands are updated together.

## Reproduction and environment

Cloud Debian 13 x86_64, Python 3.12.14, Rust 1.99.1, Pillow 13.0.0 for local image
review (the repository's Pillow 12.3.0 requirements also support the used APIs).
Xvfb :77/:78 are 1920x1080 virtual desktops; actual native capture dimensions and
scale are recorded per frame. Vulkan adapter is llvmpipe / LLVM 19.1.7, Mesa
25.0.7, two llvmpipe threads. Development libraries and graphics packages are
workspace-local as described in the [parent report](../issue-503/README.md).

Run these as single shell commands with the documented X display/Vulkan ICD and
library environment:

```sh
cargo build --workspace --locked
cargo test --workspace --locked
cargo test -p salimon-client --locked scanner
cargo test -p salimon-renderer --locked overlay
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo fmt --all -- --check
python3 -m unittest discover -s scripts -p 'test_*.py'
python3 scripts/check_docs.py --base issue-503-scanner-readability
python3 scripts/salimon-test suite --group scanner --binary target/debug/salimon-client --artifacts artifacts/issue-438-final-1x --screenshot-command '["native"]' --summary
python3 scripts/review_scanner_captures.py RESULT_JSON ... --output artifacts/scanner-image-review.json
```

For scale repeats, copy `scanner-hologram.json` into a disposable scenario with
one `identified-repeat` screenshot immediately after `identified-local-contact`,
then run it in separate processes with `WINIT_X11_SCALE_FACTOR=1.5` and `=2`.
The exact generated scenario is retained with each run. Verify the observed
`visual_context`, not just the requested environment. Native capture uses
`screenshot_command=['native']`; OS desktop screenshots fail the reviewer.

## Final verification

- `cargo build --workspace --locked`: passed.
- `cargo test --workspace --locked`: **1,195 passed, zero failed, 46 existing opt-in ignored**.
- `cargo test -p salimon-client --locked scanner`: **11 passed**.
- `cargo test -p salimon-renderer --locked overlay`: **11 passed**.
- All-target workspace Clippy with warnings denied and fmt: passed.
- All script contracts: **136 passed**, including evidence-only suite selection and negative capture-integrity cases (changed repeat, OS provenance, wrong dimensions, inconsistent power, failed run and missing image).
- Parent-relative docs/base-impact validation: zero failures; `git diff --check` passed.
- Final serial named native suite on :78: **140/140 passed** (81 integration, 28 hologram, 31 cockpit/collision); [summary](native-suite-summary.json), original scenario/result/protocol/stderr/images in the `suite-*` folders.
- Separate observed 1.5x/2x routes on :77: **29/29 each**, 58/58 total.
- Independent original GPU image review: **passed**; exact frozen RGB repeat at each scale. 640x360/1280x800/1920x1080 are checked against their actual recorded capture dimensions. Fresh selected-marker counts are 50–496; unknown/offline counts are zero. The independent :78 integration reproduction also passed 81/81 before the complete suite rerun.
- Parent PR #699's current `27a8e08f` checks are green. This child PR's remote checks are tracked at its published head separately; local results are not a fabricated remote CI pass.

## Original evidence

- [640x360 compact scanner](scale-1/step-062-compact.png)
- [1920x1080 projector and top-right status](scale-1/step-053-wide.png)
- [1280x800 fresh contact](scale-1/step-019-fresh.png) and [exact repeat](scale-1/step-020-fresh-repeat.png)
- [Powered, undiscovered idle](scale-1/step-011-idle.png) and [real blackout](scale-1/step-081-blackout.png)
- [Simulated 1.5x](scale-1_5/step-015-identified-local-contact.png) and [simulated 2x correction](scale-2/step-015-identified-local-contact.png)
- [Capture manifest/hashes](capture-manifest.json), [final image review](image-review.json), [initial failed review](preliminary-failed-image-review.json)

The compact viewport has less unobstructed projector area; its complete readout
fits and the selected marker remains visible (50 measured amber pixels). Wide
and DPI views retain 275–496 selected pixels. No physical DPI claim is made.

The capture reviewer checks native source, actual dimensions, at least five
selected amber pixels for a powered fresh contact (zero when unknown/offline)
in the declared scanner-camera ROI and **exact** frozen-frame repeats. This is a
visibility invariant, not a promoted image baseline or aesthetic approval.
The report retains image hashes, visual context and compressed original
scenario/result/protocol/stderr and command logs.

## Corrected failures and qualification boundaries

The initial 2x route passed its gameplay assertions but the independent pixel
review failed: the selected contact was hidden by the status panel. The
[original failed 2x image](failed-2x-obscured.png) and preliminary result/review
are retained. The placement fix corrects this production layout, then all native
matrix routes and the pixel/repeat checks are rerun. No threshold was widened.
The review initially applied the close-camera ROI to the deliberately wider
cockpit camera; it now records source/dimensions for that separately named view.
A Python suite-selection regression initially expected every new suite in the
baseline directory; corrected the explicit evidence-only scanner suite rule and
reran all script contracts. Initial compile/lint checks during test authoring
were corrected without disabling gates. The first docs check ran before this
report existed and correctly rejected the then-missing target.

A concurrent-window run on the shared :77 display timed out during the initial
terrain wait (`step 4: timeout waiting for response to step`, three completed
steps). Its recorded window had lost focus while the independent DPI processes
ran. That failed result is retained as `contended-display-result.json.gz`; it is
not a pass or classified as flaky. The same unchanged scenario passed 81/81 on
a dedicated :78 display in 51.5 seconds; the complete named suite is rerun there
serially, without concurrent windows on that display.

Offline evidence exercises the implemented quiet launch adapter; nonlinear
trajectory, global-frame adoption and offline detection remain #427/#432/#433.
Neither unit tests nor the native route implement those remaining tasks.
Physical keys/DPI/OS behavior remain #557, subjective hologram/text accessibility
and art review #561, actual hardware/M1/Metal budgets and frame pacing #563.
These qualifications remain **unverified**. Software Vulkan timings are not
hardware FPS. No native-device, human approval or asset/baseline promotion claim
is made, and the source issue stays open until reviewed PR merge.
