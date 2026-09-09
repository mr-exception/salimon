# World AI Maintenance Guide

## Read before changing

1. Fetch the active Notion task, Phase 0 specification, and technical
   architecture as required by the root `AGENTS.md`.
2. Read [coordinate-strategy.md](coordinate-strategy.md),
   [architecture.md](architecture.md), and [invariants.md](invariants.md).
3. Read the runtime and renderer maintenance guides before changing their typed
   integration boundary.

## Ownership

Keep absolute coordinates and portable camera behavior here. Use `f64` meters
until after origin subtraction, and expose renderer-neutral snapshots. Do not add
window events, GPU types, renderer projections, gameplay simulation, persistence,
networking, or real Task 5 world content.

## Change checklist

- Preserve the subtract-before-cast precision contract.
- Keep tour commands and updates deterministic and independently testable.
- Keep all snapshot values finite and all physical distances positive.
- Update measured float spacings when validation distances or the anchor change.
- Add tests for transition boundaries, large deltas, pause/restart behavior, and
  precision budgets.
- Run the workspace build, format, lint, and test gates plus the native visual
  precision/depth smoke test after integration.
