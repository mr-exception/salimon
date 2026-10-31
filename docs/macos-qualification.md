# macOS qualification

This is optional platform/release qualification under the
[cloud-first workflow](verification-workflow.md), not a routine feature PR gate.
Use an exact clean source revision and optimized executable, isolated saves and
settings, normal wall clock and a visible, unobscured game window. Record actual
Mac/OS/backend, physical drawable pixels, display scale/refresh, AC/battery state,
source/binary hashes, instrumentation and competing work. Finish compilation
before serial performance runs. An M1 claim requires actual M1 hardware.

## Automated collection

```sh
cargo build --release --locked -p salimon-client
WGPU_BACKEND=metal python3 scripts/verify_universe_expansion.py --binary target/release/salimon-client --output artifacts/mac-universe
WGPU_BACKEND=metal python3 scripts/benchmark_canonical.py --binary target/release/salimon-client --profile release --sample-seconds 30 --output artifacts/mac-canonical
```

The native route matrix uses isolated protocol/fixed-step controls and actual
production GPU readbacks. It proves those rendered routes on the recorded Metal
context, not physical input, ordinary-clock performance or approved appearance.
The canonical benchmark uses ordinary startup and OS-injected Return, five-second
warmup and consecutive normal-clock submitted frames. Repeat in three fresh
processes with distinct output paths; retain failed/unfocused runs as well as
accepted ones. The benchmark restores activation after its window helper; any
sampled focus loss, occlusion, missing frame or fewer than 100 frames rejects the
window. Keep raw frames/logs and report missing GPU counters as unavailable.

`macos_window.swift PID [capture.png]` inventories only that PID's on-screen
layer-zero window. It captures that window through the macOS compositor, excluding
the rest of the desktop. Its bounds include decorations and use logical points;
native renderer logs establish drawable pixels. Display mode pixels and logical
mode dimensions are separate fields. Swift requires Xcode command-line tools;
OS input/capture require the relevant existing macOS permissions.

For the packaged OS Enter/F2 smoke, use the existing local capture option:

```sh
SALIMON_SAVE_DIR="$PWD/artifacts/mac-smoke-saves" SALIMON_SETTINGS_FILE="$PWD/artifacts/mac-smoke-settings.json" WGPU_BACKEND=metal python3 scripts/packaged_smoke.py --binary target/release/salimon-client --artifacts artifacts/mac-smoke --screenshot-command '["swift","scripts/macos_window.swift","{pid}","{path}"]'
```

Pass an **absolute** helper path in the screenshot argv when running from a
different directory. The helper is a trusted local argv, never scenario-supplied
shell code. OS-injected Enter/F2 and received-event logs are distinct from human
keyboard/mouse trials. The [fragment benchmark guide](../scripts/FRAGMENT_BENCHMARKS.md)
owns matched foreground load matrices; do not substitute naturally sleeping loads
for active-load acceptance or compare different source/settings as a regression.

## Physical session, deferred until an operator is available

1. Record reviewer/date, revision/binary hash, device/input, display/refresh/DPI,
   power and isolated paths. Begin original video and received native-event logs.
2. Exercise real movement/look in both directions, cockpit/door/ramp/EVA/cargo,
   tray/intake/Fabricator/Q, Archive/Recovery, scanner/chart and construction
   controls. Check focus loss/regain, repeat/held/modal exclusion, resize and
   physically supported scales. For each criterion record expected/observed state;
   unsupported display modes are unrun.
3. With normal clock, demonstrate a positively admitted held control spanning
   **actual OS suspend/wake**. Have the operator sleep/wake/unlock. Use OS power
   timestamps, original video and runtime traces to establish reset **before**
   fresh input or a post-wake pause. A dark display, process stop or clock jump is
   insufficient. Keep helper deadlines longer than sleep and avoid the historical
   expired-hold/timestamp-overlap ambiguity.
4. Every Save Now observation needs a **new generation/current UTC**. Capture
   before-sleep, immediate after-wake and after-idle states; stale saves are failed
   observations. Check bounded simulation advance and renderer/audio recovery.
5. Return originals, raw OS/runtime/save logs, timestamps/hashes, revision and
   criterion checklist to Codex. Preserve failures; do not replace them with an
   overall impression.

## Presentation, budgets and routes

Review original images/audio and normal-clock motion separately. Record exact
artifact hashes and per-group accept/reject decisions for materials, lighting,
text/units, layouts, depth, modal priority, sound orientation/timbre/gaps and
physical comfort. Image/mask/tolerance approval and baseline promotion require
explicit scoped review and a separate baseline PR; Metal and software Vulkan
contexts cannot inherit each other's approval.

For hardware budgets, use matched foreground release workloads and repeated
CPU/render/GPU/submitted-frame distributions, retaining spikes, timeouts and
memory limitations. No single static view certifies sustained flight/terrain/
streaming, physical scanout, historical active 500-piece budgets or arbitrary
population density. Historical failures and independent regression limits remain.

For survival/recovery, use ordinary supplies, no debug placements and no assumed
travel. Record normal, impaired and poor-layout/scarce routes: discovery/setup,
navigation/gather/return/intake, paid crafting, exact-tray collection and Q use;
also module installation/Core replenishment/Archive R. Keep identities, joules,
mass, initial/final meters, exclusive stage times and independent elapsed total.
Validate ration records with `scripts/survival_route_review.py`; leave unrun
phases null and retain unavailable-feedstock/failed trials. Fixture sensitivities
are supporting automated evidence, not measured acquisition. Proposed 360/720 s
targets and recovery tuning remain owner decisions; preserve approved survival
clocks and existing construction economy decisions.

Owners: runtime [architecture](../client/runtime/architecture.md) and
[invariants](../client/runtime/invariants.md), renderer
[architecture](../client/renderer/architecture.md) and
[invariants](../client/renderer/invariants.md). This collection workflow changes
no runtime, physics, persistence, source rights or approved numerical contracts.
