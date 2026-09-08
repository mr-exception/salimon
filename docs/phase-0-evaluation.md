# Phase 0 go / revise / stop evaluation

Evaluation date: 2026-09-15  
Evaluated revision: `bddaca2a57678433b772b17cb7ee9fe2b60b9789`  
Decision: **GO WITH REVISIONS**

## Executive conclusion

The custom Rust, `winit`, and `wgpu` approach is viable for Salimon's next
prototype phase. Phase 0 demonstrated the important architectural risks at the
intended boundaries: a native lifecycle, camera-relative interplanetary
coordinates, reverse-Z depth, scalable sphere rendering, a runtime-loaded custom
ship, first-person movement, direct-speed flight, and assisted surface
transitions. The implementation does not justify stopping or replacing the
renderer with a full game engine.

The result is not an unconditional “go as-is.” The renderer has ample measured
CPU and GPU headroom on the reference machine, but the current diagnostics do
not prove the literal “never below 60 FPS” requirement. The sampled p95 presented
frame interval is slightly over a 60 Hz frame, and there is no durable benchmark
export. Phase 1 should retain the architecture while tightening performance
measurement, resolving contradictory travel-time requirements, and introducing
new domain or platform abstractions only when their next consumer exists.

## Evidence and method

The release build was evaluated on the specified reference system:

- Apple iMac `iMac21,1`, model `Z12X002L9GR/A`
- Apple M1 with 8 CPU cores and 8 GPU cores; 16 GB unified memory
- macOS 26.2 (`25C56`), native `arm64`
- Rust and Cargo 1.89.0
- requested physical drawable: 1920×1080
- optimized executable: 8,754,784 bytes
- runtime GLB: 467,856 bytes

The native release client was launched twice. Diagnostics were sampled in the
initial landed-ship view, the multi-body precision tour, and Earth's exact 12 m
near-surface dwell. Minimize/restore reset the timing history without a large
pause entering the frame window, and close/relaunch completed cleanly.

| Scenario | FPS | Frame avg | Frame p95 | CPU render avg | GPU avg | Scene draws | Visible/rendered objects |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| Landed ship, warmed sample | 60.0 | 16.66–16.67 ms | 17.12–17.51 ms | 0.44–0.63 ms | 1.07–1.09 ms | 3 | 3 / 3 |
| Precision tour, multi-body view | 60.0 | 16.66 ms | 17.20 ms | 0.53 ms | 0.87 ms | 2 | 9 / 9 |
| Earth near dwell, 12 m | 60.0 | 16.67 ms | 17.27 ms | 0.52 ms | 2.21 ms | 2 | 5 / 5 |
| After minimize/restore | 60.0 | 16.67 ms | 17.43 ms | 0.45 ms | 1.99 ms | 2 | 5 / 5 |

An early sample immediately after enabling diagnostics reported 57.6 FPS,
17.36 ms average, and 17.33 ms p95. It recovered to a 60.0 FPS average as the
rolling window filled. These are short interactive samples, not a statistically
controlled benchmark dataset.

Automated validation passed on the same revision:

```text
cargo fmt --all -- --check                                  PASS
cargo clippy --workspace --all-targets --locked -- -D warnings PASS
cargo test --workspace --locked                             PASS (140 tests)
cargo build --workspace --locked --release                  PASS
```

The 140 tests comprise 28 character, 25 runtime, 10 diagnostics, 37 renderer,
13 ship, and 27 world tests. They cover the portable movement and gravity
contracts, input mapping, lifecycle clocks, diagnostics aggregation, renderer
precision/depth/culling, ship controls, all-body landing/takeoff rules, and the
compressed catalog invariants.

## What worked

### Architecture and ownership

- The dependency split is holding: runtime composes; renderer owns GPU state;
  world, character, and ship own portable behavior; diagnostics consumes typed
  measurements. The renderer does not depend on gameplay state.
- All Phase 0 behavior remains under `client/`; `core/` is still documentation
  only. No backend, persistence, networking, or full-engine dependency leaked
  into the prototype.
- Focused crate tests are fast and failures remain attributable to a small
  module. That is consistent with the AI-maintenance architecture goal.

### Coordinate and rendering feasibility

- CPU-side `f64` absolute positions and `f64` camera-origin subtraction preserve
  local offsets before GPU `f32` conversion. Tests cover terameter translation,
  representative float spacing, and submeter surface detail.
- Infinite-far reverse-Z depth removes the finite far-clip constraint while
  preserving near-surface depth resolution. Analytic sphere intersection avoids
  large-operand cancellation near body surfaces.
- The data-driven six-body scene, generated textures, sphere LOD/culling, Sun
  lighting, ship mesh, glazing, and cockpit displays render through small,
  explicit pipelines. Sampled GPU time remained at or below 2.21 ms.

