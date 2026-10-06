# Diagnostics

`salimon-diagnostics` owns the optional engineering overlay. It accepts
typed runtime, renderer, and future domain measurements; maintains a bounded
frame window; formats an honest diagnostic snapshot; and CPU-rasterizes that
snapshot into a borrowed RGBA image for the renderer to composite.

The crate has no platform, GPU, renderer, or gameplay dependencies. It does not
sample clocks or query hardware itself. The runtime decides when to toggle and
record diagnostics, the renderer reports measurements it owns, and world/ship
modules provide optional domain values through `DomainMetrics`.

## Public contract

- `Diagnostics` is hidden by default and exposes `toggle`, `is_visible`,
  `set_scale_factor`, `reset_frame_window`, `record_memory_warning`, and
  `record_presented`.
- `FrameSample` carries one successfully presented frame's runtime and renderer
  measurements. Zero frame intervals are retained as the latest state but are
  excluded from the rolling statistics window.
- `DomainMetrics` and `BodyDistance` borrow caller-owned state. Runtime supplies
  all six catalog names and nonnegative camera-to-nominal-surface observations;
  the panel displays the closest one. Camera prototype telemetry is explicitly
  named; missing character or ship measurements render as `N/A`. The
  diagnostics layer never invents positions, speeds, or distances.
- `overlay` returns a borrowed `OverlayImage` only while diagnostics are visible.
  The renderer can use `revision` to avoid uploading unchanged pixels and scales
  the panel down uniformly when the drawable cannot contain it at source size.
- `overlay_text` exposes the matching text snapshot for tests, logging, and
  accessibility-oriented inspection.

The rolling window contains at most 120 valid frame samples. Visible text and
pixels refresh at most once per 250 milliseconds of successfully presented frame
time, except for state changes that need immediate feedback such as toggling,
display-density changes, frame-window resets, and memory warnings.

## Metric semantics

- FPS is derived from the average nonzero presented-frame interval. Frame time
  shows average and nearest-rank p95 values for the same rolling window.
- CPU render and update times are averages of supplied samples. The runtime
  supplies renderer encoding/submission wall time, excluding present wait, and
  runtime supplies the measured portable-world update/mapping time. A future
  frame with unavailable update timing would still show `N/A` and be excluded
  from that average.
- GPU timing distinguishes unsupported hardware, an asynchronous result that is
  pending, and a measured duration. CPU submission time must never be supplied
  as GPU time.
- Visible/rendered object and scene/total draw-call counts are caller-reported.
  Producers must document whether a count includes diagnostic composition.
- GPU allocator memory is optional and reports allocated/reserved bytes. It is
  not a claim about whole-process memory.
- Camera/player/ship position, camera altitude, velocity, speed, and
  body-distance values use meters as their input unit and are formatted with
  practical metric prefixes. The nearby-body row describes the closest
  distance from the engineering camera to a nominal body surface, not player or
  ship state. Camera phase and pause state describe only the engineering
  transition fixture.

Normal gameplay flight information still belongs on cockpit displays. This
overlay is an engineering surface and must remain optional.

See [README.ai.md](README.ai.md), [architecture.md](architecture.md), and
[invariants.md](invariants.md) before changing its contract.
