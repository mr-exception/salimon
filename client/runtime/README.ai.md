# Runtime AI Maintenance Guide

## Purpose

`salimon-client` is the native composition executable. Its reasoning scope is
application lifecycle, event routing, redraw scheduling, frame timing, and
coordination of client capabilities. It is not a gameplay domain.

## Read before changing

1. Fetch the active Notion task, Phase 0 specification, and technical
   architecture as required by the root [AGENTS.md](../../AGENTS.md).
2. Read [architecture.md](architecture.md) and [invariants.md](invariants.md).
3. Read the renderer's [maintenance guide](../renderer/README.ai.md) before
   changing runtime/renderer interaction.

## Ownership

The runtime owns the `winit` application handler, native window lifetime,
renderer orchestration, resize and redraw routing, surface-loss recovery policy,
and monotonic frame clock. Its current outbound dependency is
`salimon-renderer`.

Do not add GPU pipelines/resources, authoritative world or ship state, backend
behavior, persistence, or networking here. Keep new platform-specific behavior
small and plan extraction to `client/platform/` when reusable adapters become
necessary.

## Change checklist

- Preserve the lifecycle and timing invariants.
- Keep the renderer call surface narrow and typed.
- Add or update deterministic tests for non-GUI logic.
- Run all root build/format/lint/test gates.
- Perform the root native smoke check for lifecycle or rendering changes.
- Update this guide and the component architecture/invariants when ownership or
  behavior intentionally changes.
