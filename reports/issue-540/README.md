# Base Scout habitation (#540)

Second PR in the requested stack, based on `stack/539-cargo` / PR #596.

The bunk is now a real manually folding 0.85 m berth. Two existing cabin-owned
Blender panels rotate rigidly around the wall hinge; normals rotate with them.
The full swept player/belonging volume is checked before motion and while moving.
Blocked motion never deletes, relocates or duplicates objects. Existing full
walking clearance stays reserved and static poses perform no repeated clearance
scan. Blackout permits the manual mechanism; pause freezes it. State, direction
and intermediate position survive whole-session saves through schema 10; v1–v9
migrate stowed without ticking.

Galley E opens the existing exact-ID consumable wheel. Meals still come from the
existing Fabricator and occupy the existing food slots. No free food, water,
health, fatigue or other survival benefit is generated. The hygiene fixture is
inspectable, with its lack of a separate hygiene/water need stated honestly.
Sitting/lying, privacy-screen mechanisms, full tactile animation and restraints
remain the distinct existing #577 scope; this delivers the base #540 fixture.

`habitation-geometry.json` is the single source for hinge, panel dimensions and
clearance. The category exporter generates renderer/character consumers. Only the
two named cabin meshes change; linked libraries, materials, parents, all authored
anchors/colliders and the other components are preserved. Current export remains
6,767 triangles, 134 primitives and 13 materials, with 581,380 GLB bytes (16 bytes
smaller than the parent). No limit is raised. The inherited material named
Terracotta Upholstery currently exports a slate color; no palette approval or
material retuning is inferred here.

## Validation

macOS 26.2, locked Rust workspace, supported Blender 4.5.3 downloaded to task-local
scratch (installed Blender 5.2.2 is not used for the source edit).

- `cargo test --workspace --locked`: pass; runtime 364 passed / 13 opt-in ignored,
  all other crates/doc tests pass. Later generated-hinge/idle-scan refinements
  are covered by the focused/native checks below.
- `cargo test -p salimon-client --locked habitation -- --nocapture`: 5 passed.
  Physical aim and distance, outside refusal, exact pause/save/blackout motion,
  obstruction without object mutation, existing inventory/energy/survival
  preservation, strict current schema and legacy migration.
- `cargo test -p salimon-renderer --locked`: 75 passed / 14 opt-in ignored;
  production shader validation is included.
- `cargo clippy --workspace --all-targets --locked -- -D warnings`: pass.
- `cargo fmt --all -- --check`, `git diff --check`: pass.
- Category `author_habitation.py`, `export.py --blender <4.5.3 binary>`, read-only
  `verify_source.py` and `validate.py`: pass. Full outputs in [validation](validation/).
  One draft export rejected changed legacy extras and another rejected a flush
  frame/cushion surface; the source now preserves extras and leaves a 5 mm gap.
  No validator exemption or budget increase was introduced.
- `python3 scripts/check_docs.py`: pass, zero findings.
- `cargo build -p salimon-client --locked`: pass.
- `python3 scripts/salimon-test run scenarios/evidence/habitation.json --binary target/debug/salimon-client --screenshot-command '["native"]' --summary`:
  pass, all 14 steps. Actual production wgpu captures demonstrate stowed,
  deployed and restowed state; originals and exact protocol/state/result bytes
  are in [native](native/), compressed with a SHA-256 inventory.

[Stowed](native/step-006-bunk-stowed.png) ·
[Deployed](native/step-010-bunk-deployed.png) ·
[Restowed](native/step-014-bunk-restowed.png)

This is native macOS production-renderer evidence, not hosted Linux/software
Vulkan CI, human appearance/comfort acceptance (#561), physical input/platform
qualification (#557), or hardware frame-budget signoff (#563). No PR is merged.