### Playable prototype behavior

- The client starts inside a landed, custom Salimon ship whose editable source,
  readable glTF, packaged GLB, material metadata, and collision/interaction
  markers are versioned together.
- First-person walking, camera-relative full-sphere movement, ship-floor versus
  radial gravity, jumping, cockpit authority, door interlocks, direct-speed
  flight, collision correction, and uncancellable landing/takeoff are protected
  by focused tests.
- The optional diagnostics distinguish CPU timing, real GPU timestamps,
  presented-frame statistics, counts, positions, transitions, and unsupported
  metrics instead of substituting misleading values.

## What failed or remains unproven

- **The literal frame-rate criterion failed strict verification.** Warmed samples
  averaged 60.0 FPS, but p95 intervals of 17.12–17.51 ms exceed the 16.67 ms
  period of 60 Hz, and one early rolling sample showed 57.6 FPS. Render work is
  far below budget, so the observed pacing is more consistent with presentation,
  compositor, scheduling, or measurement cadence than GPU saturation. The data
  is insufficient to assign a single cause.
- **Performance evidence is not durable or reproducible enough.** Metrics are
  visible in an overlay but cannot be exported with scenario, build, machine,
  sample count, and percentile metadata. The completed benchmark task explicitly
  waived the former seven-scenario dataset based on owner validation; therefore
  this evaluation must not imply that dataset exists.
- **GPU memory is unavailable.** The overlay truthfully reports
  `N/A (backend report)` on this Metal path. Whole-process peak memory and thermal
  behavior were not measured.
- **The complete manual gameplay matrix was not rerun for this evaluation.** The
  automated suite covers every solid body's rules and precision tour, while this
  run sampled representative rendered scenes plus lifecycle recovery. Visual
  inspection of every landing, takeoff, full-sphere walk, resize extreme, and
  cockpit interaction remains owner/playtest evidence rather than a recorded
  result from this evaluation.
- **Travel-time requirements conflict.** Current task-level flight behavior and
  tests specify about 24 seconds at 2,500 km/s, while older Phase 0 scene and
  specification text also says about two minutes. The implementation consistently
  chooses the newer 24-second contract, but Phase 1 planning should make one
  value canonical before tuning navigation.
- **Phase 0 rendering is deliberately non-production.** Analytic textured spheres
  have no terrain, atmosphere, clouds, dynamic shadows, or surface streaming;
  collisions are simplified; the ship is a lightweight material demo. These are
  accepted scope limits, not evidence that those later systems are solved.

## Required revisions before or at Phase 1

1. Add a benchmark capture path that writes versioned machine/build/scenario
   metadata plus sample count, average, p95, p99, maximum frame interval, CPU
   update/render timing, GPU timing state/value, and draw/object counts. Reset
   windows at scenario and lifecycle boundaries and distinguish refresh pacing
   from renderer saturation.
2. Replace “never below 60 FPS” with an observable frame-pacing contract unless
   true hard real-time behavior is intended. A useful contract should define
   warm-up, sample duration, percentile and hitch thresholds, foreground/window
   state, and treatment of OS compositor events. Keep 1920×1080 and the reference
   M1 machine as the baseline.
3. Resolve the 24-second versus two-minute Earth-to-Mars requirement in Project
   Q&A and update the Phase 0/Phase 1 source documents consistently. Do not retune
   the implementation until that product decision is explicit.
4. Preserve the current camera-relative/reverse-Z renderer, but introduce a
   body-local or patch-local spatial hierarchy when terrain or dense surface
   content begins. A single camera rebase is sufficient for this fixture, not a
   complete universe representation.
5. Keep `winit` at the runtime boundary for the current macOS client. Extract a
   platform adapter when Windows or web work creates a second implementation;
   doing it earlier would add an abstraction without a tested consumer.
6. Add ECS/WIT/plugin boundaries only for Phase 1 domains that require independent
   scheduling, authority, or replacement. The current typed Rust crate interfaces
   are adequate for this single-process prototype and should not be rewritten
   wholesale.

## Recommendation

**GO WITH REVISIONS.** Continue to Phase 1 on the custom Rust/`wgpu` foundation.
Do not stop the project and do not migrate to a full game engine based on Phase 0
results. The measured renderer cost, precision strategy, lifecycle behavior, and
module testability support the architecture.

Treat benchmark export and a precise frame-pacing acceptance contract as the
first engineering revision. Resolve the travel-time product contradiction before
navigation tuning. Introduce terrain-scale spatial hierarchy, platform adapters,
and stronger component boundaries incrementally when Phase 1 scope exercises
them; none requires invalidating the Phase 0 foundation.
