# World AI Maintenance Guide

## Read before changing

1. Fetch the active GitHub issue, project specification, and technical
   architecture as required by the root `AGENTS.md`.
2. Read [solar-system-layout.md](solar-system-layout.md),
   [coordinate-strategy.md](coordinate-strategy.md),
   [architecture.md](architecture.md), and [invariants.md](invariants.md).
3. Read the runtime and renderer maintenance guides before changing their typed
   integration boundary.

## Ownership

Keep the immutable compressed body catalog, absolute coordinates, body/landing
geometry, reference speed, and portable camera behavior here. Use `f64` meters
until after origin subtraction, and expose renderer-neutral snapshots. Keep the
three Task 4 precision markers separate from the six canonical bodies. Do not add
window events, GPU types, renderer projections, disk/backend persistence,
networking, textured sphere rendering, or lighting. Portable resource contracts
belong in `resources`; read [resource-contracts.md](resource-contracts.md) before
extending them. Portable generation, extraction, fragment output, and one-object carrying live
in their own modules. Runtime composes character/ship placement and fragment
motion/contact; this crate owns validated physical identities, mass, poses, and
mining session retention across local streaming. Read the mining contract and focused tests
before changing aim, rate, session mutation, or obstruction semantics.
`resource_fragments` owns conservative output and surface poses; do not move
fragment creation or mass splitting into runtime or renderer. Querying output
must remain read-only. Seal a growing output tail on pickup so subsequent extraction cannot change
the carried object's mass.

## Change checklist

- Preserve the exact ordered membership: Sun, Mercury, Venus, Earth, Moon, Mars.
- Keep the Sun visual-only and all five `1.15R` solid landing volumes disjoint.
- Preserve the 24-second Earth-to-Mars surface trip at the Phase 0 reference
  maximum speed unless a source-of-truth decision changes it.
- Preserve the subtract-before-cast precision contract.
- Keep tour commands and updates deterministic and independently testable.
- Keep all catalog values finite, positive where required, and immutable.
- Preserve stable resource keys and private validated mass/pose state. Run the
  resource contract tests when changing catalog entries or domain structures.
- Update the layout table, measured float spacings, tests, and runtime mapping
  together when catalog geometry changes.
- Run the workspace build, format, lint, and test gates plus the native visual
  scene/precision/depth smoke test after integration.

## Shared maintenance rules

Follow the root [coding conventions](../../docs/coding-conventions.md),
[validation matrix](../../docs/validation.md) and
[feature map](../../docs/maintenance-map.md). Update affected contracts/guides
with behavior changes and record completion evidence under root `reports/`.
