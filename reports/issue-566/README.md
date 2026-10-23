# #566 — Matched Scout lighting cloud evidence

This delivery extends the report-side collector with an explicit cloud
admission profile and an X11 target-window helper. The default M1/Metal profile
retains its strict context. No game shader, scheduling or simulation rule is
changed in this PR. Current/control measurements and final results are appended
after serial collection below.

## Matched workload and controls

Both release applications use the same stacked Scout asset, camera JSON,
1280×800 viewport, scale 1, frozen seed 0/core-restart state, power transition and
logging. Four named views × two power states × three repeated six-second windows
follow 0.75-second warmups. Each accepted window must contain at least 100
consecutive focused, non-occluded frames, zero simulation ticks and unchanged
authoritative source/context. X11 admission independently verifies exact target
PID/window, IsViewable map state and 1280×800 geometry. Captures come from the
actual completed GPU color surface, not reconstructed pixels.

The diagnostic control applies the retained [expression patch](../issue-483/evidence/control.patch)
to `ship_mesh.wgsl` in an isolated source/target checkout. It replaces only the
pre-#463 fill/emission expressions within current interfaces. It is **not an
exact historical whole-game build**. Application source/input hashes establish
all other inputs match. Control lighting is not installed in the stack.

Raw CPU update/render, submitted-present interval/wall, asynchronous latest
device duration, backlog and every spike remain retained, including warmup logs.
Software-device timestamps, if available, are not hardware GPU counters. Frozen
fixture intervals are not normal-play FPS, optical scanout or a hardware budget.
Actual hardware counters/budgets remain #563; physical platform/input #557;
original-image mood/projection/readability/baseline review #561. No candidate,
mask or tolerance is promoted. Cycles remains a separate mood reference.

## Commands and admission checks

Run from the repository root with Rust 1.90.0 and locked dependencies. Current
uses a freshly created `CARGO_TARGET_DIR=/workspace/salimon/target-stack`;
control uses an initially empty `artifacts/issue-566/control-target`, with a
detached source checkout at `e55d96a` plus the expression patch only.

```sh
git worktree add --detach /workspace/salimon-566-control e55d96a3332e92f48009a1afb974f8750fa63ce2
git -C /workspace/salimon-566-control apply /workspace/salimon/reports/issue-483/evidence/control.patch
CARGO_TARGET_DIR=/workspace/salimon/target-stack cargo build --release --workspace --locked
CARGO_TARGET_DIR=/workspace/salimon/artifacts/issue-566/control-target cargo build --release --manifest-path /workspace/salimon-566-control/Cargo.toml -p salimon-client --locked
python -m unittest discover -s reports/issue-483 -p 'test_*.py' -v
python -m unittest discover -s scripts -p 'test_*.py'
```

Collector environment: `DISPLAY=:101`, `WGPU_BACKEND=vulkan`,
`VK_DRIVER_FILES=/workspace/tools/lvp.json`,
`SALIMON_XDOTOOL=/workspace/tools/linux/usr/bin/xdotool`,
`SALIMON_XDO_LIBDIR=/workspace/tools/linux/usr/lib/x86_64-linux-gnu`.
Each lane uses its own initially empty `XDG_CACHE_HOME` under artifacts/issue-566.
Do not set `SALIMON_REFERENCE_SIZE` for the cloud profile: that older switch
forces 1920×1080. The collector removes it for this profile; Metal retains it.

```sh
XDG_CACHE_HOME=/workspace/salimon/artifacts/issue-566/cache-control-final python reports/issue-483/frame_reference.py --profile cloud-llvmpipe-1280 --sample-seconds 6 --binary /workspace/tools/salimon-566-control --helper reports/issue-566/x11_focus.py --output artifacts/issue-566/control-final
XDG_CACHE_HOME=/workspace/salimon/artifacts/issue-566/cache-current-final python reports/issue-483/frame_reference.py --profile cloud-llvmpipe-1280 --sample-seconds 6 --binary /workspace/tools/salimon-566-current --helper reports/issue-566/x11_focus.py --output artifacts/issue-566/current-final
```

The two executables were copied unchanged from their release targets into
`/workspace/tools/salimon-566-control` and `salimon-566-current`; the hashes
in environment.json identify those exact staged binaries.

## Retained failed attempts

The first cloud display smoke probe rejected a 1920×1080 window on the 1280×800
display. The explicit cloud sizing now uses the normal E2E viewport; no geometry
or focus check was bypassed. The second probe verifies exact PID/focus, viewable
state and native GPU-color capture. Both probes used the earlier #542 binary
while builds ran concurrently and are **admission/capture evidence only**, with
no timing claim. Their original logs/context/images remain separate from final
current/control lanes.

The first 118-script-test run overlapped builds and failed the existing 100ms
invalid-PNG subprocess fixture on timeout before it wrote its expected bad PNG.
Its raw failure remains; the serial rerun is recorded below without changing
the fixture's deadline. Six parser/admission tests pass, including strict Metal
rejection of cloud context, unknown profile, lost focus, geometry/adapter/context
changes, isolation and unexpected ticks.

