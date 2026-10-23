# #566 — Matched Scout lighting cloud evidence

This delivery extends the report-side collector with an explicit cloud
admission profile and an X11 target-window helper. The default M1/Metal profile
retains its strict context. No game shader, scheduling or simulation rule is
changed in this PR. Current/control measurements and final results are appended
after serial collection below.

## Matched workload and controls

Both release applications use the same stacked Scout asset, camera JSON,
1280×800 viewport, scale1, frozen seed0/core-restart state, power transition and
logging. Four named views × two power states × three repeated3-second windows
follow0.75-second warmups. Each accepted window must contain at least100
consecutive focused, non-occluded frames, zero simulation ticks and unchanged
authoritative source/context. X11 admission independently verifies exact target
PID/window, IsViewable map state and1280×800 geometry. Captures come from the
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

Run from the repository root with Rust1.90.0 and locked dependencies. Current
uses a freshly created `CARGO_TARGET_DIR=/workspace/salimon/target-stack`;
control uses an initially empty `artifacts/issue-566/control-target`, with a
detached source checkout at `e55d96a` plus the expression patch only.

```sh
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
forces1920×1080. The collector removes it for this profile; Metal retains it.

```sh
python reports/issue-483/frame_reference.py --profile cloud-llvmpipe-1280 --binary /absolute/control/salimon-client --helper reports/issue-566/x11_focus.py --output artifacts/issue-566/control
python reports/issue-483/frame_reference.py --profile cloud-llvmpipe-1280 --binary /absolute/current/salimon-client --helper reports/issue-566/x11_focus.py --output artifacts/issue-566/current
```

## Retained failed attempts

The first cloud display smoke probe rejected a1920×1080 window on the1280×800
display. The explicit cloud sizing now uses the normal E2E viewport; no geometry
or focus check was bypassed. The second probe verifies exact PID/focus, viewable
state and native GPU-color capture. Both probes used the earlier #542 binary
while builds ran concurrently and are **admission/capture evidence only**, with
no timing claim. Their original logs/context/images remain separate from final
current/control lanes.

The first118-script-test run overlapped builds and failed the existing100ms
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
glTF mapping `(x,-z,y)`,70-degree vertical FOV and0.05m near clip. The native
shared-camera unit test passes in the workspace suite. This source inspection
checks shared projection intent, not Cycles/native PBR or human mood agreement;
existing Blender4.5.3 originals remain separate, without a new offline-render claim.

Updated canonical Scout presentation, report-side logging usage/runtime guide,
and #463/#483 delivery reports. The parser/collector/helper and tests own this
verification change; application rendering and Universe U01–U34 behavior have
no implementation changes here.
