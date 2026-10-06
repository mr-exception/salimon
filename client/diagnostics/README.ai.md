# Diagnostics AI Maintenance Guide

## Purpose

`salimon-diagnostics` converts typed observations into a bounded statistical
snapshot and CPU-rasterized RGBA panel. It is an observer. It does not own the
event loop, GPU resources, authoritative domain state, or benchmark policy.

## Read before changing

1. Fetch the active GitHub issue, affected project specification, and technical
   architecture as required by the root `AGENTS.md`.
2. Read [architecture.md](architecture.md) and [invariants.md](invariants.md).
3. Read the runtime and renderer maintenance guides before changing either side
   of their diagnostics integration.

## Ownership and dependencies

The crate owns metric aggregation, display formatting, the embedded tiny ASCII
font, CPU rasterization, the overlay pixel revision, and unavailable-state
semantics. It uses only the Rust standard library.

Callers own all source measurements. Keep `FrameSample`, `DomainMetrics`, and
`OverlayImage` typed and narrow. Camera telemetry must remain explicitly distinct
from player and ship state. Do not add `winit`, `wgpu`, world, character, or ship
dependencies here; doing so would invert the intended observation flow.

## Change checklist

- Preserve the 120-sample bound and ignore zero intervals in frame statistics.
- Preserve the 250-millisecond overlay refresh cadence and immediate refreshes
  for explicit UI/state changes.
- Keep unsupported, pending, and measured GPU states distinct.
- Render absent or invalid domain measurements as `N/A`; never synthesize
  plausible gameplay values.
- Keep raster dimensions and `rgba8.len() == width * height * 4` consistent.
- Bump the image revision only when pixels are rebuilt.
- Add deterministic tests for formatting, aggregation, cadence, and raster
  changes.
- Run root formatting, lint, build, and test gates plus the native smoke check
  after integration.

## Shared maintenance rules

Follow the root [coding conventions](../../docs/coding-conventions.md),
[validation matrix](../../docs/validation.md) and
[feature map](../../docs/maintenance-map.md). Update affected contracts/guides
with behavior changes and record completion evidence under root `reports/`.