The two historical M1 locked-desktop attempts and zero accepted M1 timing
windows remain [failed in #483](../issue-483/README.md); Linux admission cannot
convert those into passes.

## Projection and documentation

Native `scout_reference.rs` and Blender `preview_lighting.py` read the same
`scout-cameras.json` positions/targets: native ship-pose transform versus inverse
glTF mapping `(x,-z,y)`, 70-degree vertical FOV and 0.05m near clip. The native
shared-camera unit test passes in the workspace suite. This source inspection
checks shared projection intent, not Cycles/native PBR or human mood agreement;
existing Blender 4.5.3 originals remain separate dated, pre-#538 mood references, without a new offline-render or new-layout appearance claim.

Updated canonical Scout presentation, report-side logging usage/runtime guide,
and #463/#483 delivery reports. The parser/collector/helper and tests own this
verification change; application rendering and Universe U01–U34 behavior have
no implementation changes here.

The initial three-second timing matrix admitted three cockpit windows, then
rejected the Core view with 90 frames. Its failure record and all admitted/short
windows are retained separately. Both final lanes use six-second windows;
the 100-frame minimum and every other admission check remain unchanged.

## Final results

**PASS: 48 admitted windows** (24 per variant), all with at least100 consecutive
focused/non-occluded zero-tick frames. Cross-variant authoritative source states,
including construction observations, match. All three frozen repeats and the
canonical image match byte for byte in each view/power state. All 25 independent
X11 focus records per lane show the exact PID/window, viewable state and complete
1280×800 window at 0,0. [Comparison and p50/p95/p99/max distributions](comparison.json)
and [environment/binary/source hashes](environment.json) retain the full context.
[Application input hashes](application-input-hashes.json.gz) prove the only
compiled-input difference is `ship_mesh.wgsl`; [actual control patch](control-source.patch)
retains that exact difference. Native originals/protocol/windows/focus and raw
frame logs are in [control](control/) and [current](current/), large text losslessly
gzip-compressed. Each lane retains 32 original PNGs. The first short matrix is
retained in [short-window-attempt](short-window-attempt/); admission probes remain
[failed](admission-first/) and [passed](admission-final/) separately.

| View / power | Frames control/current | CPU render p95 ms control/current | Interval p95 ms control/current | Interval max ms control/current |
| --- | ---: | ---: | ---: | ---: |
| aft-door / off | 809/821 | 0.310/0.317 | 43.195/42.046 | 250.000/52.518 |
| aft-door / on | 791/808 | 0.317/0.338 | 42.975/42.569 | 72.343/52.838 |
| cockpit / off | 984/1015 | 0.386/0.355 | 36.319/36.207 | 157.807/64.851 |
| cockpit / on | 987/998 | 0.372/0.342 | 36.047/35.488 | 72.189/59.015 |
| core / off | 510/502 | 0.328/0.350 | 53.356/55.153 | 63.651/202.957 |
| core / on | 527/551 | 0.364/0.305 | 52.082/50.261 | 98.387/59.167 |
| fabricator / off | 566/563 | 0.329/0.277 | 51.852/51.064 | 60.764/60.688 |
| fabricator / on | 532/561 | 0.371/0.309 | 53.527/51.600 | 72.999/63.562 |

The smallest admitted window contains 155 frames. All spikes remain: e.g. the control aft-door/off asynchronous latest **software-device** duration reaches 288.117ms, and current Core/off reaches 192.387ms. Those raw values are not hardware GPU counters or attributed shader regressions. CPU render instrumentation excludes asynchronous device work. No aggregate speedup, 60FPS or normal-play cadence conclusion follows from this single serial frozen-fixture comparison.

Final checks: **895 release workspace tests pass / 32 opt-in ignored**, Clippy all
targets, debug/release workspace builds and formatting pass (fresh isolated
source build; logs in #559). **118 script tests pass** on the serial rerun and
**six admission/parser tests pass**; logs, the initial flaky timeout and both
clean build records are in [validation](validation/). Actual native captures
validate both shader variants. `analyze.py` independently rechecks all windows,
original PNG dimensions/hashes, focus geometry, cross-variant source equality
and exact repeat/canonical hashes. No tolerance or test deadline is weakened.

Inspect the original [Core blackout](current/run-3zwdaa2p/core-False.png) and
[Core powered](current/run-3zwdaa2p/core-True.png) surfaces: the screen atlas
blanks/restores and bus-powered fill/emission changes. This observation is not
human mood/readability approval. All 1280 originals go to #561; actual hardware
GPU/cadence budgets remain #563 and physical-platform/input checks #557. The
historical M1 failures remain unchanged. No new 1920 capture, Blender render,
physical display or normal-play timing is claimed here.
