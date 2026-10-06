# Issue #110 — Current native-first documentation

Date: 2026-10-06  
Base revision: `a5cd238e01f196f1129a21b1c64e51a2eb3ec6b6`

## Outcome

Aligned maintained product/architecture guides with the native-first Rust,
`winit` and `wgpu` client. macOS remains the reference playable/performance target;
native builds also target Windows and Debian-based Linux. Web/WASM and the
backend remain future capabilities.

The [phase guide](../../docs/project-phases.md) distinguishes the original
navigation baseline, dated GO WITH REVISIONS evaluation and current prototype.
Mining, physical fragments, one-object carrying, transfer, contact and EVA are
implemented; local streaming retention is in-memory session state, not a save
system. No new numbered phase or formal performance closure was invented.
Replaced retired Notion workflow references in crate AI guides, reduced Task N
wording in maintained ownership guides, added math/physics to the root workspace
map, and corrected the stale iron-fragment migration description.

## Scout count clarification

The issue described a 15-collider contract at filing time. Main now includes
issue #111's generated traversal geometry: **20 colliders and four markers**.
[Model contracts](../../models/contracts.md) already record that count. Verified
the checked-in sidecar against `REQUIRED_COLLIDERS` and `MARKERS` in the scout
extractor; retained the current count rather than reverting it to 15.

## Validation

Environment: Linux, Python 3.12.14. Documentation-only changes.

- `git diff --check`: passed.
- `python /tmp/check110.py`: passed ad hoc documentation audit: checked changed
  Markdown relative links and heading anchors, compared generated scout collider
  identities/count and marker count to extractor/guide, checked crate AI guides
  for retired Notion instructions, and verified no existing report or dated
  evaluation changes. This was a temporary audit script, not a runtime test.
- Source/path review: Cargo workspace/dependencies, resource mesh exports,
  runtime presentation, native build workflow and the generated scout sidecar
  agree with the documented boundaries. All changed paths are Markdown files.
- `python models/assets/ships/salimon-scout/validate.py`: failed to start full
  validation because model requirements (`jsonschema`) are absent. The independent
  standard-library contract inventory check passed; no model/export was changed.
- Rust build/tests/Clippy, Blender regeneration and native graphical checks:
  not run, as runtime, models and generated exports are unchanged. No new GPU,
  performance or visual claim is made.

## Evidence integrity and limitations

Existing `reports/` evidence and `docs/phase-0-evaluation.md` remain unchanged.
Historical sections inside the baseline specification retain their original
numbers/results and are explicitly labeled by date or historical scope.
No screenshots apply to this documentation task. Future phase planning, web
shipping and the reference-machine performance floor remain separate decisions.
